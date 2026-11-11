// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Uniffi interface for requesting authorization with `nym-authorization-service`.

use std::{
    sync::{Arc, mpsc},
    thread,
};

use futures::channel::oneshot;

use nym_authorization_service::{Authorization, Error as InnerError};

type Result<T, E = AuthorizationError> = std::result::Result<T, E>;

/// Externalized authorization to hand over to the daemon.
///
/// [`Authorization`] must be created and dropped on the same thread, so each token keeps it on a
/// dedicated thread. The authorization is released on that thread once the token is dropped.
#[derive(uniffi::Object)]
pub struct AuthorizationToken {
    bytes: Vec<u8>,
    _release: mpsc::Sender<()>,
}

#[uniffi::export]
impl AuthorizationToken {
    /// Ask the user to authenticate. `prompt` is shown in the authentication dialog.
    #[uniffi::constructor]
    pub async fn new(prompt: String) -> Result<Arc<Self>> {
        let (reply, rx) = oneshot::channel();
        let (release, release_rx) = mpsc::channel::<()>();

        thread::Builder::new()
            .name("authorization".to_owned())
            .spawn(move || hold_authorization(&prompt, reply, release_rx))
            .map_err(|_| AuthorizationError::Internal)?;

        let bytes = rx.await.unwrap_or(Err(AuthorizationError::Internal))?;
        Ok(Arc::new(Self {
            bytes,
            _release: release,
        }))
    }
}

impl AuthorizationToken {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Obtains the authorization and keeps it until the token's release sender is dropped.
fn hold_authorization(
    prompt: &str,
    reply: oneshot::Sender<Result<Vec<u8>>>,
    release: mpsc::Receiver<()>,
) {
    match Authorization::request(prompt).and_then(|authorization| {
        let token = authorization.token()?;
        Ok((token, authorization))
    }) {
        Ok((token, authorization)) => {
            // Fails if the caller is gone, in which case the release sender is already dropped.
            let _ = reply.send(Ok(token));
            let _ = release.recv();
            drop(authorization);
        }
        Err(err) => {
            let _ = reply.send(Err(err.into()));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum AuthorizationError {
    #[error("User cancelled")]
    UserCancelled,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Prompt contains a NUL byte")]
    InvalidPrompt,

    #[error("Authorization error: {code}")]
    Os { code: i32 },

    #[error("Internal error")]
    Internal,
}

impl From<InnerError> for AuthorizationError {
    fn from(err: InnerError) -> Self {
        match err {
            InnerError::UserCancelled => Self::UserCancelled,
            InnerError::AuthenticationFailed => Self::AuthenticationFailed,
            InnerError::InvalidPrompt => Self::InvalidPrompt,
            InnerError::Os(code) => Self::Os { code },
        }
    }
}
