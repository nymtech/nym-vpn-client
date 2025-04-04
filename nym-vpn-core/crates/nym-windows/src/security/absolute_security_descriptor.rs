// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::{
    core::Result,
    Win32::{
        Foundation::{LocalFree, HLOCAL},
        Security::{
            InitializeSecurityDescriptor, SetSecurityDescriptorDacl, SetSecurityDescriptorGroup,
            SetSecurityDescriptorOwner, PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR,
        },
        System::{
            Memory::{LocalAlloc, LPTR},
            SystemServices::SECURITY_DESCRIPTOR_REVISION,
        },
    },
};

use super::{Acl, Sid};

/// Struct representing absolute security descriptor.
///
/// Pointers held in the absolute security descriptor reference difference blocks of memory.
#[derive(Debug)]
pub struct AbsoluteSecurityDescriptor {
    inner: PSECURITY_DESCRIPTOR,
    owner: Option<Sid>,
    group: Option<Sid>,
    acl: Option<Acl>,
}

impl AbsoluteSecurityDescriptor {
    /// Initialize new security descriptor.
    pub fn new() -> Result<Self> {
        let buffer = unsafe { LocalAlloc(LPTR, std::mem::size_of::<SECURITY_DESCRIPTOR>())? };
        let inner = PSECURITY_DESCRIPTOR(buffer.0);
        unsafe { InitializeSecurityDescriptor(inner, SECURITY_DESCRIPTOR_REVISION)? };
        Ok(Self {
            inner,
            owner: None,
            group: None,
            acl: None,
        })
    }

    /// Set object owner replacing any owner information already present.
    ///
    /// Pass `None` to clear owner information leaving object without owner.
    pub fn set_owner(&mut self, owner: Option<Sid>) -> Result<()> {
        // We must hold the reference to owner during the lifetime of the underlying `PSECURITY_DESCRIPTOR`
        self.owner = owner;
        unsafe {
            SetSecurityDescriptorOwner(
                self.inner,
                self.owner.as_ref().map(|sid| sid.inner()),
                false,
            )
        }
    }

    /// Set object group replacing any group information already present.
    ///
    /// Pass `None` to clear group information leaving object without group.
    pub fn set_group(&mut self, group: Option<Sid>) -> Result<()> {
        // We must hold the reference to group during the lifetime of the underlying `PSECURITY_DESCRIPTOR`
        self.group = group;
        unsafe {
            SetSecurityDescriptorGroup(
                self.inner,
                self.group.as_ref().map(|sid| sid.inner()),
                false,
            )
        }
    }

    /// Set discretionary access control list
    pub fn set_dacl(&mut self, acl: Acl) -> Result<()> {
        // We must hold the ACL reference during the lifetime of the underlying `PSECURITY_DESCRIPTOR`
        // https://stackoverflow.com/questions/36549937/winapi-security-descriptor-with-size-security-descriptor-min-length-has-acl#comment60744624_36549937
        self.acl = Some(acl);

        unsafe {
            SetSecurityDescriptorDacl(
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

impl Drop for AbsoluteSecurityDescriptor {
    fn drop(&mut self) {
        if !self.inner.is_invalid() {
            unsafe { LocalFree(Some(HLOCAL(self.inner.0))) };
        }
    }
}
