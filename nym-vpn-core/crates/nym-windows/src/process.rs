// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    ffi::{CStr, c_char},
    mem,
    os::windows::io::AsRawHandle,
};
use tokio::net::windows::named_pipe::NamedPipeServer;
use verifysign::CodeSignVerifier;
use windows::Win32::{
    Foundation::{CloseHandle, ERROR_NO_MORE_FILES, HANDLE},
    System::{
        Diagnostics::ToolHelp::{
            CREATE_TOOLHELP_SNAPSHOT_FLAGS, CreateToolhelp32Snapshot, MODULEENTRY32, Module32First,
            Module32Next, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        },
        Pipes::GetNamedPipeClientProcessId,
    },
};

/// Errors returned by some functions in this module.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error returned from `GetNamedPipeClientProcessIdFailed`
    #[error("failed to get client PID")]
    GetNamedPipeClientProcessId(#[source] windows::core::Error),

    /// Could not get verification data for the given PID
    #[error("failed to get the verifier for the client PID: {0:?}")]
    PidVerifier(verifysign::Error),

    /// Certificate signature verification failure
    #[error("failed to verify the certificate signatures {0:?}")]
    Verification(verifysign::Error),

    /// No serial number found in signature
    #[error("no serial number found in signature")]
    NoSerialNumber,

    /// Serial number mismatch
    #[error("serial number of certificate signature is not from Nym")]
    SerialNumberMismatch,
}

/// A snapshot of process modules, threads, and heaps
pub struct ProcessSnapshot {
    handle: HANDLE,
}

impl ProcessSnapshot {
    /// Create a new process snapshot using `CreateToolhelp32Snapshot`
    pub fn new(
        flags: CREATE_TOOLHELP_SNAPSHOT_FLAGS,
        process_id: u32,
    ) -> windows::core::Result<ProcessSnapshot> {
        Ok(ProcessSnapshot {
            handle: unsafe { CreateToolhelp32Snapshot(flags, process_id) }?,
        })
    }

    /// Return the raw handle
    pub fn as_raw(&self) -> HANDLE {
        self.handle
    }

    /// Return an iterator over the modules in the snapshot
    pub fn modules(&self) -> ProcessSnapshotModules<'_> {
        let mut entry: MODULEENTRY32 = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<MODULEENTRY32>() as u32;

        ProcessSnapshotModules {
            snapshot: self,
            iter_started: false,
            temp_entry: entry,
        }
    }

    /// Return an iterator over the processes in the snapshot
    pub fn processes(&self) -> ProcessSnapshotEntries<'_> {
        let mut entry: PROCESSENTRY32W = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

        ProcessSnapshotEntries {
            snapshot: self,
            iter_started: false,
            temp_entry: entry,
        }
    }
}

impl Drop for ProcessSnapshot {
    fn drop(&mut self) {
        if let Err(e) = unsafe { CloseHandle(self.handle) } {
            tracing::error!("Failed to close process snapshot handle: {}", e);
        }
    }
}

/// Description of a snapshot module entry. See `MODULEENTRY32`
pub struct ModuleEntry {
    /// Module name
    pub name: String,
    /// Module base address (in the owning process)
    pub base_address: *const u8,
    /// Size of the module (in bytes)
    pub size: usize,
}

/// Module iterator for [ProcessSnapshot]
pub struct ProcessSnapshotModules<'a> {
    snapshot: &'a ProcessSnapshot,
    iter_started: bool,
    temp_entry: MODULEENTRY32,
}

