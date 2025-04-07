// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::marker::PhantomData;

use windows::{
    core::Result,
    Win32::Security::{
        Authorization::{GetExplicitEntriesFromAclW, EXPLICIT_ACCESS_W},
        ACL,
    },
};

use super::AclEntryList;

/// Borrowed version of `Acl`
#[derive(Debug)]
pub struct BorrowedAcl<'a> {
    inner: *const ACL,
    data: PhantomData<&'a ACL>,
}

impl BorrowedAcl<'_> {
    /// Create new instance from pointer without taking ownership of memory.
    ///
    /// # Safety
    /// The caller must ensure the validity of pointer during the lifetime of this struct.
    pub unsafe fn from_ptr(ptr: *const ACL) -> Self {
        Self {
            inner: ptr,
            data: PhantomData,
        }
    }

    /// Get ACL entries.
    pub fn get_entries(&self) -> Result<AclEntryList> {
        let mut num_entries = 0;
        let mut entries: *mut EXPLICIT_ACCESS_W = std::ptr::null_mut();

        unsafe { GetExplicitEntriesFromAclW(self.inner, &mut num_entries, &mut entries).ok()? };

        Ok(unsafe { AclEntryList::new(entries, num_entries) })
    }
}
