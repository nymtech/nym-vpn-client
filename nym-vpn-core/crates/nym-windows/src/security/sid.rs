// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::{
    core::{Result, PWSTR},
    Win32::{
        Foundation::{self, HLOCAL},
        Security::{self, PSID},
        System::SystemServices,
    },
};

/// Struct that uniquely identifies users or groups.
#[derive(Debug)]
pub struct Sid {
    inner: PSID,
}

impl Sid {
    /// Returns a SID that corresponds to everyone on the machine.
    pub fn everyone() -> Result<Self> {
        let mut inner = PSID::default();
        unsafe {
            Security::AllocateAndInitializeSid(
                &Security::SECURITY_WORLD_SID_AUTHORITY,
                1,
                SystemServices::SECURITY_WORLD_RID as u32,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                &mut inner as _,
            )?;
        }
        Ok(Self { inner })
    }

    /// Convert SID to string.
    pub fn to_string(&self) -> Result<String> {
        let mut wide_str = PWSTR::null();
        unsafe { Security::Authorization::ConvertSidToStringSidW(self.inner, &mut wide_str as _)? };
        let result = unsafe { wide_str.to_string()? };
        if !wide_str.is_null() {
            unsafe { Foundation::LocalFree(Some(HLOCAL(wide_str.0 as *mut _))) };
        }

        Ok(result)
    }

    /// Returns the inner `PSID`.
    ///
    /// # Safety
    /// The returned value stores raw pointers inside, which are only guaranteed to remain valid during the lifetime of this struct.
    pub unsafe fn inner(&self) -> PSID {
        self.inner
    }
}

impl Drop for Sid {
    fn drop(&mut self) {
        if !self.inner.is_invalid() {
            unsafe { Security::FreeSid(self.inner) };
        }
    }
}
