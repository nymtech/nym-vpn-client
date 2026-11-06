// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Uniffi interface for requesting authorization with `nym-authorization-service`.

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
};

use futures::channel::oneshot;

use nym_authorization_service::{Authorization, Error as InnerError};

type Result<T, E = AuthorizationError> = std::result::Result<T, E>;

/// Requests authorization from the user for foreign code, one request at a time.
///
/// [`Authorization`] is not thread-safe, so it is created and kept on a single worker thread. Only
/// the token leaves that thread, wrapped in an [`AuthorizationToken`] that keeps the authorization
/// alive until it is dropped.
#[derive(uniffi::Object)]
pub struct AuthorizationManager {
    commands: mpsc::Sender<Command>,
    in_progress: Arc<AtomicBool>,
}

#[uniffi::export]
impl AuthorizationManager {
    #[uniffi::constructor]
    pub fn new() -> Result<Self> {
        let (commands, rx) = mpsc::channel();
        thread::Builder::new()
            .name("authorization".to_owned())
            .spawn(move || run_worker(rx))
            .map_err(|_| AuthorizationError::Internal)?;

        Ok(Self {
            commands,
            in_progress: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Ask the user to authenticate. `prompt` is shown in the authentication dialog.
    ///
    /// Fails with [`AuthorizationError::InProgress`] while another request is pending.
    pub async fn authorize(&self, prompt: String) -> Result<Arc<AuthorizationToken>> {
        if self.in_progress.swap(true, Ordering::AcqRel) {
            Err(AuthorizationError::InProgress)
        } else {
            let (reply, rx) = oneshot::channel();
            self.commands
                .send(Command::Authorize {
                    prompt,
                    reply,
                    in_progress: InProgressGuard(self.in_progress.clone()),
                })
                .map_err(|_| AuthorizationError::Internal)?;

            let (id, bytes) = rx.await.unwrap_or(Err(AuthorizationError::Internal))?;
            Ok(Arc::new(AuthorizationToken {
                id,
                bytes,
                commands: self.commands.clone(),
            }))
        }
    }
}

/// Externalized authorization to hand over to the daemon. The authorization it refers to is
/// released when this is dropped.
#[derive(uniffi::Object)]
pub struct AuthorizationToken {
    id: u64,
    bytes: Vec<u8>,
    commands: mpsc::Sender<Command>,
}

impl AuthorizationToken {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Drop for AuthorizationToken {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Release(self.id));
    }
}

enum Command {
    Authorize {
        prompt: String,
        reply: oneshot::Sender<Result<(u64, Vec<u8>)>>,
        in_progress: InProgressGuard,
    },
    Release(u64),
}

/// Holds authorizations until their tokens are dropped. Exits once the manager and all tokens are
/// gone.
fn run_worker(commands: mpsc::Receiver<Command>) {
    let mut authorizations = HashMap::new();
    let mut next_id = 0;

    while let Ok(command) = commands.recv() {
        match command {
            Command::Authorize {
                prompt,
                reply,
                in_progress,
            } => {
                let result = Authorization::new(&prompt)
                    .and_then(|authorization| Ok((authorization.token()?, authorization)))
                    .map_err(AuthorizationError::from);
                drop(in_progress);

                match result {
                    Ok((token, authorization)) => {
                        let id = next_id;
                        next_id += 1;
                        // Keep the authorization only if the caller is still waiting for it.
                        if reply.send(Ok((id, token))).is_ok() {
                            authorizations.insert(id, authorization);
                        }
                    }
                    Err(err) => {
                        let _ = reply.send(Err(err));
                    }
                }
            }
            Command::Release(id) => {
                authorizations.remove(&id);
            }
        }
    }
}

/// Clears the in-progress flag when dropped, including when the request never reaches the worker.
struct InProgressGuard(Arc<AtomicBool>);

impl Drop for InProgressGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
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

    #[error("Another authorization is in progress")]
    InProgress,

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
