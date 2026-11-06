// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    ffi::{CString, c_void},
    ptr::{self, NonNull},
};

use objc2_security::{
    AuthorizationCreate, AuthorizationEnvironment, AuthorizationExternalForm, AuthorizationFlags,
    AuthorizationItem, AuthorizationMakeExternalForm, AuthorizationRef,
    kAuthorizationEnvironmentPrompt,
};

use crate::{
    Error, Result,
    service::{check, copy_unlock_right, free_authorization},
};

/// Authorization holding the unlock right, obtained in the user's session.
///
/// The token returned by [`Self::token`] stays valid only while this value is alive.
pub struct UnlockAuthorization(AuthorizationRef);

impl UnlockAuthorization {
    /// Ask the user logged in to the current session to authenticate. Blocks while the prompt is
    /// shown. `prompt` is displayed in the authentication dialog.
    pub fn request(prompt: &str) -> Result<Self> {
        let mut auth_ref: AuthorizationRef = ptr::null();
        check(unsafe {
            AuthorizationCreate(
                ptr::null(),
                ptr::null(),
                AuthorizationFlags::Defaults,
                &mut auth_ref,
            )
        })?;
        let authorization = Self(auth_ref);

        let prompt = CString::new(prompt).map_err(|_| Error::InvalidPrompt)?;
        let mut prompt_item = AuthorizationItem {
            name: kAuthorizationEnvironmentPrompt.as_ptr(),
            valueLength: prompt.as_bytes().len(),
            value: prompt.as_ptr().cast_mut().cast::<c_void>(),
            flags: 0,
        };
        let environment = AuthorizationEnvironment {
            count: 1,
            items: &mut prompt_item,
        };

        copy_unlock_right(
            authorization.0,
            &environment,
            AuthorizationFlags::InteractionAllowed | AuthorizationFlags::ExtendRights,
        )?;

        Ok(authorization)
    }

    /// Externalized authorization to hand over to the process verifying it.
    pub fn token(&self) -> Result<Vec<u8>> {
        let mut form = AuthorizationExternalForm { bytes: [0; 32] };
        check(unsafe { AuthorizationMakeExternalForm(self.0, NonNull::from(&mut form)) })?;
        Ok(form.bytes.iter().map(|&byte| byte as u8).collect())
    }
}

impl Drop for UnlockAuthorization {
    fn drop(&mut self) {
        free_authorization(self.0, AuthorizationFlags::Defaults);
    }
}
