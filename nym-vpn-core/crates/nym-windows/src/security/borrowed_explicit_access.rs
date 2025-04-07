// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::marker::PhantomData;

use windows::Win32::Security::Authorization::EXPLICIT_ACCESS_W;

use super::{AceFlags, BorrowedTrustee};

/// Borrowed version of `ExplicitAccess`
#[derive(Debug)]
pub struct BorrowedExplicitAccess<'a> {
    inner: *const EXPLICIT_ACCESS_W,
    data: PhantomData<&'a EXPLICIT_ACCESS_W>,
}

impl<'a> BorrowedExplicitAccess<'a> {
    /// Create new instance from pointer, without taking ownership of memory.
    ///
    /// # Safety
    /// The caller must ensure the validity of pointer during the lifetime of this struct.
    pub unsafe fn from_ptr(ptr: *const EXPLICIT_ACCESS_W) -> Self {
        Self {
            inner: ptr,
            data: PhantomData,
        }
    }

    /// Returns access permissions
    pub fn get_access_permissions(&self) -> u32 {
        unsafe { (*self.inner).grfAccessPermissions }
    }

    /// Returns inheritance flags.
    pub fn get_inheritance(&self) -> AceFlags {
        let raw_flags = unsafe { (*self.inner).grfInheritance };
        AceFlags::from_bits_retain(raw_flags.0)
    }

    /// Returns trustee data.
    pub fn get_trustee(&self) -> BorrowedTrustee<'a> {
        unsafe { BorrowedTrustee::new(&(*self.inner).Trustee) }
    }
}
