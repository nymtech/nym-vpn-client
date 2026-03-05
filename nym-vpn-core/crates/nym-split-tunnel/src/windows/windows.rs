// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_windows::io::Overlapped;
use std::{
    ffi::{OsStr, OsString},
    fs, io, iter, mem,
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        io::{AsRawHandle, RawHandle},
    },
    path::{Component, Path},
    ptr,
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, ERROR_INSUFFICIENT_BUFFER, FALSE, FILETIME, HANDLE, TRUE, WAIT_ABANDONED,
        WAIT_ABANDONED_0, WAIT_FAILED, WAIT_OBJECT_0,
    },
    Storage::FileSystem::{GetFinalPathNameByHandleW, QueryDosDeviceW, VOLUME_NAME_NT},
    System::{
        IO::GetOverlappedResult,
        ProcessStatus::GetProcessImageFileNameW,
        Threading::{
            GetProcessTimes, INFINITE, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
            WaitForMultipleObjects, WaitForSingleObject,
        },
    },
};

/// Obtains a device path without resolving links or mount points.
pub fn get_device_path<P: AsRef<Path>>(path: P) -> Result<OsString, io::Error> {
    // Preferentially, use GetFinalPathNameByHandleW. If the file does not exist
    // or cannot be opened, infer the path from the label only.
    if let Ok(file) = fs::OpenOptions::new().read(true).open(path.as_ref()) {
        return unsafe { get_final_path_name_by_handle(file.as_raw_handle() as HANDLE) };
    }

    let mut components = path.as_ref().components();
    let drive_comp = components.next();
    let drive = match (drive_comp, components.next()) {
        (Some(Component::Prefix(prefix)), Some(Component::RootDir)) => prefix.as_os_str(),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path must be absolute",
            ));
        }
    };

    let mut new_path = query_dos_device(drive)?;
    let suffix = path
        .as_ref()
        .strip_prefix(drive_comp.unwrap())
        .map_err(|_error| {
            io::Error::new(io::ErrorKind::InvalidInput, "path missing own component")
        })?;
    new_path.push(suffix);

    Ok(new_path)
}

pub unsafe fn get_final_path_name_by_handle(raw_handle: HANDLE) -> Result<OsString, io::Error> {
    let buffer_size =
        unsafe { GetFinalPathNameByHandleW(raw_handle, ptr::null_mut(), 0u32, VOLUME_NAME_NT) }
            as usize;
    let mut buffer = vec![0; buffer_size];

    let status = unsafe {
        GetFinalPathNameByHandleW(
            raw_handle,
            buffer.as_mut_ptr(),
            buffer_size as u32,
            VOLUME_NAME_NT,
        )
    } as usize;

    if status == 0 {
        return Err(io::Error::last_os_error());
    }

    buffer.resize(buffer_size - 1, 0);
    Ok(OsStringExt::from_wide(&buffer))
}

/// Obtains the real device path for a label (such as C:).
/// The underlying function may return multiple paths, but only the first is returned.
fn query_dos_device<T: AsRef<OsStr>>(device_name: T) -> io::Result<OsString> {
    let device_name_c: Vec<u16> = device_name
        .as_ref()
        .encode_wide()
        .chain(iter::once(0u16))
        .collect();
    let mut new_prefix = vec![0u16; 64];

    loop {
        let prefix_len = unsafe {
            QueryDosDeviceW(
                device_name_c.as_ptr(),
                new_prefix.as_mut_ptr(),
                new_prefix.len() as u32,
            ) as usize
        };

        if prefix_len == 0 {
            let last_error = io::Error::last_os_error();
            if last_error.raw_os_error() == Some(ERROR_INSUFFICIENT_BUFFER as i32) {
                // resize buffer and try again
                new_prefix.resize(2 * new_prefix.len(), 0);
                continue;
            }
            break Err(last_error);
        }

        // We must scan for the first null terminator
        // Because `new_prefix` may contain multiple strings.

        let real_len = new_prefix.iter().position(|&c| c == 0u16).unwrap();
        unsafe { new_prefix.set_len(real_len) };

        break Ok(OsString::from_wide(&new_prefix));
    }
}

/// Object that frees its handle when dropped.
pub struct WinHandle(HANDLE);

