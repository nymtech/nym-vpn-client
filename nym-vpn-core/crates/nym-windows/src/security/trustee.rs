// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::{
    core::PWSTR,
    Win32::Security::Authorization::{
        TRUSTEE_IS_ALIAS, TRUSTEE_IS_COMPUTER, TRUSTEE_IS_DELETED, TRUSTEE_IS_DOMAIN,
        TRUSTEE_IS_GROUP, TRUSTEE_IS_INVALID, TRUSTEE_IS_SID, TRUSTEE_IS_USER,
        TRUSTEE_IS_WELL_KNOWN_GROUP, TRUSTEE_TYPE, TRUSTEE_W,
    },
};

use super::Sid;

/// Identifies the user account, group account, or logon session.
#[derive(Debug)]
pub struct Trustee {
    inner: TRUSTEE_W,
    // Retained to guarantee that the sid pointer held within `inner` is valid.
    _sid: Sid,
}

impl Trustee {
    /// Create new trustee with sid and type.
    pub fn new(sid: Sid, trustee_type: TrusteeType) -> Self {
        let inner = TRUSTEE_W {
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: trustee_type.to_raw(),

            // SAFETY: ptstrName is only the first variant of a union type but windows bindings lack the detail
            // so we must cast to unrelated type (LPWSTR) which simply holds a pointer.
            //
            // union {
            //     LPWSTR             ptstrName;
            //     SID                *pSid;
            //     OBJECTS_AND_SID    *pObjectsAndSid;
            //     OBJECTS_AND_NAME_W *pObjectsAndName;
            // };
            ptstrName: PWSTR(unsafe { sid.inner().0 as _ }),

            ..Default::default()
        };

        Self { inner, _sid: sid }
    }

    /// Returns a copy of inner `TRUSTEE_W`.
    ///
    /// # Safety
    /// The returned value stores raw pointers inside, which are only guaranteed to remain valid during the lifetime of this struct.
    pub unsafe fn inner(&self) -> TRUSTEE_W {
        self.inner
    }
}

/// Type of trustee.
#[derive(Debug, Copy, Clone)]
pub enum TrusteeType {
    /// tbd
    User,
    /// tbd
    Group,
    /// tbd
    Domain,
    /// tbd
    Alias,
    /// tbd
    WellKnownGroup,
    /// tbd
    Deleted,
    /// tbd
    Invalid,
    /// tbd
    Computer,
}

impl TrusteeType {
    fn to_raw(&self) -> TRUSTEE_TYPE {
        match self {
            Self::User => TRUSTEE_IS_USER,
            Self::Group => TRUSTEE_IS_GROUP,
            Self::Domain => TRUSTEE_IS_DOMAIN,
            Self::Alias => TRUSTEE_IS_ALIAS,
            Self::WellKnownGroup => TRUSTEE_IS_WELL_KNOWN_GROUP,
            Self::Deleted => TRUSTEE_IS_DELETED,
            Self::Invalid => TRUSTEE_IS_INVALID,
            Self::Computer => TRUSTEE_IS_COMPUTER,
        }
    }
}
