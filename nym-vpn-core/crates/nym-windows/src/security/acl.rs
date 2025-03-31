// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::{
    core::Result,
    Win32::{
        Foundation::{self, HLOCAL},
        Security::{
            self,
            Authorization::{GetExplicitEntriesFromAclW, EXPLICIT_ACCESS_W},
        },
    },
};

use super::ExplicitAccess;

/// Access control list.
#[derive(Debug)]
pub struct Acl {
    inner: *const Security::ACL,
    _entries: Vec<ExplicitAccess>,
}

impl Acl {
    /// Create new ACL with given entries.
    pub fn new(entries: Vec<ExplicitAccess>) -> Result<Self> {
        let mut inner: *mut Security::ACL = std::ptr::null_mut();
        let raw_entries = entries
            .iter()
            .map(|explicit_access| unsafe { explicit_access.inner() })
            .collect::<Vec<_>>();

        unsafe {
            Security::Authorization::SetEntriesInAclW(Some(&raw_entries), None, &mut inner).ok()?;
        }

        Ok(Self {
            inner,
            _entries: entries,
        })
    }

    /// TBD
    pub fn get_entries(&self) -> Result<Vec<ExplicitAccess>> {
        let mut num_entries = 0;

        // todo: call LocalFree() on this pointer!
        let mut pentries: *mut EXPLICIT_ACCESS_W = std::ptr::null_mut();

        unsafe { GetExplicitEntriesFromAclW(self.inner, &mut num_entries, &mut pentries).ok()? };

        for i in 0..num_entries {
            let offset = isize::try_from(i).expect("failed to convert to pointer offset");
            let entry = unsafe { pentries.offset(offset) };

            println!("{:?}", unsafe { *entry });
        }

        Ok(Vec::new())
    }

    /// Returns the inner pointer to `ACL`.
    ///
    /// # Safety
    /// The returned pointer is only guaranteed to remain valid during the lifetime of this struct.
    pub unsafe fn as_ptr(&self) -> *const Security::ACL {
        self.inner
    }
}

impl Drop for Acl {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            // SAFETY: pointer returned by SetEntriesInAclW is allocated with LocalAlloc
            unsafe { Foundation::LocalFree(Some(HLOCAL(self.inner as *mut _))) };
        }
    }
}
