// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::marker::PhantomData;

use windows::{
    core::Result,
    Win32::{
        Foundation::{LocalFree, BOOL, HLOCAL},
        Security::{
            GetSecurityDescriptorControl, GetSecurityDescriptorDacl, PSECURITY_DESCRIPTOR,
            SECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_CONTROL, SE_SELF_RELATIVE,
        },
    },
};

use super::BorrowedAcl;

/// Struct representing relative security descriptor.
///
/// Pointers held in the relative security descriptor reference the same contiguous block of memory.
#[derive(Debug)]
pub struct RelativeSecurityDescriptor<'a> {
    inner: PSECURITY_DESCRIPTOR,
    data: PhantomData<&'a SECURITY_DESCRIPTOR>,
}

impl<'a> RelativeSecurityDescriptor<'a> {
    /// Create new relative security descriptor.
    ///
    /// # Safety
    ///
    /// Fails with assertion if given non-relative security descriptor.
    pub unsafe fn from_ptr(ptr: PSECURITY_DESCRIPTOR) -> Self {
        assert!(is_relative_security_descriptor(ptr).unwrap());

        Self {
            inner: ptr,
            data: PhantomData,
        }
    }

    /// Returns ACL
    pub fn get_acl(&self) -> Result<Option<BorrowedAcl<'a>>> {
        let mut is_dacl_present = BOOL::default();
        let mut is_dacl_defaulted = BOOL::default();
        let mut pdacl = std::ptr::null_mut();

        unsafe {
            GetSecurityDescriptorDacl(
                self.inner,
                &mut is_dacl_present,
                &mut pdacl,
                &mut is_dacl_defaulted,
            )?;
        }

        if is_dacl_present.as_bool() {
            Ok(Some(unsafe { BorrowedAcl::from_ptr(pdacl) }))
        } else {
            Ok(None)
        }
    }
}

impl Drop for RelativeSecurityDescriptor<'_> {
    fn drop(&mut self) {
        if !self.inner.is_invalid() {
            unsafe { LocalFree(Some(HLOCAL(self.inner.0))) };
        }
    }
}

/// Returns true if security descriptor is in self-relative format.
///
/// More info in [documentation](https://docs.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-getsecuritydescriptorcontrol)
fn is_relative_security_descriptor(security_descriptor: PSECURITY_DESCRIPTOR) -> Result<bool> {
    let mut revision = 0;
    let mut control = SECURITY_DESCRIPTOR_CONTROL::default();

    unsafe {
        GetSecurityDescriptorControl(
            security_descriptor,
            // Safety: struct is transparent and holds u16.
            &mut control as *mut _ as *mut _,
            &mut revision,
        )?
    };

    Ok(control.contains(SE_SELF_RELATIVE))
}
