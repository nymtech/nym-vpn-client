// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::{
    core::Result,
    Win32::{
        Foundation::{self, HLOCAL},
        Security::{self, PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR},
        System::{Memory, SystemServices},
    },
};

use super::Acl;

/// Struct that contains the security information associated with an object.
#[derive(Debug)]
pub struct SecurityDescriptor {
    inner: PSECURITY_DESCRIPTOR,
    acl: Option<Acl>,
}

impl SecurityDescriptor {
    /// Initialize new security descriptor.
    pub fn new() -> Result<Self> {
        let buffer = unsafe {
            Memory::LocalAlloc(Memory::LPTR, std::mem::size_of::<SECURITY_DESCRIPTOR>())?
        };
        // SAFETY: The pointer has enough capacity to hold SECURITY_DESCRIPTOR.
        let inner = PSECURITY_DESCRIPTOR(buffer.0);
        unsafe {
            Security::InitializeSecurityDescriptor(
                inner,
                SystemServices::SECURITY_DESCRIPTOR_REVISION,
            )?
        };
        Ok(Self { inner, acl: None })
    }

    /// Set discretionary access control list
    pub fn set_dacl(&mut self, acl: Acl) -> Result<()> {
        // We must hold the ACL reference during the lifetime of the underlying `PSECURITY_DESCRIPTOR`
        // https://stackoverflow.com/questions/36549937/winapi-security-descriptor-with-size-security-descriptor-min-length-has-acl#comment60744624_36549937
        self.acl = Some(acl);

        unsafe {
            Security::SetSecurityDescriptorDacl(
                self.inner,
                // true indicates that dacl should be set.
                true,
                self.acl.as_ref().map(|v| v.as_ptr()),
                // false indicates that dacl is explicitly specified by user
                false,
            )
        }
    }

    /// Returns inner `PSECURITY_DESCRIPTOR`.
    ///
    /// # Safety
    /// The returned value stores raw pointers inside, which are only guaranteed to remain valid during the lifetime of this struct.
    pub unsafe fn inner(&self) -> PSECURITY_DESCRIPTOR {
        self.inner
    }
}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        if !self.inner.is_invalid() {
            unsafe { Foundation::LocalFree(Some(HLOCAL(self.inner.0))) };
        }
    }
}
