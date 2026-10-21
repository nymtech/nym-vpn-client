// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Mutex;

use block2::RcBlock;
use objc2::{rc::Retained, runtime::Bool};
use objc2_foundation::{NSError, NSString};
pub use objc2_local_authentication::LAErrorDomain;
use objc2_local_authentication::{LAContext, LAError, LAPolicy};
use tokio::sync::oneshot;

use super::security::{SecError, SecResult};

#[derive(Debug)]
pub struct AuthError {
    pub kind: AuthErrorKind,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthErrorKind {
    Failed,
    Cancelled,
    Other,
}

impl AuthError {
    fn from_ns_error(error: &NSError) -> Self {
        let is_la_error = error.domain().isEqualToString(unsafe { LAErrorDomain });
        let kind = if is_la_error {
            match LAError(error.code()) {
                LAError::AuthenticationFailed => AuthErrorKind::Failed,
                LAError::UserCancel | LAError::SystemCancel | LAError::AppCancel => {
                    AuthErrorKind::Cancelled
                }
                _ => AuthErrorKind::Other,
            }
        } else {
            AuthErrorKind::Other
        };

        Self {
            kind,
            description: error.localizedDescription().to_string(),
        }
    }

    fn internal(description: &str) -> Self {
        Self {
            kind: AuthErrorKind::Other,
            description: description.to_owned(),
        }
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.description)
    }
}

/// Create a context that does not allow user interaction, so that operations requiring user
/// presence fail instead of prompting.
pub fn non_interactive_context() -> Retained<LAContext> {
    let context = unsafe { LAContext::new() };
    unsafe { context.setInteractionNotAllowed(true) };
    context
}

/// Authenticate the device owner and return the evaluated context.
///
/// Keychain operations performed with the returned context do not prompt the user again until
/// the context is invalidated.
pub async fn authenticate(reason: &str) -> SecResult<Retained<LAContext>> {
    let context = unsafe { LAContext::new() };
    let reason = NSString::from_str(reason);

    let (tx, rx) = oneshot::channel::<Result<(), AuthError>>();
    let tx = Mutex::new(Some(tx));
    let reply = RcBlock::new(move |success: Bool, error: *mut NSError| {
        let result = if success.as_bool() {
            Ok(())
        } else {
            Err(unsafe { error.as_ref() }
                .map(AuthError::from_ns_error)
                .unwrap_or_else(|| AuthError::internal("policy evaluation failed")))
        };

        if let Some(tx) = tx.lock().ok().and_then(|mut tx| tx.take()) {
            _ = tx.send(result);
        }
    });

    unsafe {
        context.evaluatePolicy_localizedReason_reply(
            LAPolicy::DeviceOwnerAuthentication,
            &reason,
            &reply,
        )
    };

    rx.await
        .map_err(|_| SecError::Internal("policy evaluation reply was dropped"))?
        .map_err(SecError::Auth)?;

    Ok(context)
}
