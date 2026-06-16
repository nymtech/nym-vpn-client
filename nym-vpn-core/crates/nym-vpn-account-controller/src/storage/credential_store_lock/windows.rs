// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fs::{File, OpenOptions};
use std::io;
use std::mem::MaybeUninit;
use std::os::windows::io::AsRawHandle;
use std::path::Path;

use windows_sys::Win32::Foundation::{ERROR_IO_PENDING, ERROR_LOCK_VIOLATION, HANDLE};
use windows_sys::Win32::Storage::FileSystem::{
    LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY, LockFileEx,
};
use windows_sys::Win32::System::IO::{GetOverlappedResult, OVERLAPPED};

use crate::error::Error;

const LOCK_FILE_NAME: &str = "credential_store_access.lock";

pub struct CredentialStoreAccessLock {
    _file: File,
}

impl CredentialStoreAccessLock {
    pub fn try_acquire(data_dir: &Path) -> Result<Self, Error> {
        Self::acquire_with(data_dir, true)
    }

    pub fn acquire_blocking(data_dir: &Path) -> Result<Self, Error> {
        Self::acquire_with(data_dir, false)
    }

    fn acquire_with(data_dir: &Path, non_blocking: bool) -> Result<Self, Error> {
        let path = data_dir.join(LOCK_FILE_NAME);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .map_err(Error::CredentialStoreLockIo)?;

        lock_file_handle(file.as_raw_handle() as HANDLE, non_blocking)?;

        Ok(Self { _file: file })
    }
}

fn lock_file_handle(handle: HANDLE, non_blocking: bool) -> Result<(), Error> {
    let mut flags = LOCKFILE_EXCLUSIVE_LOCK;
    if non_blocking {
        flags |= LOCKFILE_FAIL_IMMEDIATELY;
        // Synchronous acquire: NULL overlapped so contention surfaces as
        // ERROR_LOCK_VIOLATION instead of ERROR_IO_PENDING.
        let ok = unsafe { LockFileEx(handle, flags, 0, u32::MAX, u32::MAX, std::ptr::null_mut()) };
        if ok != 0 {
            return Ok(());
        }
        let err = io::Error::last_os_error();
        if lock_contention_os_error(err.raw_os_error().unwrap_or(0)) {
            return Err(Error::CredentialStoreBusy);
        }
        return Err(Error::CredentialStoreLockIo(err));
    }

    let mut overlapped = MaybeUninit::<OVERLAPPED>::zeroed();
    // SAFETY: LockFileEx reads dwFlags and optional overlapped; zero-init is valid.
    let ok = unsafe {
        LockFileEx(
            handle,
            flags,
            0,
            u32::MAX,
            u32::MAX,
            overlapped.as_mut_ptr(),
        )
    };

    if ok != 0 {
        return Ok(());
    }

    let err = io::Error::last_os_error();
    let raw = err.raw_os_error().unwrap_or(0);

    if raw == ERROR_IO_PENDING as i32 {
        let mut bytes = 0u32;
        // SAFETY: overlapped was passed to LockFileEx; wait for async lock completion.
        let waited = unsafe { GetOverlappedResult(handle, overlapped.as_mut_ptr(), &mut bytes, 1) };
        if waited != 0 {
            return Ok(());
        }
        return Err(Error::CredentialStoreLockIo(io::Error::last_os_error()));
    }

    Err(Error::CredentialStoreLockIo(err))
}

fn lock_contention_os_error(raw: i32) -> bool {
    raw == ERROR_LOCK_VIOLATION as i32 || raw == ERROR_IO_PENDING as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_violation_is_contention() {
        assert!(lock_contention_os_error(ERROR_LOCK_VIOLATION as i32));
    }

    #[test]
    fn io_pending_is_contention() {
        assert!(lock_contention_os_error(ERROR_IO_PENDING as i32));
    }

    #[test]
    fn unrelated_os_error_is_not_contention() {
        assert!(!lock_contention_os_error(5));
    }
}
