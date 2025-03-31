// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::OsStr;

use windows::{
    core::{Result, HSTRING},
    Win32::Security::{
        Authorization::{
            GetNamedSecurityInfoW, SetNamedSecurityInfoW, SE_FILE_OBJECT, SE_OBJECT_TYPE,
            SE_SERVICE,
        },
        ATTRIBUTE_SECURITY_INFORMATION, DACL_SECURITY_INFORMATION, GROUP_SECURITY_INFORMATION,
        OBJECT_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION,
        PROTECTED_DACL_SECURITY_INFORMATION,
    },
};

use super::{Acl, Sid};

/// This struct is awesome
#[derive(Debug, Copy, Clone)]
pub enum SecurityObjectType {
    /// tbd
    FileObject,
    /// tbd
    Service,
}

impl SecurityObjectType {
    fn to_raw(&self) -> SE_OBJECT_TYPE {
        match self {
            Self::FileObject => SE_FILE_OBJECT,
            Self::Service => SE_SERVICE,
        }
    }
}

bitflags::bitflags! {
    /// tbd
    pub struct SecurityInfo: u32 {
        /// tbd
        const ATTRIBUTE = ATTRIBUTE_SECURITY_INFORMATION.0;
        /// The DACL of the object is being referenced.
        const DACL = DACL_SECURITY_INFORMATION.0;
        /// The DACL cannot inherit access control entries (ACEs).tbd
        const PROTECTED_DACL = PROTECTED_DACL_SECURITY_INFORMATION.0;
        /// tbd
        const GROUP = GROUP_SECURITY_INFORMATION.0;
        /// tbd
        const OWNER = OWNER_SECURITY_INFORMATION.0;
    }
}

impl SecurityInfo {
    fn to_raw(&self) -> OBJECT_SECURITY_INFORMATION {
        OBJECT_SECURITY_INFORMATION(self.bits())
    }
}

/// Set security information in the security descriptor of a specified object.
///
/// Documentation: <https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-setnamedsecurityinfow>
pub fn set_named_security_info<S>(
    object_name: S,
    object_type: SecurityObjectType,
    security_info: SecurityInfo,
    owner: Option<&Sid>,
    group: Option<&Sid>,
    dacl: Option<&Acl>,
) -> Result<()>
where
    S: AsRef<OsStr>,
{
    unsafe {
        SetNamedSecurityInfoW(
            &HSTRING::from(object_name.as_ref()),
            object_type.to_raw(),
            security_info.to_raw(),
            owner.as_ref().map(|x| x.inner()),
            group.as_ref().map(|x| x.inner()),
            dacl.as_ref().map(|x| x.as_ptr()),
            None,
        )
        .ok()
    }
}

pub fn get_security_info(
    object_name: S,
    object_type: SecurityObjectType,
    security_info: SecurityInfo,
    owner: Option<&Sid>,
    group: Option<&Sid>,
    dacl: Option<&Acl>,
) -> Result<()>
where
    S: AsRef<OsStr>,
{
    unsafe {
        GetNamedSecurityInfoW(
            &HSTRING::from(object_name.as_ref()),
            object_type.to_raw(),
            securityinfo,
            ppsidowner,
            ppsidgroup,
            ppdacl,
            ppsacl,
            ppsecuritydescriptor,
        )
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::{
        Security::WinBuiltinAdministratorsSid, Storage::FileSystem::FILE_ALL_ACCESS,
    };

    use super::*;
    use crate::security::{
        explicit_access::{AccessMode, AceFlags},
        Acl, ExplicitAccess, Sid, Trustee, TrusteeType,
    };

    #[test]
    fn test_set_named_security() {
        let data_dir = std::path::PathBuf::from("C:\\ProgramData\\test");

        let permissions = FILE_ALL_ACCESS;

        let local_system_trustee =
            Trustee::new(Sid::local_system().unwrap(), TrusteeType::WellKnownGroup);
        let mut allow_local_system_access = ExplicitAccess::new(local_system_trustee);
        allow_local_system_access.set_access_mode(AccessMode::SetAccess);
        allow_local_system_access.set_access_permissions(permissions.0);
        allow_local_system_access.set_inheritance(AceFlags::NO_INHERITANCE);

        let administrators_trustee = Trustee::new(
            Sid::new_well_known(WinBuiltinAdministratorsSid, None).unwrap(),
            TrusteeType::WellKnownGroup,
        );
        let mut allow_admin_group_access = ExplicitAccess::new(administrators_trustee);
        allow_admin_group_access.set_access_mode(AccessMode::SetAccess);
        allow_admin_group_access.set_access_permissions(permissions.0);
        allow_admin_group_access.set_inheritance(AceFlags::NO_INHERITANCE);

        let acl = Acl::new(vec![allow_local_system_access, allow_admin_group_access]).unwrap();

        set_named_security_info(
            &data_dir,
            SecurityObjectType::FileObject,
            SecurityInfo::DACL | SecurityInfo::PROTECTED_DACL,
            None,
            None,
            Some(&acl),
        )
        .unwrap();
    }
}
