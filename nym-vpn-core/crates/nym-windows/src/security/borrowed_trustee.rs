// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use windows::{
    core::Result,
    Win32::Security::{
        Authorization::{GetTrusteeFormW, GetTrusteeTypeW, TRUSTEE_W},
        PSID,
    },
};

use super::{Sid, TrusteeForm, TrusteeType};

/// Borrowed version of `Trustee`
#[derive(Debug)]
pub struct BorrowedTrustee<'a> {
    inner: &'a TRUSTEE_W,
}
impl<'a> BorrowedTrustee<'a> {
    /// Create new instance from reference, without taking ownership of memory.
    ///
    /// # Safety
    /// The caller must ensure the validity of pointer during the lifetime of this struct.
    pub unsafe fn new(trustee: &'a TRUSTEE_W) -> Self {
        Self { inner: trustee }
    }

    /// Get type of trustee.
    pub fn get_trustee_type(&self) -> TrusteeType {
        TrusteeType::from(unsafe { GetTrusteeTypeW(Some(self.inner)) })
    }

    /// Get trustee form.
    pub fn get_trustee_form(&self) -> TrusteeForm {
        TrusteeForm::from(unsafe { GetTrusteeFormW(self.inner) })
    }

    /// Get trustee specific info.
    pub fn get_trustee_specific_info(&self) -> Result<TrusteeSpecificInfo> {
        match self.get_trustee_form() {
            TrusteeForm::Name => {
                let name = unsafe { self.inner.ptstrName.to_string() }?;
                Ok(TrusteeSpecificInfo::Name(name))
            }
            TrusteeForm::Sid => {
                let psid = PSID(self.inner.ptstrName.0 as *mut _);
                let sid = unsafe { Sid::copy_from(psid)? };
                Ok(TrusteeSpecificInfo::Sid(sid))
            }
            TrusteeForm::ObjectsAndName => {
                // todo: implement parsing of OBJECTS_AND_NAME_W
                Ok(TrusteeSpecificInfo::ObjectsAndName)
            }
            TrusteeForm::ObjectsAndSid => {
                // todo: implement parsing of OBJECTS_AND_SID
                Ok(TrusteeSpecificInfo::ObjectsAndSid)
            }
        }
    }
}

#[derive(Debug)]
pub enum TrusteeSpecificInfo {
    Name(String),
    Sid(Sid),
    ObjectsAndSid,
    ObjectsAndName,
}

impl fmt::Display for TrusteeSpecificInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(name) => f.write_str(name),
            Self::Sid(sid) => f.write_str(
                sid.to_string()
                    .as_deref()
                    .unwrap_or("failed to convert SID to string"),
            ),
            Self::ObjectsAndName => f.write_str("[ObjectsAndName]"),
            Self::ObjectsAndSid => f.write_str("[ObjectsAndSid]"),
        }
    }
}
