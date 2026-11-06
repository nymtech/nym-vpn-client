// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    ffi::{CStr, c_char},
    ptr::{self, NonNull},
    time::Duration,
};

use objc2_core_foundation::{CFBoolean, CFDictionary, CFNumber, CFString, CFType};
use objc2_security::{
    AuthorizationCopyRights, AuthorizationCreate, AuthorizationCreateFromExternalForm,
    AuthorizationEnvironment, AuthorizationExternalForm, AuthorizationFlags, AuthorizationFree,
    AuthorizationItem, AuthorizationRef, AuthorizationRightSet, AuthorizationRights,
    errAuthorizationCanceled, errAuthorizationDenied, errAuthorizationInteractionNotAllowed,
    errAuthorizationSuccess,
};

use crate::{Client, Error, Result};

const UNLOCK_RIGHT: &CStr = c"net.nymtech.vpn.unlock-account";

/// How long the right granted to the client stays valid, enough to hand it over to the process
/// verifying it.
const UNLOCK_RIGHT_TIMEOUT_SECS: i32 = 30;

/// How long the service stays authorized after verifying a token.
pub const AUTHORIZATION_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// Software lock unlocked with the token obtained with [`crate::Authorization`]. Callers check
/// [`Self::is_authorized`] before performing operations that require user presence.
///
/// Only the client that unlocked it is authorized, and only one client at a time. The
/// authorization expires after [`AUTHORIZATION_TIMEOUT`], including time spent asleep.
#[derive(Default)]
pub struct AuthorizationService {
    grant: Option<Grant>,
}

struct Grant {
    client: Client,
    expires_at: Duration,
}

impl AuthorizationService {
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
            // When a right is registered with class set to user, it instructs the authd (Authorization daemon) daemon
            // that granting this privilege requires verifying a specific identity.
            CFString::from_static_str("class"),
            // If set to false, it specifies that a user does not need to provide authentication to gain this right.
            CFString::from_static_str("authenticate-user"),
            // If set to true, the owner of the session (the currently logged-in user) would automatically get this right.
            // This might bypass additional authentication if the user is already logged in.
            CFString::from_static_str("session-owner"),
            // If a user is operating as the root user (which has elevated permissions), and this key is set to true, the root user
            // could potentially gain this right without further authentication.
            // However, typically, getting to a root user status already requires authentication,
            // so this isn’t a "no authentication" scenario for most users.
            CFString::from_static_str("allow-root"),
            // If set to true, the owner of the session (the currently logged-in user) would automatically get this right.
            // This might bypass additional authentication if the user is already logged in.
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
        let description = CFString::from_static_str("Access your Nym VPN account");

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
        free_authorization(auth_ref, AuthorizationFlags::Defaults);
        check(status)
    }

    /// Verify the token and authorize `client`, replacing any other client's authorization. The
    /// token cannot be used again afterwards.
    pub fn authorize(&mut self, token: &[u8], client: Client) -> Result<()> {
        verify_unlock_token(token)?;
        self.grant = Some(Grant {
            client,
            expires_at: continuous_time()? + AUTHORIZATION_TIMEOUT,
        });
        Ok(())
    }

    /// Revoke the authorization if it belongs to `client`.
    pub fn deauthorize(&mut self, client: &Client) {
        if self
            .grant
            .as_ref()
            .is_some_and(|grant| grant.client == *client)
        {
            self.grant = None;
        }
    }

    pub fn is_authorized(&self, client: &Client) -> bool {
        self.grant.as_ref().is_some_and(|grant| {
            grant.client == *client && continuous_time().is_ok_and(|now| now < grant.expires_at)
        })
    }
}

/// Time since boot including time spent asleep, unlike [`std::time::Instant`] on macOS.
fn continuous_time() -> Result<Duration> {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) } == 0 {
        Ok(Duration::new(time.tv_sec as u64, time.tv_nsec as u32))
    } else {
        Err(Error::Os(
            std::io::Error::last_os_error()
                .raw_os_error()
                .unwrap_or_default(),
        ))
    }
}

