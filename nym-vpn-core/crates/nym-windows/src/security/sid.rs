// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::{
    core::{Result, HRESULT, PWSTR},
    Win32::{
        Foundation::{LocalFree, ERROR_INSUFFICIENT_BUFFER, HLOCAL},
        Security::{
            Authentication::Identity::{
                LsaClose, LsaFreeMemory, LsaOpenPolicy, LsaQueryInformationPolicy,
                PolicyAccountDomainInformation, LSA_HANDLE, LSA_OBJECT_ATTRIBUTES,
                POLICY_ACCOUNT_DOMAIN_INFO, POLICY_VIEW_LOCAL_INFORMATION,
            },
            Authorization::ConvertSidToStringSidW,
            CopySid, CreateWellKnownSid, EqualSid, FreeSid, GetLengthSid, GetTokenInformation,
            IsWellKnownSid, LookupAccountSidW, TokenUser, PSID, SECURITY_MAX_SID_SIZE,
            SID_NAME_USE, TOKEN_QUERY, TOKEN_USER, WELL_KNOWN_SID_TYPE,
        },
        System::{
            Memory::{LocalAlloc, LPTR},
            Threading::{GetCurrentProcess, OpenProcessToken},
        },
    },
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct LocalDomain {
    pub domain_name: String,
    pub domain_sid: Sid,
}

impl LocalDomain {
    /// Query local domain information returning domain name and SID
    #[allow(dead_code)]
    pub fn query() -> Result<Self> {
        let object_attributes = LSA_OBJECT_ATTRIBUTES {
            // Safety: unlikely to exceed u32
            Length: std::mem::size_of::<LSA_OBJECT_ATTRIBUTES>() as u32,
            ..Default::default()
        };
        let mut policy_handle = LSA_HANDLE::default();

        unsafe {
            LsaOpenPolicy(
                None,
                &object_attributes,
                // Safety: safe because the constant is equal to 1i32
                POLICY_VIEW_LOCAL_INFORMATION as u32,
                &mut policy_handle,
            )
            .ok()?;
        }

        let mut ppadi: *mut POLICY_ACCOUNT_DOMAIN_INFO = std::ptr::null_mut();

        unsafe {
            LsaQueryInformationPolicy(
                policy_handle,
                PolicyAccountDomainInformation,
                &mut ppadi as *mut _ as _,
            )
            .to_hresult()
            .ok()?;
        }

        let domain_name = unsafe { (*ppadi).DomainName.Buffer.to_hstring() };
        let domain_sid = unsafe { Sid::copy_from((*ppadi).DomainSid) };

        unsafe {
            LsaFreeMemory(Some(ppadi as *const _))
                .to_hresult()
                .ok()
                .or(LsaClose(policy_handle).to_hresult().ok())?;
        }

        Ok(Self {
            domain_name: domain_name.to_string(),
            domain_sid: domain_sid?,
        })
    }
}

/// Struct that uniquely identifies users or groups.
#[derive(Debug, Eq)]
pub struct Sid {
    inner: PSID,
}

impl Sid {
    /// Create new SID copying data from raw pointer.
    pub(crate) unsafe fn copy_from(psid: PSID) -> Result<Self> {
        let sid_len = GetLengthSid(psid);
        let sid_len_sz = usize::try_from(sid_len).expect("sid length is too large");
        let buffer = LocalAlloc(LPTR, sid_len_sz)?;
        let dest_sid = PSID(buffer.0 as *mut _);

        unsafe { CopySid(sid_len, dest_sid, psid)? };

        Ok(Self { inner: dest_sid })
    }

    /// Create new well known SID with domain set to local computer.
    pub fn well_known(well_known_sid: WellKnownSid) -> Result<Self> {
        Self::well_known_with_domain(well_known_sid, None)
    }

    /// Create new well known SID, optionally providing domain.
    ///
    /// * `well_known_sid` - a type of well known SID to create.
    /// * `domain_sid` - a reference to domain sid that's used when creating a well known sid. Pass `None` to use the local computer.
    pub fn well_known_with_domain(
        well_known_sid: WellKnownSid,
        domain_sid: Option<&Sid>,
    ) -> Result<Self> {
        let mut cbsize = SECURITY_MAX_SID_SIZE;
        let buffer = unsafe { LocalAlloc(LPTR, cbsize as usize)? };
        let inner = PSID(buffer.0 as *mut _);

        unsafe {
            CreateWellKnownSid(
                well_known_sid.into(),
                domain_sid.as_ref().map(|x| x.inner()),
                Some(inner),
                &mut cbsize,
            )?
        };

        Ok(Self { inner })
    }

