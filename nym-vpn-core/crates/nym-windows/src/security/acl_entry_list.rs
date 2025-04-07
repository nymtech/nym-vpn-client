// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::marker::PhantomData;

use windows::Win32::{
    Foundation::{LocalFree, HLOCAL},
    Security::Authorization::EXPLICIT_ACCESS_W,
};

use super::BorrowedExplicitAccess;

/// Container type holding a list of `EXPLICIT_ACCESS_W` structs.
#[derive(Debug)]
pub struct AclEntryList<'a> {
    entries: *mut EXPLICIT_ACCESS_W,
    num_entries: u32,
    data: PhantomData<&'a EXPLICIT_ACCESS_W>,
}

impl Drop for AclEntryList<'_> {
    fn drop(&mut self) {
        unsafe { LocalFree(Some(HLOCAL(self.entries as *mut _))) };
    }
}

impl AclEntryList<'_> {
    /// Take owneship of an array of `EXPLICIT_ACCESS_W` structs.
    pub(crate) unsafe fn new(entries: *mut EXPLICIT_ACCESS_W, num_entries: u32) -> Self {
        Self {
            entries,
            num_entries,
            data: PhantomData,
        }
    }

    /// Return a view into explicit access structs.
    pub fn as_vec(&self) -> Vec<BorrowedExplicitAccess> {
        (0..self.num_entries)
            .map(|i| {
                // Safety: cast to isize should be fine as number of entries is likely limited.
                let entry_ptr = unsafe { self.entries.offset(i as isize) };
                let explicit_access = unsafe { BorrowedExplicitAccess::from_ptr(entry_ptr) };
                explicit_access
            })
            .collect::<Vec<_>>()
    }
}
