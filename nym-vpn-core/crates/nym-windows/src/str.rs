use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

/// Convert a Rust string to a null-terminated wide string for Win32 APIs.
pub fn wstr(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}
