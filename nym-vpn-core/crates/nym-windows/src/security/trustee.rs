// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::Win32::Security::Authorization::{
    BuildTrusteeWithSidW, TRUSTEE_FORM, TRUSTEE_IS_ALIAS, TRUSTEE_IS_COMPUTER, TRUSTEE_IS_DELETED,
    TRUSTEE_IS_DOMAIN, TRUSTEE_IS_GROUP, TRUSTEE_IS_INVALID, TRUSTEE_IS_NAME,
    TRUSTEE_IS_OBJECTS_AND_NAME, TRUSTEE_IS_OBJECTS_AND_SID, TRUSTEE_IS_SID, TRUSTEE_IS_UNKNOWN,
    TRUSTEE_IS_USER, TRUSTEE_IS_WELL_KNOWN_GROUP, TRUSTEE_TYPE, TRUSTEE_W,
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
        let mut trustee = TRUSTEE_W::default();
        unsafe { BuildTrusteeWithSidW(&mut trustee, Some(sid.inner())) };
        trustee.TrusteeForm = TRUSTEE_IS_SID;
        trustee.TrusteeType = trustee_type.into();

        Self {
            inner: trustee,
            _sid: sid,
        }
    }

    /// Returns a copy of inner `TRUSTEE_W`.
    ///
    /// # Safety
    /// The returned value stores raw pointers inside, which are only guaranteed to remain valid during the lifetime of this struct.
    pub unsafe fn inner(&self) -> TRUSTEE_W {
        self.inner
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TrusteeForm {
    Sid,
    Name,
    ObjectsAndSid,
    ObjectsAndName,
}

impl From<TRUSTEE_FORM> for TrusteeForm {
    fn from(trustee_form: TRUSTEE_FORM) -> Self {
        match trustee_form {
            TRUSTEE_IS_SID => Self::Sid,
            TRUSTEE_IS_NAME => Self::Name,
            TRUSTEE_IS_OBJECTS_AND_SID => Self::ObjectsAndSid,
            TRUSTEE_IS_OBJECTS_AND_NAME => Self::ObjectsAndName,
            _ => Self::ObjectsAndName,
        }
    }
}

/// Type of trustee.
#[derive(Debug, Copy, Clone)]
pub enum TrusteeType {
    User,
    Group,
    Domain,
    Alias,
    WellKnownGroup,
    Deleted,
    Invalid,
    Computer,
    Unknown,
}

impl From<TrusteeType> for TRUSTEE_TYPE {
    fn from(value: TrusteeType) -> Self {
        match value {
            TrusteeType::User => TRUSTEE_IS_USER,
            TrusteeType::Group => TRUSTEE_IS_GROUP,
            TrusteeType::Domain => TRUSTEE_IS_DOMAIN,
            TrusteeType::Alias => TRUSTEE_IS_ALIAS,
            TrusteeType::WellKnownGroup => TRUSTEE_IS_WELL_KNOWN_GROUP,
            TrusteeType::Deleted => TRUSTEE_IS_DELETED,
            TrusteeType::Invalid => TRUSTEE_IS_INVALID,
            TrusteeType::Computer => TRUSTEE_IS_COMPUTER,
            TrusteeType::Unknown => TRUSTEE_IS_UNKNOWN,
        }
    }
}

impl From<TRUSTEE_TYPE> for TrusteeType {
    fn from(trustee_type: TRUSTEE_TYPE) -> Self {
        match trustee_type {
            TRUSTEE_IS_USER => Self::User,
            TRUSTEE_IS_GROUP => Self::Group,
            TRUSTEE_IS_DOMAIN => Self::Domain,
            TRUSTEE_IS_ALIAS => Self::Alias,
            TRUSTEE_IS_WELL_KNOWN_GROUP => Self::WellKnownGroup,
            TRUSTEE_IS_DELETED => Self::Deleted,
            TRUSTEE_IS_INVALID => Self::Invalid,
            TRUSTEE_IS_COMPUTER => Self::Computer,
            TRUSTEE_IS_UNKNOWN => Self::Unknown,
            _ => Self::Unknown,
        }
    }
}
