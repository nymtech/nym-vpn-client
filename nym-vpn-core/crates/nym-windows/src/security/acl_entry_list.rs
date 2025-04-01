// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::marker::PhantomData;

use windows::Win32::{
    Foundation::{LocalFree, HLOCAL},
    Security::Authorization::EXPLICIT_ACCESS_W,
};

use super::BorrowedExplicitAccess;

/// tbd
#[derive(Debug)]
pub struct AclEntryList<'a> {
    entries: *mut EXPLICIT_ACCESS_W,
    num_entries: u32,
    data: PhantomData<&'a EXPLICIT_ACCESS_W>,
}

impl<'a> Drop for AclEntryList<'a> {
    fn drop(&mut self) {
        unsafe { LocalFree(Some(HLOCAL(self.entries as *mut _))) };
    }
}

impl<'a> AclEntryList<'a> {
    /// tbd
    pub(crate) unsafe fn from_ptr(entries: *mut EXPLICIT_ACCESS_W, num_entries: u32) -> Self {
        Self {
            entries,
            num_entries,
            data: PhantomData,
        }
    }

    /// tbd
    pub fn as_vec(&self) -> Vec<BorrowedExplicitAccess> {
        (0..self.num_entries)
            .into_iter()
            .map(|i| {
                // Safety: cast to isize should be fine as number of entries is likely limited.
                let entry_ptr = unsafe { self.entries.offset(i as isize) };
                let explicit_access = unsafe { BorrowedExplicitAccess::from_ptr(entry_ptr) };
                explicit_access
            })
            .collect::<Vec<_>>()
    }
}
