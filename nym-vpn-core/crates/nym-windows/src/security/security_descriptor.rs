// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::{
    core::Result,
    Win32::{
        Foundation::{self, HLOCAL},
        Security::{
            self, PSECURITY_DESCRIPTOR, PSID, SECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_CONTROL,
            SE_SELF_RELATIVE,
        },
        System::{Memory, SystemServices},
    },
};

use super::{Acl, Sid};

/// Struct that contains the security information associated with an object.
#[derive(Debug)]
pub struct SecurityDescriptor {
    inner: PSECURITY_DESCRIPTOR,
    owner: Option<Sid>,
    group: Option<Sid>,
    acl: Option<Acl>,
}

impl SecurityDescriptor {
    /// Initialize new security descriptor.
    pub fn new() -> Result<Self> {
        let buffer = unsafe {
            Memory::LocalAlloc(Memory::LPTR, std::mem::size_of::<SECURITY_DESCRIPTOR>())?
        };
        // Safety: The pointer has enough capacity to hold SECURITY_DESCRIPTOR.
        let inner = PSECURITY_DESCRIPTOR(buffer.0);
        unsafe {
            Security::InitializeSecurityDescriptor(
                inner,
                SystemServices::SECURITY_DESCRIPTOR_REVISION,
            )?
        };
        Ok(Self {
            inner,
            owner: None,
            group: None,
            acl: None,
        })
    }

    /// Initialize new security descriptor with given capacity.
    ///
    /// This method is used internally to create self-relative security descriptor stored within a contiguous memory block.
    ///
    /// Note that `capacity` less than `sizeof(SECURITY_DESCRIPTOR)` will be ignored and identical to calling `SecurityDescriptor::new()`.
    pub(crate) unsafe fn new_with_capacity(capacity: usize) -> Result<Self> {
        let min_capacity: usize = std::mem::size_of::<SECURITY_DESCRIPTOR>();
        let actual_capacity = std::cmp::max(min_capacity, capacity);

        let buffer = unsafe { Memory::LocalAlloc(Memory::LPTR, actual_capacity)? };
        // SAFETY: The pointer has enough capacity to hold SECURITY_DESCRIPTOR.
        let inner = PSECURITY_DESCRIPTOR(buffer.0);

        Ok(Self {
            inner,
            owner: None,
            group: None,
            acl: None,
        })
    }

    /// Returns true if security descriptor is in self-relative format.
    ///
    /// More info in [documentation](https://docs.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-getsecuritydescriptorcontrol)
    pub fn is_relative(&self) -> Result<bool> {
        let mut revision = 0;
        let mut control = SECURITY_DESCRIPTOR_CONTROL::default();

        unsafe {
            Security::GetSecurityDescriptorControl(
                self.inner,
                // Safety: struct is transparent and holds u16.
                &mut control as *mut _ as *mut _,
                &mut revision,
            )?
        };

        Ok(control.contains(SE_SELF_RELATIVE))
    }

    /// Set object owner replacing any owner information already present.
    ///
    /// Pass `None` to clear owner information leaving object without owner.
    pub fn set_owner(&mut self, owner: Option<Sid>) -> Result<()> {
        // We must hold the reference to owner during the lifetime of the underlying `PSECURITY_DESCRIPTOR`
        self.owner = owner;
        unsafe {
            Security::SetSecurityDescriptorOwner(
                self.inner,
                self.owner.as_ref().map(|sid| sid.inner()),
                false,
            )
        }
    }

    /// Get object owner.
    ///
    /// Returns `None` when owner is not set.
    pub fn get_owner(&self) -> Result<Option<Sid>> {
        let mut sid = PSID::default();
        let mut _owner_defaulted = windows::Win32::Foundation::BOOL::default();

        unsafe {
            Security::GetSecurityDescriptorOwner(self.inner, &mut sid, &mut _owner_defaulted)?
        };

        if sid.is_invalid() {
            Ok(None)
        } else {
            // Safety: make a copy of sid since `GetSecurityDescriptorOwner` returns a pointer to the internal buffer.
            Ok(Some(unsafe { Sid::new_with_copy(sid)? }))
        }
    }

    /// Set object group replacing any group information already present.
    ///
    /// Pass `None` to clear group information leaving object without group.
    pub fn set_group(&mut self, group: Option<Sid>) -> Result<()> {
        // We must hold the reference to group during the lifetime of the underlying `PSECURITY_DESCRIPTOR`
        self.group = group;
        unsafe {
            Security::SetSecurityDescriptorGroup(
                self.inner,
                self.group.as_ref().map(|sid| sid.inner()),
                false,
            )
        }
    }

    /// Get object group.
    ///
    /// Returns `None` when group is not set.
    pub fn get_group(&self) -> Result<Option<Sid>> {
        let mut sid = PSID::default();
        let mut _group_defaulted = windows::Win32::Foundation::BOOL::default();
        unsafe {
            Security::GetSecurityDescriptorGroup(self.inner, &mut sid, &mut _group_defaulted)?;
        }

        if sid.is_invalid() {
            Ok(None)
        } else {
            // Safety: make a copy of sid since `GetSecurityDescriptorGroup` returns a pointer to the internal buffer.
            Ok(Some(unsafe { Sid::new_with_copy(sid)? }))
        }
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
