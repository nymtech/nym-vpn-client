// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::Win32::Security::{
    Authorization::{
        ACCESS_MODE, DENY_ACCESS, EXPLICIT_ACCESS_W, GRANT_ACCESS, NOT_USED_ACCESS, REVOKE_ACCESS,
        SET_ACCESS, SET_AUDIT_FAILURE, SET_AUDIT_SUCCESS,
    },
    ACE_FLAGS, CONTAINER_INHERIT_ACE, INHERIT_NO_PROPAGATE, INHERIT_ONLY, INHERIT_ONLY_ACE,
    NO_INHERITANCE, NO_PROPAGATE_INHERIT_ACE, OBJECT_INHERIT_ACE,
    SUB_CONTAINERS_AND_OBJECTS_INHERIT, SUB_CONTAINERS_ONLY_INHERIT, SUB_OBJECTS_ONLY_INHERIT,
};

use super::{AccessRights, Trustee};

/// Access control information for a specified trustee.
///
/// For more information see: https://learn.microsoft.com/en-us/windows/win32/api/accctrl/ns-accctrl-explicit_access_w
#[derive(Debug)]
pub struct ExplicitAccess {
    inner: EXPLICIT_ACCESS_W,
    _trustee: Trustee,
}

impl ExplicitAccess {
    /// Create a new `ExplicitAccess` struct filling in all of the information at once.
    pub fn new(
        trustee: Trustee,
        access_mode: AccessMode,
        access_permissions: AccessRights,
        inheritance_flags: AceFlags,
    ) -> Self {
        let inner = EXPLICIT_ACCESS_W {
            Trustee: unsafe { trustee.inner() },
            grfAccessMode: access_mode.into(),
            grfAccessPermissions: access_permissions.bits(),
            grfInheritance: ACE_FLAGS(inheritance_flags.bits()),
        };
        Self {
            inner,
            _trustee: trustee,
        }
    }

    /// Set access mode.
    ///
    /// For a discretionary access control list (DACL), this flag indicates whether the ACL allows or denies the specified access rights.
    /// For a system access control list (SACL), this flag indicates whether the ACL generates audit messages for successful attempts to use the specified access rights, or failed attempts, or both.
    ///
    /// For more information, see [`ACCESS_MODE`](https://learn.microsoft.com/en-us/windows/win32/api/accctrl/ne-accctrl-access_mode)
    pub fn set_access_mode(&mut self, access_mode: AccessMode) {
        self.inner.grfAccessMode = access_mode.into();
    }

    /// Set access permissions.
    ///
    /// Use values defined by `FileAccessRights`, `GenericAccessRights, `StandardAccessRights`.
    pub fn set_access_permissions(&mut self, permissions: AccessRights) {
        self.inner.grfAccessPermissions = permissions.bits();
    }

    /// Set bit flags that determines whether other containers or objects can inherit the ACE from the primary object to which the ACL is attached.
    pub fn set_inheritance(&mut self, inheritance_flags: AceFlags) {
        self.inner.grfInheritance = ACE_FLAGS(inheritance_flags.bits());
    }

    /// Returns the inner `EXPLICIT_ACCESS_W`.
    ///
    /// # Safety
    /// The returned value stores raw pointers inside, which are only guaranteed to remain valid during the lifetime of this struct.
    pub unsafe fn inner(&self) -> EXPLICIT_ACCESS_W {
        self.inner
    }
}

/// Access mode describing how access permissions should be applied
pub enum AccessMode {
    /// Value not used.
    NotUsedAccess,
    /// The new ACE combines the specified rights with any existing allowed or denied rights of the trustee.
    GrantAccess,
    /// Discard any existing access control information for the trustee.
    SetAccess,
    /// Denies the specified rights in addition to any currently denied rights of the trustee.
    DenyAccess,
    /// Indicates that all existing `ACCESS_ALLOWED_ACE` or `SYSTEM_AUDIT_ACE` structures for the specified trustee are removed.
    RevokeAccess,
    SetAuditSuccess,
    SetAuditFailure,
}

impl From<AccessMode> for ACCESS_MODE {
    fn from(mode: AccessMode) -> ACCESS_MODE {
        match mode {
            AccessMode::NotUsedAccess => NOT_USED_ACCESS,
            AccessMode::GrantAccess => GRANT_ACCESS,
            AccessMode::SetAccess => SET_ACCESS,
            AccessMode::DenyAccess => DENY_ACCESS,
            AccessMode::RevokeAccess => REVOKE_ACCESS,
            AccessMode::SetAuditSuccess => SET_AUDIT_SUCCESS,
            AccessMode::SetAuditFailure => SET_AUDIT_FAILURE,
        }
    }
}

bitflags::bitflags! {
    /// ACE inheritance flags.
    /// See: <https://learn.microsoft.com/en-us/windows/win32/api/accctrl/ns-accctrl-explicit_access_w>
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct AceFlags: u32 {
        const CONTAINER_INHERIT_ACE = CONTAINER_INHERIT_ACE.0;
        const INHERIT_NO_PROPAGATE = INHERIT_NO_PROPAGATE.0;
        const INHERIT_ONLY = INHERIT_ONLY.0;
        const INHERIT_ONLY_ACE = INHERIT_ONLY_ACE.0;
        const NO_INHERITANCE = NO_INHERITANCE.0;
        const NO_PROPAGATE_INHERIT_ACE = NO_PROPAGATE_INHERIT_ACE.0;
        const OBJECT_INHERIT_ACE = OBJECT_INHERIT_ACE.0;
        const SUB_CONTAINERS_AND_OBJECTS_INHERIT = SUB_CONTAINERS_AND_OBJECTS_INHERIT.0;
        const SUB_CONTAINERS_ONLY_INHERIT = SUB_CONTAINERS_ONLY_INHERIT.0;
        const SUB_OBJECTS_ONLY_INHERIT = SUB_OBJECTS_ONLY_INHERIT.0;
    }

}