    /// Returns true if SID is well known.
    pub fn is_well_known(&self, sid_type: WellKnownSid) -> bool {
        unsafe { IsWellKnownSid(self.inner, sid_type.into()).as_bool() }
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
        let buffer = unsafe { LocalAlloc(LPTR, len)? };

        let result = unsafe {
            GetTokenInformation(
                token_handle,
                TokenUser,
                Some(buffer.0),
                buffer_size,
                &mut buffer_size,
            )
        }
        .and_then(|_| {
            let token_user = buffer.0 as *const TOKEN_USER;
            // Safety: guaranteed to have a valid pointer
            let psid = unsafe { (*token_user).User.Sid };
            unsafe { Sid::copy_from(psid) }
        });

        unsafe { LocalFree(Some(buffer)) };

        result
    }

    /// Convert SID to string.
    pub fn to_string(&self) -> Result<String> {
        let mut wide_str = PWSTR::null();
        unsafe { ConvertSidToStringSidW(self.inner, &mut wide_str as _)? };
        let result = unsafe { wide_str.to_string()? };
        if !wide_str.is_null() {
            unsafe { LocalFree(Some(HLOCAL(wide_str.0 as *mut _))) };
        }

        Ok(result)
    }

    /// Lookup user account associated with the SID.
    pub fn lookup_account(&self) -> Result<AccountLookupResult> {
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

        Ok(AccountLookupResult {
            account_name: unsafe { account_name_str.to_string()? },
            domain_name: unsafe { domain_name_str.to_string()? },
            sid_type,
        })
    }

    /// Returns a copy of the SID.
    pub fn try_clone(&self) -> Result<Self> {
        unsafe { Self::copy_from(self.inner) }
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
        unsafe { EqualSid(self.inner, other.inner).is_ok() }
    }
}

impl Drop for Sid {
    fn drop(&mut self) {
        if !self.inner.is_invalid() {
            unsafe { FreeSid(self.inner) };
        }
    }
}

/// Result of translating SID to account name and domain.
#[derive(Debug, Clone)]
pub struct AccountLookupResult {
    /// User account name.
    pub account_name: String,

    /// Domain name.
    pub domain_name: String,

    /// Type of security identifier (SID)
    pub sid_type: SID_NAME_USE,
}

/// A mirror of `WELL_KNOWN_SID_TYPE`
#[derive(Debug, Clone, Copy)]
pub enum WellKnownSid {
    /// Indicates a SID that matches the owner or creator of an object.
    CreatorOwner,
    /// Indicates a SID that matches everyone.
    World,
    /// Indicates a SID that matches the local system.
    LocalSystem,
    /// Indicates a SID that matches the administrator group.
    BuiltinAdministrators,
    /// Indicates a SID that matches built-in user accounts.
    BuiltinUsers,
    // todo: add more well known SIDs from WELL_KNOWN_SID_TYPE
}

impl From<WellKnownSid> for WELL_KNOWN_SID_TYPE {
    fn from(well_known_sid: WellKnownSid) -> Self {
        use windows::Win32::Security as S;
        match well_known_sid {
            WellKnownSid::CreatorOwner => S::WinCreatorOwnerSid,
            WellKnownSid::World => S::WinWorldSid,
            WellKnownSid::LocalSystem => S::WinLocalSystemSid,
            WellKnownSid::BuiltinAdministrators => S::WinBuiltinAdministratorsSid,
            WellKnownSid::BuiltinUsers => S::WinBuiltinUsersSid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_well_known() {
        let sid = Sid::well_known(WellKnownSid::World).unwrap();
        assert!(sid.is_well_known(WellKnownSid::World));
        assert!(!sid.is_well_known(WellKnownSid::BuiltinUsers));
    }

    #[test]
    fn test_sid_to_string() {
        let sid = Sid::well_known(WellKnownSid::World).unwrap();
        let sid_str = sid.to_string().unwrap();
        assert_eq!(sid_str, "S-1-1-0");
    }

    #[test]
    fn test_clone_sid() {
        let src = Sid::well_known(WellKnownSid::World).unwrap();
        let dst = src.try_clone().unwrap();
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

    #[test]
    fn test_get_local_domain() {
        LocalDomain::query().unwrap();
    }
}
