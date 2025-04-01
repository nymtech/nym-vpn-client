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

use super::Trustee;

/// Access control information for a specified trustee.
///
/// For more information see: https://learn.microsoft.com/en-us/windows/win32/api/accctrl/ns-accctrl-explicit_access_w
#[derive(Debug)]
pub struct ExplicitAccess {
    inner: EXPLICIT_ACCESS_W,
    _trustee: Trustee,
}

impl ExplicitAccess {
    /// Create a new `ExplicitAccess` struct.
    pub fn new(trustee: Trustee) -> Self {
        let inner = EXPLICIT_ACCESS_W {
            Trustee: unsafe { trustee.inner() },
            ..Default::default()
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
        self.inner.grfAccessMode = access_mode.to_raw();
    }

    /// Set access permissions.
    ///
    /// Permissions mask is expected to contain any of the values listed under [`ACCESS_MASK`](https://learn.microsoft.com/en-us/windows/win32/secauthz/access-mask)
    pub fn set_access_permissions(&mut self, permissions: u32) {
        self.inner.grfAccessPermissions = permissions;
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
    /// tbd
    SetAuditSuccess,
    /// tbd
    SetAuditFailure,
}

impl AccessMode {
    fn to_raw(&self) -> ACCESS_MODE {
        match self {
            Self::NotUsedAccess => NOT_USED_ACCESS,
            Self::GrantAccess => GRANT_ACCESS,
            Self::SetAccess => SET_ACCESS,
            Self::DenyAccess => DENY_ACCESS,
            Self::RevokeAccess => REVOKE_ACCESS,
            Self::SetAuditSuccess => SET_AUDIT_SUCCESS,
            Self::SetAuditFailure => SET_AUDIT_FAILURE,
        }
    }
}

bitflags::bitflags! {
    /// ACE inheritance flags.
    /// See: <https://learn.microsoft.com/en-us/windows/win32/api/accctrl/ns-accctrl-explicit_access_a>
    #[derive(Debug)]
    pub struct AceFlags: u32 {
        /// tbd
        const CONTAINER_INHERIT_ACE = CONTAINER_INHERIT_ACE.0;
        /// tbd
        const INHERIT_NO_PROPAGATE = INHERIT_NO_PROPAGATE.0;
        /// tbd
        const INHERIT_ONLY = INHERIT_ONLY.0;
        /// tbd
        const INHERIT_ONLY_ACE = INHERIT_ONLY_ACE.0;
        /// tbd
        const NO_INHERITANCE = NO_INHERITANCE.0;
        /// tbd
        const NO_PROPAGATE_INHERIT_ACE = NO_PROPAGATE_INHERIT_ACE.0;
        /// tbd
        const OBJECT_INHERIT_ACE = OBJECT_INHERIT_ACE.0;
        /// tbd
        const SUB_CONTAINERS_AND_OBJECTS_INHERIT = SUB_CONTAINERS_AND_OBJECTS_INHERIT.0;
        /// tbd
        const SUB_CONTAINERS_ONLY_INHERIT = SUB_CONTAINERS_ONLY_INHERIT.0;
        /// tbd
        const SUB_OBJECTS_ONLY_INHERIT = SUB_OBJECTS_ONLY_INHERIT.0;
    }

}
