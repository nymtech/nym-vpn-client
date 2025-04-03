// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::{
    core::Result,
    Win32::{Foundation::BOOL, Security::SECURITY_ATTRIBUTES},
};

use super::{
    AbsoluteSecurityDescriptor, AccessMode, AceFlags, Acl, ExplicitAccess, Sid, Trustee,
    TrusteeType,
};

/// Struct that contains the security identifier for an object and specifies whether the handle retrieved by specifying this struct is inheritable.
#[derive(Debug)]
pub struct SecurityAttributes {
    inner: SECURITY_ATTRIBUTES,
    _security_descriptor: AbsoluteSecurityDescriptor,
}

unsafe impl Send for SecurityAttributes {}

impl SecurityAttributes {
    /// Create new security attributes with security descriptor.
    pub fn new(security_descriptor: AbsoluteSecurityDescriptor) -> Self {
        Self {
            inner: SECURITY_ATTRIBUTES {
                bInheritHandle: BOOL::from(false),
                lpSecurityDescriptor: unsafe { security_descriptor.inner().0 as _ },
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            },
            _security_descriptor: security_descriptor,
        }
    }

    /// Create new security attributes with permissions for everyone.
    ///
    /// Permissions mask is expected to contain any of the values listed under [`ACCESS_MASK`](https://learn.microsoft.com/en-us/windows/win32/secauthz/access-mask)
    /// Use values defined by `FileAccessRights`, `GenericAccessRights`, `StandardAccessRights`.
    pub fn allow_everyone(permissions: u32) -> Result<SecurityAttributes> {
        let trustee = Trustee::new(Sid::everyone()?, TrusteeType::WellKnownGroup);

        let mut explicit_access = ExplicitAccess::new(trustee);
        explicit_access.set_access_mode(AccessMode::SetAccess);
        explicit_access.set_access_permissions(permissions);
        explicit_access.set_inheritance(AceFlags::NO_INHERITANCE);

        let acl = Acl::new(vec![explicit_access])?;
        let mut security_descriptor = AbsoluteSecurityDescriptor::new()?;
        security_descriptor.set_dacl(acl)?;

        Ok(SecurityAttributes::new(security_descriptor))
    }

    /// Returns a mutable pointer to the underlying `SECURITY_ATTRIBUTES` struct.
    ///
    /// # Safety
    /// The returned pointer is guaranteed to remain valid during the lifetime of this struct.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut SECURITY_ATTRIBUTES {
        &mut self.inner
    }
}

#[cfg(test)]
mod test {
    use super::SecurityAttributes;
    use crate::security::GenericAccessRights;

    #[test]
    fn test_allow_everyone_everything() {
        let permissions = GenericAccessRights::GENERIC_READ | GenericAccessRights::GENERIC_WRITE;
        SecurityAttributes::allow_everyone(permissions.bits())
            .expect("failed to create security attributes that allow everyone everything");
    }
}
