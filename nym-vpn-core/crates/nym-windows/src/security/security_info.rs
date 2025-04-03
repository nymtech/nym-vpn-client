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

/// Retrieve a copy of the security descriptor for an object specified by name.
///
/// Documentation: <https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-getnamedsecurityinfow>
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
            object_type.to_raw(),
            security_info.to_raw(),
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
        let data_dir = std::path::PathBuf::from("C:\\ProgramData\\test");

        let permissions = FileAccessRights::FILE_ALL_ACCESS;

        let local_system_sid = Sid::local_system().unwrap();
        let administrators_sid =
            Sid::new_well_known(WellKnownSid::WinBuiltinAdministratorsSid, None).unwrap();

        let ace_flags = AceFlags::OBJECT_INHERIT_ACE | AceFlags::CONTAINER_INHERIT_ACE;

        let local_system_trustee = Trustee::new(
            local_system_sid.clone().unwrap(),
            TrusteeType::WellKnownGroup,
        );
        let mut allow_local_system_access = ExplicitAccess::new(local_system_trustee);
        allow_local_system_access.set_access_mode(AccessMode::SetAccess);
        allow_local_system_access.set_access_permissions(permissions.bits());
        allow_local_system_access.set_inheritance(ace_flags);

        let administrators_trustee = Trustee::new(
            administrators_sid.clone().unwrap(),
            TrusteeType::WellKnownGroup,
        );
        let mut allow_admin_group_access = ExplicitAccess::new(administrators_trustee);
        allow_admin_group_access.set_access_mode(AccessMode::SetAccess);
        allow_admin_group_access.set_access_permissions(permissions.bits());
        allow_admin_group_access.set_inheritance(ace_flags);

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

        let security_descriptor = get_named_security_info(
            &data_dir,
            SecurityObjectType::FileObject,
            SecurityInfo::DACL,
        )
        .unwrap();

        let acl = security_descriptor.get_acl().unwrap().unwrap();
        let entry_list = acl.get_entries().unwrap();
        let entries = entry_list.as_vec();

        assert!(entries.len() == 2);

        assert_eq!(entries[0].get_access_permissions(), permissions.bits());
        assert_eq!(entries[0].get_inheritance(), ace_flags);
        assert!(matches!(
            entries[0]
                .get_trustee()
                .get_trustee_specific_info()
                .unwrap(),
            TrusteeSpecificInfo::Sid(sid) if sid == local_system_sid
        ));

        assert_eq!(entries[1].get_access_permissions(), permissions.bits());
        assert_eq!(entries[1].get_inheritance(), ace_flags);
        assert!(matches!(
            entries[1]
                .get_trustee()
                .get_trustee_specific_info()
                .unwrap(),
            TrusteeSpecificInfo::Sid(sid) if sid == administrators_sid
        ));
    }
}
