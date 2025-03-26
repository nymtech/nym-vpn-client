// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::{
    core::{Result, HRESULT, PWSTR},
    Win32::{
        Foundation::{self, LocalFree, ERROR_INSUFFICIENT_BUFFER, HLOCAL},
        Security::{
            self, CreateWellKnownSid, GetTokenInformation, IsWellKnownSid, LookupAccountSidW,
            TokenUser, PSID, SECURITY_MAX_SID_SIZE, SID_NAME_USE, TOKEN_QUERY, TOKEN_USER,
            WELL_KNOWN_SID_TYPE,
        },
        System::{
            Memory::{self, LocalAlloc},
            SystemServices,
            Threading::{GetCurrentProcess, OpenProcessToken},
        },
    },
};

/// Struct that uniquely identifies users or groups.
#[derive(Debug, Eq)]
pub struct Sid {
    inner: PSID,
}

impl Sid {
    /// Create new SID from raw pointer.
    pub(crate) unsafe fn new_with_copy(psid: PSID) -> Result<Self> {
        let sid_len = Security::GetLengthSid(psid);
        let sid_len_sz = usize::try_from(sid_len).expect("sid length is too large");
        let buffer = Memory::LocalAlloc(Memory::LPTR, sid_len_sz)?;
        let dest_sid = PSID(buffer.0 as *mut _);

        unsafe { Security::CopySid(sid_len, dest_sid, psid)? };

        Ok(Self { inner: dest_sid })
    }

    /// Create new well known SID.
    pub fn new_well_known(sid_type: WELL_KNOWN_SID_TYPE) -> Result<Self> {
        let mut cbsize = SECURITY_MAX_SID_SIZE;
        let empty_sid = Self::empty()?;

        unsafe { CreateWellKnownSid(sid_type, None, Some(empty_sid.inner), &mut cbsize)? };

        Ok(empty_sid)
    }

    /// Create new empty SID allocating enough memory to fit any kind of SID.
    fn empty() -> Result<Self> {
        let len = usize::try_from(SECURITY_MAX_SID_SIZE)
            .expect("SECURITY_MAX_SID_SIZE is longer than usize");
        let buffer = unsafe { Memory::LocalAlloc(Memory::LPTR, len)? };
        let inner = PSID(buffer.0 as *mut _);
        Ok(Self { inner })
    }

    /// Returns SID for current user.
    pub fn current_user() -> Result<Self> {
        let mut token_handle = windows::Win32::Foundation::HANDLE::default();

        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle)? };

        let mut buffer_size = 0;
        unsafe { GetTokenInformation(token_handle, TokenUser, None, 0, &mut buffer_size) }
            .or_else(|err| {
                // It's expected to return the insufficient buffer error
                if err.code() == HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0) {
                    Ok(())
                } else {
                    Err(err)
                }
            })?;

        let len: usize = usize::try_from(buffer_size).expect("buffer_size is larger than usize");
        let buffer = unsafe { LocalAlloc(Memory::LPTR, len)? };
        match unsafe {
            GetTokenInformation(
                token_handle,
                TokenUser,
                Some(buffer.0),
                buffer_size,
                &mut buffer_size,
            )
        } {
            Ok(()) => {
                let token_user = buffer.0 as *const TOKEN_USER;
                // Safety: safe in Ok()
                let psid = unsafe { (*token_user).User.Sid };
                unsafe { Sid::new_with_copy(psid) }
            }
            Err(e) => {
                unsafe { LocalFree(Some(buffer)) };
                Err(e)
            }
        }
    }

    /// Returns true if SID is well known.
    pub fn is_well_known(&self, sid_type: WELL_KNOWN_SID_TYPE) -> bool {
        unsafe { IsWellKnownSid(self.inner, sid_type).as_bool() }
    }

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

    /// Lookup user account associated with the SID.
    pub fn lookup_account(&self) -> Result<LookedUpAccount> {
        let mut account_name_len = 0;
        let mut domain_name_len = 0;
        let mut sid_type = SID_NAME_USE::default();

        unsafe {
            LookupAccountSidW(
                None,
                self.inner,
                None,
                &mut account_name_len,
                None,
                &mut domain_name_len,
                &mut sid_type,
            )
        }
        .or_else(|err| {
            // It's expected to return the insufficient buffer error
            if err.code() == HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0) {
                Ok(())
            } else {
                Err(err)
            }
        })?;

        let account_name_len_sz =
            usize::try_from(account_name_len).expect("name len is larger than usize");
        let domain_name_len_sz =
            usize::try_from(domain_name_len).expect("domain len is larger than usize");

        let mut name_bytes = vec![0u16; account_name_len_sz];
        let mut domain_bytes = vec![0u16; domain_name_len_sz];

        let account_name_str = PWSTR(name_bytes.as_mut_ptr());
        let domain_name_str = PWSTR(domain_bytes.as_mut_ptr());

        unsafe {
            LookupAccountSidW(
                None,
                self.inner,
                Some(account_name_str),
                &mut account_name_len,
                Some(domain_name_str),
                &mut domain_name_len,
                &mut sid_type,
            )?
        };

        Ok(LookedUpAccount {
            account_name: unsafe { account_name_str.to_string()? },
            domain_name: unsafe { domain_name_str.to_string()? },
            sid_type,
        })
    }

    /// Returns a copy of the SID.
    pub fn clone(&self) -> Result<Self> {
        unsafe { Self::new_with_copy(self.inner) }
    }

    /// Returns the inner `PSID`.
    ///
    /// # Safety
    /// The returned value stores raw pointers inside, which are only guaranteed to remain valid during the lifetime of this struct.
    pub unsafe fn inner(&self) -> PSID {
        self.inner
    }
}

impl PartialEq for Sid {
    fn eq(&self, other: &Self) -> bool {
        unsafe { Security::EqualSid(self.inner, other.inner).is_ok() }
    }
}

impl Drop for Sid {
    fn drop(&mut self) {
        if !self.inner.is_invalid() {
            unsafe { Security::FreeSid(self.inner) };
        }
    }
}

#[derive(Debug)]
pub struct LookedUpAccount {
    pub account_name: String,
    pub domain_name: String,
    /// Type of security identifier (SID)
    pub sid_type: SID_NAME_USE,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_everyone_sid_to_string() {
        let sid = Sid::everyone().unwrap();
        let sid_str = sid.to_string().unwrap();
        assert_eq!(sid_str, "S-1-1-0");
    }

    #[test]
    fn test_clone_sid() {
        let src = Sid::everyone().unwrap();
        let dst = src.clone().unwrap();
        assert_eq!(src, dst);
    }

    #[test]
    fn test_current_user_sid() {
        Sid::current_user().unwrap();
    }

    #[test]
    fn test_lookup_account() {
        let sid = Sid::current_user().unwrap();
        sid.lookup_account().unwrap();
    }
}