impl Iterator for ProcessSnapshotModules<'_> {
    type Item = windows::core::Result<ModuleEntry>;

    fn next(&mut self) -> Option<windows::core::Result<ModuleEntry>> {
        if self.iter_started {
            if let Err(last_error) =
                unsafe { Module32Next(self.snapshot.as_raw(), &mut self.temp_entry) }
            {
                return if last_error.code() == ERROR_NO_MORE_FILES.to_hresult() {
                    None
                } else {
                    Some(Err(last_error))
                };
            }
        } else {
            if let Err(e) = unsafe { Module32First(self.snapshot.as_raw(), &mut self.temp_entry) } {
                return Some(Err(e));
            }
            self.iter_started = true;
        }

        let cstr_ref = &self.temp_entry.szModule[0];
        let cstr = unsafe { CStr::from_ptr(cstr_ref as *const c_char) };
        Some(Ok(ModuleEntry {
            name: cstr.to_string_lossy().into_owned(),
            base_address: self.temp_entry.modBaseAddr,
            size: self.temp_entry.modBaseSize as usize,
        }))
    }
}

/// Description of a snapshot process entry. See `PROCESSENTRY32W`
pub struct ProcessEntry {
    /// Process identifier
    pub pid: u32,
    /// Process identifier of the parent process
    pub parent_pid: u32,
}

/// Process iterator for [ProcessSnapshot]
pub struct ProcessSnapshotEntries<'a> {
    snapshot: &'a ProcessSnapshot,
    iter_started: bool,
    temp_entry: PROCESSENTRY32W,
}

impl Iterator for ProcessSnapshotEntries<'_> {
    type Item = windows::core::Result<ProcessEntry>;

    fn next(&mut self) -> Option<windows::core::Result<ProcessEntry>> {
        if self.iter_started {
            if let Err(last_error) =
                unsafe { Process32NextW(self.snapshot.as_raw(), &mut self.temp_entry) }
            {
                return if last_error.code() == ERROR_NO_MORE_FILES.to_hresult() {
                    None
                } else {
                    Some(Err(last_error))
                };
            }
        } else {
            if let Err(e) = unsafe { Process32FirstW(self.snapshot.as_raw(), &mut self.temp_entry) }
            {
                return Some(Err(e));
            }
            self.iter_started = true;
        }

        Some(Ok(ProcessEntry {
            pid: self.temp_entry.th32ProcessID,
            parent_pid: self.temp_entry.th32ParentProcessID,
        }))
    }
}

/// Information about a client process connected to the named pipe server.
pub struct ClientProcess {
    /// Process ID
    pub pid: u32,
}

impl TryFrom<&NamedPipeServer> for ClientProcess {
    type Error = Error;

    fn try_from(named_pipe_server: &NamedPipeServer) -> Result<Self, Self::Error> {
        let mut pid: u32 = 0;

        unsafe { GetNamedPipeClientProcessId(HANDLE(named_pipe_server.as_raw_handle()), &mut pid) }
            .map_err(Error::GetNamedPipeClientProcessId)?;

        Ok(Self { pid })
    }
}

impl ClientProcess {
    pub fn verify_certificate_signature(
        &self,
        nym_certificate_serial_number: String,
    ) -> Result<(), Error> {
        let verifier = CodeSignVerifier::for_pid(self.pid).map_err(Error::PidVerifier)?;
        let context = verifier.verify().map_err(Error::Verification)?;
        let signer_serial = context.serial().ok_or(Error::NoSerialNumber)?;
        if signer_serial != nym_certificate_serial_number {
            Err(Error::SerialNumberMismatch)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};

    use super::*;

    #[tokio::test]
    async fn deny_unsigned() {
        let pipe_name = PathBuf::from(r"\\.\pipe\nym-vpn-testing");
        let server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&pipe_name)
            .unwrap();
        let _client = ClientOptions::new()
            .read(true)
            .write(true)
            .open(&pipe_name)
            .unwrap();

        let client_process = ClientProcess::try_from(&server).unwrap();
        let result = client_process.verify_certificate_signature("dummy".to_string());

        // needed for some reason on windows, otherwise the test blocks
        drop(server);

        assert!(matches!(
            result.unwrap_err(),
            Error::Verification(verifysign::Error::Unsigned)
        ))
    }
}
