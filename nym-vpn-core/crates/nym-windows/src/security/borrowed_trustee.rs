// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::{
    core::Result,
    Win32::Security::{Authorization::TRUSTEE_W, PSID},
};

use super::{Sid, TrusteeForm, TrusteeType};

/// tbd
pub struct BorrowedTrustee<'a> {
    inner: &'a TRUSTEE_W,
}
impl<'a> BorrowedTrustee<'a> {
    /// tbd
    pub unsafe fn new(trustee: &'a TRUSTEE_W) -> Self {
        Self { inner: trustee }
    }

    /// tbd
    pub fn get_trustee_type(&self) -> TrusteeType {
        TrusteeType::from((*self.inner).TrusteeType)
    }

    /// tbd
    pub fn get_trustee_form(&self) -> TrusteeForm {
        TrusteeForm::from((*self.inner).TrusteeForm)
    }

    /// tbd
    pub fn get_trustee_specific_info(&self) -> Result<TrusteeSpecificInfo> {
        // union {
        //     LPWSTR             ptstrName;
        //     SID                *pSid;
        //     OBJECTS_AND_SID    *pObjectsAndSid;
        //     OBJECTS_AND_NAME_W *pObjectsAndName;
        // };

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

/// tbd
#[derive(Debug)]
pub enum TrusteeSpecificInfo {
    /// tbd
    Name(String),
    /// tbd
    Sid(Sid),
    /// tbd
    ObjectsAndSid,
    /// tbd
    ObjectsAndName,
}
