// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! File system security functions.

use std::path::Path;

use windows::{
    core::{Result, HRESULT, HSTRING},
    Win32::{
        Foundation::ERROR_INSUFFICIENT_BUFFER,
        Security::{self, OBJECT_SECURITY_INFORMATION},
    },
};

use super::SecurityDescriptor;

/// Set file security information for file at path.
///
/// See [OBJECT_SECURITY_INFORMATION](https://docs.microsoft.com/en-us/windows/win32/api/winnt/ne-winnt-object_security_information) for the values accepted in `security_info`
pub fn set_file_security<P: AsRef<Path>>(
    path: P,
    security_info: OBJECT_SECURITY_INFORMATION,
    security_descriptor: SecurityDescriptor,
) -> Result<()> {
    unsafe {
        Security::SetFileSecurityW(
            &HSTRING::from(path.as_ref().as_os_str()),
            security_info,
            security_descriptor.inner(),
        )
        .ok()
    }
}

/// Get file security information for file at path.
///
/// See [OBJECT_SECURITY_INFORMATION](https://docs.microsoft.com/en-us/windows/win32/api/winnt/ne-winnt-object_security_information) for the values accepted in `requested_info`
pub fn get_file_security<P: AsRef<Path>>(
    path: P,
    requested_info: OBJECT_SECURITY_INFORMATION,
) -> Result<SecurityDescriptor> {
    let path_str = HSTRING::from(path.as_ref().as_os_str());
    let mut length_needed: u32 = 0;

    // Query size of buffer needed to store security descriptor.
    unsafe {
        Security::GetFileSecurityW(&path_str, requested_info.0, None, 0, &mut length_needed).ok()
    }
    .or_else(|err| {
        // It's expected to return the insufficient buffer error
        if err.code() == HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0) {
            Ok(())
        } else {
            Err(err)
        }
    })?;

    // Allocate buffer to store security descriptor in self-relative format.
    assert!(length_needed > 0);
    let num_bytes = usize::try_from(length_needed).expect("length needed is too large");
    let security_descriptor = unsafe { SecurityDescriptor::new_with_capacity(num_bytes)? };

    // Get security descriptor.
    unsafe {
        Security::GetFileSecurityW(
            &path_str,
            requested_info.0,
            // Safety: this will mutate the security descriptor in place
            Some(security_descriptor.inner()),
            length_needed,
            &mut length_needed,
        )
        .ok()?
    };

    Ok(security_descriptor)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use windows::Win32::{
        Foundation::{GENERIC_READ, GENERIC_WRITE},
        Security::Authorization::TRUSTEE_IS_WELL_KNOWN_GROUP,
    };

    use super::*;
    use crate::security::{Acl, ExplicitAccess, Sid, Trustee};

    #[test]
    fn test_set_file_security() {
        let mut file = tempfile::NamedTempFile::new().expect("failed to create temp file");
        file.write_all(b"test").expect("failed to write to file");

        let trustee = Trustee::new(Sid::everyone().unwrap(), TRUSTEE_IS_WELL_KNOWN_GROUP);
        let permissions = GENERIC_READ | GENERIC_WRITE;

        let mut explicit_access = ExplicitAccess::new(trustee);
        explicit_access.set_access_mode(Security::Authorization::SET_ACCESS);
        explicit_access.set_access_permissions(permissions.0);
        explicit_access.set_inheritance(Security::NO_INHERITANCE);

        let acl = Acl::new(vec![explicit_access]).unwrap();

        let mut security_descriptor =
            SecurityDescriptor::new().expect("failed to create security descriptor");
        security_descriptor.set_dacl(acl).unwrap();

        set_file_security(
            file.path(),
            Security::DACL_SECURITY_INFORMATION,
            security_descriptor,
        )
        .expect("failed to set file security");
    }

    #[test]
    fn test_get_file_security() {
        let mut file = tempfile::NamedTempFile::new().expect("failed to create temp file");
        file.write_all(b"test").expect("failed to write to file");

        let security_descriptor =
            get_file_security(file.path(), Security::DACL_SECURITY_INFORMATION)
                .expect("failed to get file security");
        assert!(security_descriptor
            .is_relative()
            .expect("failed to check if security descriptor is relative"));
    }
}
