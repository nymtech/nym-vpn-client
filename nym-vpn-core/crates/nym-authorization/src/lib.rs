// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "macos")]
mod service;
#[cfg(target_os = "macos")]
mod unlock;

#[cfg(target_os = "macos")]
pub use service::AuthorizationService;
#[cfg(target_os = "macos")]
pub use unlock::UnlockAuthorization;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("User cancelled")]
    UserCancelled,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Prompt contains a NUL byte")]
    InvalidPrompt,

    #[error("Authorization error: {0}")]
    Os(i32),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
