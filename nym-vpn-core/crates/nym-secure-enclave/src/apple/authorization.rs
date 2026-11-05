// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Unlocking with Authorization Services.
//!
//! The daemon runs as root, so it cannot authenticate the logged-in user itself. Instead the
//! client obtains the unlock right in the user's session and hands the externalized authorization
//! to the daemon, which verifies that the right was granted without prompting.

use std::{
    ffi::{CStr, CString, c_char, c_void},
    ptr::{self, NonNull},
};

use objc2_core_foundation::{CFBoolean, CFDictionary, CFNumber, CFString, CFType};
use objc2_security::{
    AuthorizationCopyRights, AuthorizationCreate, AuthorizationCreateFromExternalForm,
    AuthorizationEnvironment, AuthorizationExternalForm, AuthorizationFlags, AuthorizationFree,
    AuthorizationItem, AuthorizationMakeExternalForm, AuthorizationRef, AuthorizationRightSet,
    AuthorizationRights, errAuthorizationCanceled, errAuthorizationDenied,
    errAuthorizationInteractionNotAllowed, errAuthorizationSuccess,
    kAuthorizationEnvironmentPrompt,
};

use crate::{Result, SecureEnclaveError};

const UNLOCK_RIGHT: &CStr = c"net.nymtech.vpn.unlock-account";

/// How long the right granted to the client stays valid, enough to hand it over to the daemon.
const UNLOCK_RIGHT_TIMEOUT_SECS: i32 = 30;

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

        let prompt = CString::new(prompt)
            .map_err(|_| SecureEnclaveError::Other("prompt contains a NUL byte".to_owned()))?;
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

    /// Externalized authorization to send to the daemon.
    pub fn token(&self) -> Result<Vec<u8>> {
        let mut form = AuthorizationExternalForm { bytes: [0; 32] };
        check(unsafe { AuthorizationMakeExternalForm(self.0, NonNull::from(&mut form)) })?;
        Ok(form.bytes.iter().map(|&byte| byte as u8).collect())
    }
}

impl Drop for UnlockAuthorization {
    fn drop(&mut self) {
        unsafe { AuthorizationFree(self.0, AuthorizationFlags::Defaults) };
    }
}

/// Register the unlock right in the authorization database. Requires root.
///
/// The right requires the owner of the client's session to authenticate, is never granted to
/// root without authentication and is not shared with other authorizations in the session, so
/// every unlock prompts again.
pub fn register_unlock_right() -> Result<()> {
    let mut auth_ref: AuthorizationRef = ptr::null();
    check(unsafe {
        AuthorizationCreate(
            ptr::null(),
            ptr::null(),
            AuthorizationFlags::Defaults,
            &mut auth_ref,
        )
    })?;

    let keys = [
        CFString::from_static_str("class"),
        CFString::from_static_str("authenticate-user"),
        CFString::from_static_str("session-owner"),
        CFString::from_static_str("allow-root"),
        CFString::from_static_str("shared"),
        CFString::from_static_str("timeout"),
    ];
    let class = CFString::from_static_str("user");
    let timeout = CFNumber::new_i32(UNLOCK_RIGHT_TIMEOUT_SECS);
    let values: [&CFType; 6] = [
        &class,
        CFBoolean::new(true),
        CFBoolean::new(true),
        CFBoolean::new(false),
        CFBoolean::new(false),
        &timeout,
    ];
    let keys: Vec<&CFString> = keys.iter().map(|key| &**key).collect();
    let definition = CFDictionary::<CFString, CFType>::from_slices(&keys, &values);
    let description = CFString::from_static_str("access your NymVPN account");

    let status = unsafe {
        AuthorizationRightSet(
            auth_ref,
            NonNull::from(UNLOCK_RIGHT).cast::<c_char>(),
            definition.as_opaque(),
            Some(&description),
            None,
            None,
        )
    };
    unsafe { AuthorizationFree(auth_ref, AuthorizationFlags::Defaults) };
    check(status)
}

/// Verify that the token grants the unlock right, without prompting.
///
/// The rights are destroyed afterwards, so the token cannot be used again.
pub fn verify_unlock_token(token: &[u8]) -> Result<()> {
    let bytes: [u8; 32] = token
        .try_into()
        .map_err(|_| SecureEnclaveError::AuthenticationFailed)?;
    let form = AuthorizationExternalForm {
        bytes: bytes.map(|byte| byte as c_char),
    };

    let mut auth_ref: AuthorizationRef = ptr::null();
    check(unsafe {
        AuthorizationCreateFromExternalForm(NonNull::from(&form), NonNull::from(&mut auth_ref))
    })?;

    let result = copy_unlock_right(auth_ref, ptr::null(), AuthorizationFlags::ExtendRights);
    unsafe { AuthorizationFree(auth_ref, AuthorizationFlags::DestroyRights) };
    result
}

fn copy_unlock_right(
    auth_ref: AuthorizationRef,
    environment: *const AuthorizationEnvironment,
    flags: AuthorizationFlags,
) -> Result<()> {
    let mut right = AuthorizationItem {
        name: UNLOCK_RIGHT.as_ptr(),
        valueLength: 0,
        value: ptr::null_mut(),
        flags: 0,
    };
    let rights = AuthorizationRights {
        count: 1,
        items: &mut right,
    };
    check(unsafe {
        AuthorizationCopyRights(
            auth_ref,
            NonNull::from(&rights),
            environment,
            flags,
            ptr::null_mut(),
        )
    })
}

fn check(status: i32) -> Result<()> {
    match status {
        x if x == errAuthorizationSuccess => Ok(()),
        x if x == errAuthorizationCanceled => Err(SecureEnclaveError::UserCancelled),
        x if x == errAuthorizationDenied || x == errAuthorizationInteractionNotAllowed => {
            Err(SecureEnclaveError::AuthenticationFailed)
        }
        status => Err(SecureEnclaveError::Other(format!(
            "authorization error: {status}"
        ))),
    }
}