/// Verify that the token grants the unlock right, without prompting.
///
/// The rights are destroyed afterwards, so the token cannot be used again.
fn verify_unlock_token(token: &[u8]) -> Result<()> {
    let bytes: [u8; 32] = token.try_into().map_err(|_| Error::AuthenticationFailed)?;
    let form = AuthorizationExternalForm {
        bytes: bytes.map(|byte| byte.cast_signed()),
    };

    let mut auth_ref: AuthorizationRef = ptr::null();
    check(unsafe {
        AuthorizationCreateFromExternalForm(NonNull::from(&form), NonNull::from(&mut auth_ref))
    })?;

    let result = copy_unlock_right(auth_ref, ptr::null(), AuthorizationFlags::ExtendRights);
    free_authorization(auth_ref, AuthorizationFlags::DestroyRights);
    result
}

pub fn copy_unlock_right(
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

pub fn free_authorization(auth_ref: AuthorizationRef, flags: AuthorizationFlags) {
    let status = unsafe { AuthorizationFree(auth_ref, flags) };
    if status != errAuthorizationSuccess {
        tracing::error!("Failed to free authorization ({flags:?}): {status}");
    }
}

pub fn check(status: i32) -> Result<()> {
    match status {
        x if x == errAuthorizationSuccess => Ok(()),
        x if x == errAuthorizationCanceled => Err(Error::UserCancelled),
        x if x == errAuthorizationDenied || x == errAuthorizationInteractionNotAllowed => {
            Err(Error::AuthenticationFailed)
        }
        status => Err(Error::Os(status)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIENT: Client = Client {
        user_id: 501,
        audit_session_id: Some(100_001),
    };

    fn service_authorized_until(client: Client, expires_at: Duration) -> AuthorizationService {
        AuthorizationService {
            grant: Some(Grant { client, expires_at }),
        }
    }

    fn deadline() -> Duration {
        continuous_time().unwrap() + AUTHORIZATION_TIMEOUT
    }

    #[test]
    fn not_authorized_by_default() {
        assert!(!AuthorizationService::default().is_authorized(&CLIENT));
    }

    #[test]
    fn authorized_before_deadline() {
        assert!(service_authorized_until(CLIENT, deadline()).is_authorized(&CLIENT));
    }

    #[test]
    fn expires_after_deadline() {
        let expired = continuous_time().unwrap() - Duration::from_secs(1);
        assert!(!service_authorized_until(CLIENT, expired).is_authorized(&CLIENT));
    }

    #[test]
    fn other_user_is_not_authorized() {
        let other = Client {
            user_id: 502,
            ..CLIENT
        };
        assert!(!service_authorized_until(CLIENT, deadline()).is_authorized(&other));
    }

    #[test]
    fn other_session_is_not_authorized() {
        let other = Client {
            audit_session_id: Some(100_002),
            ..CLIENT
        };
        assert!(!service_authorized_until(CLIENT, deadline()).is_authorized(&other));
    }

    #[test]
    fn unknown_session_does_not_match_known_session() {
        let other = Client {
            audit_session_id: None,
            ..CLIENT
        };
        assert!(!service_authorized_until(CLIENT, deadline()).is_authorized(&other));
    }

    #[test]
    fn deauthorize_revokes_authorization() {
        let mut service = service_authorized_until(CLIENT, deadline());
        service.deauthorize(&CLIENT);
        assert!(!service.is_authorized(&CLIENT));
    }

    #[test]
    fn deauthorize_by_other_client_is_ignored() {
        let other = Client {
            user_id: 502,
            ..CLIENT
        };
        let mut service = service_authorized_until(CLIENT, deadline());
        service.deauthorize(&other);
        assert!(service.is_authorized(&CLIENT));
    }
}
