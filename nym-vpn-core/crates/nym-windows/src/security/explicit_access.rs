// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::Win32::Security::{
    Authorization::{ACCESS_MODE, EXPLICIT_ACCESS_W},
    ACE_FLAGS,
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
    pub fn set_access_mode(&mut self, access_mode: ACCESS_MODE) {
        self.inner.grfAccessMode = access_mode;
    }

    /// Set access permissions
    ///
    /// Permissions mask is expected to contain any of the values listed under [`ACCESS_MASK`](https://learn.microsoft.com/en-us/windows/win32/secauthz/access-mask)
    pub fn set_access_permissions(&mut self, permissions: u32) {
        self.inner.grfAccessPermissions = permissions;
    }

    /// Set bit flags that determines whether other containers or objects can inherit the ACE from the primary object to which the ACL is attached.
    pub fn set_inheritance(&mut self, inheritance_flags: ACE_FLAGS) {
        self.inner.grfInheritance = inheritance_flags;
    }

    /// Returns the inner `EXPLICIT_ACCESS_W`.
    ///
    /// # Safety
    /// The returned value stores raw pointers inside, which are only guaranteed to remain valid during the lifetime of this struct.
    pub unsafe fn inner(&self) -> EXPLICIT_ACCESS_W {
        self.inner
    }
}
