// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::{
    core::PWSTR,
    Win32::Security::Authorization::{TRUSTEE_IS_SID, TRUSTEE_TYPE, TRUSTEE_W},
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
    pub fn new(sid: Sid, trustee_type: TRUSTEE_TYPE) -> Self {
        let inner = TRUSTEE_W {
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: trustee_type,

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
