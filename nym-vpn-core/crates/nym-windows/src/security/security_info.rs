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
        PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
    },
};

use super::{Acl, RelativeSecurityDescriptor, Sid};

/// Types of Windows objects that support security
#[derive(Debug, Copy, Clone)]
pub enum SecurityObjectType {
    /// Indicates a file or directory
    FileObject,
    /// Indicates a Windows service. A service object can be a local service, such as ServiceName, or a remote service, such as \\ComputerName\ServiceName.
    Service,
}

impl From<SecurityObjectType> for SE_OBJECT_TYPE {
    fn from(value: SecurityObjectType) -> SE_OBJECT_TYPE {
        match value {
            SecurityObjectType::FileObject => SE_FILE_OBJECT,
            SecurityObjectType::Service => SE_SERVICE,
        }
    }
}

bitflags::bitflags! {
    pub struct SecurityInfo: u32 {
        const ATTRIBUTE = ATTRIBUTE_SECURITY_INFORMATION.0;
        /// The DACL of the object is being referenced.
        const DACL = DACL_SECURITY_INFORMATION.0;
        /// The DACL cannot inherit access control entries (ACEs).
        const PROTECTED_DACL = PROTECTED_DACL_SECURITY_INFORMATION.0;
        const GROUP = GROUP_SECURITY_INFORMATION.0;
        const OWNER = OWNER_SECURITY_INFORMATION.0;
    }
}

impl From<SecurityInfo> for OBJECT_SECURITY_INFORMATION {
    fn from(value: SecurityInfo) -> OBJECT_SECURITY_INFORMATION {
        OBJECT_SECURITY_INFORMATION(value.bits())
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
            object_type.into(),
            security_info.into(),
            owner.as_ref().map(|x| x.inner()),
            group.as_ref().map(|x| x.inner()),
            dacl.as_ref().map(|x| x.as_ptr()),
            None,
        )
        .ok()
    }
}

/// Retrieve a copy of the security descriptor for an object specified by name.
///
/// Documentation: <https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-getnamedsecurityinfow>
#[allow(dead_code)]
pub fn get_named_security_info<'a, S>(
    object_name: S,
    object_type: SecurityObjectType,
    security_info: SecurityInfo,
) -> Result<RelativeSecurityDescriptor<'a>>
where
    S: AsRef<OsStr>,
{
    let mut sid_owner = PSID::default();
    let mut sid_group = PSID::default();
    let mut dacl = std::ptr::null_mut();
    let mut sacl: *mut windows::Win32::Security::ACL = std::ptr::null_mut();
    let mut security_descriptor = PSECURITY_DESCRIPTOR::default();

    unsafe {
        GetNamedSecurityInfoW(
            &HSTRING::from(object_name.as_ref()),
            object_type.into(),
            security_info.into(),
            Some(&mut sid_owner as _),
            Some(&mut sid_group as _),
            Some(&mut dacl as _),
            Some(&mut sacl as _),
            &mut security_descriptor,
        )
        .ok()?;
    }

    assert!(!security_descriptor.is_invalid());
    Ok(unsafe { RelativeSecurityDescriptor::from_ptr(security_descriptor) })
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::security::{
        explicit_access::{AccessMode, AceFlags},
        Acl, ExplicitAccess, FileAccessRights, Sid, Trustee, TrusteeSpecificInfo, TrusteeType,
        WellKnownSid,
    };

    #[test]
    fn test_set_named_security() {
        let temp_dir = tempfile::tempdir().unwrap();
        let data_dir = temp_dir.path();

        let local_system_sid = Sid::well_known(WellKnownSid::LocalSystem).unwrap();
        let users_sid = Sid::well_known(WellKnownSid::BuiltinUsers).unwrap();

        let local_system_trustee =
            Trustee::new(local_system_sid.try_clone().unwrap(), TrusteeType::User);
        let allow_local_system_access = ExplicitAccess::new(
            local_system_trustee,
            AccessMode::SetAccess,
            FileAccessRights::FILE_ALL_ACCESS.into(),
            AceFlags::OBJECT_INHERIT_ACE | AceFlags::CONTAINER_INHERIT_ACE,
        );

        let allow_users_access = ExplicitAccess::new(
            Trustee::new(users_sid.try_clone().unwrap(), TrusteeType::WellKnownGroup),
            AccessMode::SetAccess,
            FileAccessRights::FILE_ALL_ACCESS.into(),
            AceFlags::OBJECT_INHERIT_ACE | AceFlags::CONTAINER_INHERIT_ACE,
        );

        let acl = Acl::new(vec![allow_local_system_access, allow_users_access]).unwrap();

        set_named_security_info(
            data_dir,
            SecurityObjectType::FileObject,
            SecurityInfo::DACL | SecurityInfo::PROTECTED_DACL,
            None,
            None,
            Some(&acl),
        )
        .unwrap();

        let security_descriptor =
            get_named_security_info(data_dir, SecurityObjectType::FileObject, SecurityInfo::DACL)
                .unwrap();

        let acl = security_descriptor.get_acl().unwrap().unwrap();
        let entry_list = acl.get_entries().unwrap();
        let entries = entry_list.as_vec();

        assert_eq!(entries.len(), 2);

        assert_eq!(
            entries[0].get_access_permissions(),
            FileAccessRights::FILE_ALL_ACCESS.bits()
        );
        assert_eq!(
            entries[0].get_inheritance(),
            AceFlags::OBJECT_INHERIT_ACE | AceFlags::CONTAINER_INHERIT_ACE
        );
        assert!(matches!(
            entries[0]
                .get_trustee()
                .get_trustee_specific_info()
                .unwrap(),
            TrusteeSpecificInfo::Sid(sid) if sid == local_system_sid
        ));

        assert_eq!(
            entries[1].get_access_permissions(),
            FileAccessRights::FILE_ALL_ACCESS.bits()
        );
        assert_eq!(
            entries[1].get_inheritance(),
            AceFlags::OBJECT_INHERIT_ACE | AceFlags::CONTAINER_INHERIT_ACE
        );
        assert!(matches!(
            entries[1]
                .get_trustee()
                .get_trustee_specific_info()
                .unwrap(),
            TrusteeSpecificInfo::Sid(sid) if sid == users_sid
        ));
    }
}
