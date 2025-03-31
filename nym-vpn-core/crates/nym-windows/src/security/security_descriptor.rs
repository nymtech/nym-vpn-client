// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::marker::PhantomData;

use windows::{
    core::Result,
    Win32::{
        Foundation::{self, HLOCAL},
        Security::{
            self,
            Authorization::{GetExplicitEntriesFromAclW, EXPLICIT_ACCESS_W, TRUSTEE_W},
            GetSecurityDescriptorDacl, PSECURITY_DESCRIPTOR, PSID, SECURITY_DESCRIPTOR,
            SECURITY_DESCRIPTOR_CONTROL, SE_SELF_RELATIVE,
        },
        System::{Memory, SystemServices},
    },
};

use super::{trustee::TrusteeForm, AceFlags, Acl, Sid, TrusteeType};

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
    #[allow(dead_code)]
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
            Ok(Some(unsafe { Sid::copy_from(sid)? }))
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
            Ok(Some(unsafe { Sid::copy_from(sid)? }))
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

/// Struct that contains the security information associated with an object in a contiguous memory buffer.
#[derive(Debug)]
pub struct RelativeSecurityDescriptor<'a> {
    inner: PSECURITY_DESCRIPTOR,
    data: PhantomData<&'a SECURITY_DESCRIPTOR>,
}

impl<'a> RelativeSecurityDescriptor<'a> {
    unsafe fn from_ptr(ptr: PSECURITY_DESCRIPTOR) -> Self {
        Self {
            inner: ptr,
            data: PhantomData,
        }
    }

    fn get_acl(&self) -> Result<Option<BorrowedAcl<'a>>> {
        let mut lpbdaclpresent = Foundation::BOOL::default();
        let mut pdacl = std::ptr::null_mut();
        let mut lpbdacldefaulted = Foundation::BOOL::default();

        unsafe {
            GetSecurityDescriptorDacl(
                self.inner,
                &mut lpbdaclpresent,
                &mut pdacl,
                &mut lpbdacldefaulted,
            )?;
        }

        if lpbdaclpresent.as_bool() {
            Ok(Some(unsafe { BorrowedAcl::from_ptr(pdacl) }))
        } else {
            Ok(None)
        }
    }
}

/// tbd
pub struct BorrowedAcl<'a> {
    inner: *const Security::ACL,
    data: PhantomData<&'a Security::ACL>,
}

impl<'a> BorrowedAcl<'a> {
    unsafe fn from_ptr(ptr: *const Security::ACL) -> Self {
        Self {
            inner: ptr,
            data: PhantomData,
        }
    }

    /// TBD
    pub fn get_entries(&self) -> Result<Vec<BorrowedExplicitAccess>> {
        let mut num_entries = 0;

        // todo: call LocalFree() on this pointer!
        let mut pentries: *mut EXPLICIT_ACCESS_W = std::ptr::null_mut();

        unsafe { GetExplicitEntriesFromAclW(self.inner, &mut num_entries, &mut pentries).ok()? };

        let entries = (0..num_entries)
            .into_iter()
            .map(|i| {
                // Safety: cast to isize should be fine as number of entries is likely limited.
                let entry_ptr = unsafe { pentries.offset(i as isize) };
                let explicit_access = unsafe { BorrowedExplicitAccess::from_ptr(entry_ptr) };
                explicit_access
            })
            .collect::<Vec<_>>();

        Ok(entries)
    }
}

pub struct BorrowedExplicitAccess<'a> {
    inner: *const EXPLICIT_ACCESS_W,
    data: PhantomData<&'a EXPLICIT_ACCESS_W>,
}

impl<'a> BorrowedExplicitAccess<'a> {
    unsafe fn from_ptr(ptr: *const EXPLICIT_ACCESS_W) -> Self {
        Self {
            inner: ptr,
            data: PhantomData,
        }
    }

    pub fn get_access_permissions(&self) -> u32 {
        unsafe { (*self.inner).grfAccessPermissions }
    }

    pub fn get_inheritance(&self) -> AceFlags {
        let raw_flags = unsafe { (*self.inner).grfInheritance };
        AceFlags::from_bits_retain(raw_flags.0)
    }

    pub fn get_trustee(&self) -> BorrowedTrustee<'a> {
        unsafe { BorrowedTrustee::new(&(*self.inner).Trustee) }
    }
}

pub struct BorrowedTrustee<'a> {
    inner: &'a TRUSTEE_W,
}
impl<'a> BorrowedTrustee<'a> {
    unsafe fn new(trustee: &'a TRUSTEE_W) -> Self {
        Self { inner: trustee }
    }

    pub fn get_trustee_type(&self) -> TrusteeType {
        TrusteeType::from((*self.inner).TrusteeType)
    }

    pub fn get_trustee_form(&self) -> TrusteeForm {
        TrusteeForm::from((*self.inner).TrusteeForm)
    }

    pub fn get_trustee_name(&self) -> Result<TrusteeSpecificInfo> {
        // union {
        //     LPWSTR             ptstrName;
        //     SID                *pSid;
        //     OBJECTS_AND_SID    *pObjectsAndSid;
        //     OBJECTS_AND_NAME_W *pObjectsAndName;
        // };

        match self.get_trustee_form() {
            TrusteeForm::Name => {
                let name = unsafe { self.inner.ptstrName.to_hstring().to_string() };
                Ok(TrusteeSpecificInfo::Name(name))
            }
            TrusteeForm::Sid => {
                let psid = PSID(self.inner.ptstrName.0 as *mut _);
                let sid = unsafe { Sid::copy_from(psid)? };
                Ok(TrusteeSpecificInfo::Sid(sid))
            }
            TrusteeForm::ObjectsAndName => {
                todo!()
            }
            TrusteeForm::ObjectsAndSid => {
                todo!()
            }
        }
    }
}

pub enum TrusteeSpecificInfo {
    Name(String),
    Sid(Sid),
    ObjectsAndSid,
    ObjectsAndName,
}
