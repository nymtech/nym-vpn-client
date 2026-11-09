// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use objc2_foundation::NSError;
use objc2_local_authentication::{LAError, LAErrorDomain};

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
    pub fn from_ns_error(error: &NSError) -> Self {
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

    pub fn internal(description: &str) -> Self {
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