impl WinHandle {
    pub fn get_raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for WinHandle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

#[repr(u32)]
pub enum ProcessAccess {
    QueryLimitedInformation = PROCESS_QUERY_LIMITED_INFORMATION,
    // TODO: could be extended
}

/// Open an existing process object.
pub fn open_process(
    access: ProcessAccess,
    inherit_handle: bool,
    pid: u32,
) -> Result<WinHandle, io::Error> {
    let handle = unsafe {
        OpenProcess(
            access as u32,
            if inherit_handle { TRUE } else { FALSE },
            pid,
        )
    };
    if handle.is_null() {
        return Err(io::Error::last_os_error());
    }
    Ok(WinHandle(handle))
}

/// Returns the age of a running process.
pub fn get_process_creation_time(handle: HANDLE) -> Result<u64, io::Error> {
    // TODO: FileTimeToSystemTime -> chrono::NaiveDateTime
    let mut creation_time: FILETIME = unsafe { mem::zeroed() };
    let mut dummy: FILETIME = unsafe { mem::zeroed() };
    if unsafe {
        GetProcessTimes(
            handle,
            &raw mut creation_time,
            &raw mut dummy,
            &raw mut dummy,
            &raw mut dummy,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }

    let time =
        ((creation_time.dwHighDateTime as u64) << u32::BITS) | (creation_time.dwLowDateTime as u64);
    Ok(time)
}

/// Returns the device path for a running process.
pub fn get_process_device_path(handle: HANDLE) -> Result<OsString, io::Error> {
    let mut initial_capacity = 512;
    let mut iterations = 0;
    loop {
        let result = get_process_device_path_inner(handle, initial_capacity);
        match result {
            Ok(path) => return Ok(path),
            Err(error) => {
                if error.raw_os_error().unwrap() as u32 == ERROR_INSUFFICIENT_BUFFER {
                    // Try again with a larger buffer, up to a point
                    iterations += 1;
                    if iterations > 3 {
                        return Err(io::Error::other("the process path is too long"));
                    }
                    initial_capacity *= 2;
                    continue;
                }
                return Err(error);
            }
        }
    }
}

fn get_process_device_path_inner(
    handle: HANDLE,
    buffer_capacity: usize,
) -> Result<OsString, io::Error> {
    let mut buffer = Vec::<u16>::new();
    buffer.reserve_exact(buffer_capacity);

    let written = unsafe {
        GetProcessImageFileNameW(
            handle,
            buffer.as_mut_ptr() as *mut _,
            buffer.capacity() as u32,
        )
    };
    if written == 0 {
        return Err(io::Error::last_os_error());
    }

    // `written` does not include a null terminator
    unsafe { buffer.set_len(written as usize) };

    Ok(OsStringExt::from_wide(&buffer))
}

/// Waits for an object to be signaled, or until a timeout interval has elapsed.
///
/// # Safety
///
/// * `object` must be a valid object that can be signaled, such as an event object.
pub unsafe fn wait_for_single_object(
    object: &impl AsRawHandle,
    timeout: Option<Duration>,
) -> io::Result<()> {
    let timeout = match timeout {
        Some(timeout) => u32::try_from(timeout.as_millis()).map_err(|_error| {
            io::Error::new(io::ErrorKind::InvalidInput, "the duration is too long")
        })?,
        None => INFINITE,
    };
    let result = unsafe { WaitForSingleObject(object.as_raw_handle(), timeout) };
    match result {
        WAIT_OBJECT_0 => Ok(()),
        WAIT_FAILED => Err(io::Error::last_os_error()),
        WAIT_ABANDONED => Err(io::Error::other("abandoned mutex")),
        error => Err(io::Error::from_raw_os_error(error as i32)),
    }
}

/// Waits for one or several objects to be signaled. On success, this returns a pointer to an
/// object in `objects` that was signaled.
///
/// # Safety
///
/// * `objects` must be a slice of valid objects that can be signaled, such as event objects.
pub unsafe fn wait_for_multiple_objects(
    objects: &[RawHandle],
    wait_all: bool,
) -> io::Result<RawHandle> {
    unsafe {
        let objects_len = u32::try_from(objects.len())
            .map_err(|_error| io::Error::new(io::ErrorKind::InvalidInput, "too many objects"))?;
        let result = WaitForMultipleObjects(
            objects_len,
            objects.as_ptr(),
            if wait_all { 1 } else { 0 },
            INFINITE,
        );
        let signaled_index = if result < objects_len {
            result
        } else if result >= WAIT_ABANDONED_0 && result < WAIT_ABANDONED_0 + objects_len {
            return Err(io::Error::other("abandoned mutex"));
        } else {
            return Err(io::Error::last_os_error());
        };
        Ok(objects[usize::try_from(signaled_index).expect("usize must be larger than u32")])
    }
}

/// Retrieves the result of an overlapped operation. On success, this returns
/// the number of bytes transferred. For device I/O, this is the number of bytes
/// written to the output buffer.
///
/// # Panics
///
/// This function will panic if `overlapped` does not contain an event.
pub fn get_overlapped_result(handle: HANDLE, overlapped: &mut Overlapped) -> io::Result<u32> {
    let event = overlapped.get_event().unwrap();

    // SAFETY: This is a valid event object.
    unsafe { wait_for_single_object(event, None) }?;

    // SAFETY: The handle and overlapped object are valid.
    let mut returned_bytes = 0u32;
    let result =
        unsafe { GetOverlappedResult(handle, overlapped.as_mut_ptr(), &raw mut returned_bytes, 0) };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(returned_bytes)
}
