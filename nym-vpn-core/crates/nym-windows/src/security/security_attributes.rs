// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::{
    core::Result,
    Win32::{Foundation::BOOL, Security::SECURITY_ATTRIBUTES},
};

use super::{
    AbsoluteSecurityDescriptor, AccessMode, AccessRights, AceFlags, Acl, ExplicitAccess, Sid,
    Trustee, TrusteeType, WellKnownSid,
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
    pub fn allow_everyone(permissions: AccessRights) -> Result<SecurityAttributes> {
        let trustee = Trustee::new(
            Sid::well_known(WellKnownSid::World)?,
            TrusteeType::WellKnownGroup,
        );

        let explicit_access = ExplicitAccess::new(
            trustee,
            AccessMode::SetAccess,
            permissions,
            AceFlags::NO_INHERITANCE,
        );

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
        SecurityAttributes::allow_everyone(permissions.into())
            .expect("failed to create security attributes that allow everyone everything");
    }
}
