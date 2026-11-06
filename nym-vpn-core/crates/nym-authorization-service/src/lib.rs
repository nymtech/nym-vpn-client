// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Request and verify permissions using Authorization Services on macOS.
//!
//! In a multi-process architecture such as the Nym VPN app, the daemon runs as root outside the
//! user's session. It cannot verify user presence itself, so it cannot tell whether sensitive data
//! is being accessed by the user or by a malicious actor.
//!
//! macOS acts as the broker instead: the authorization daemon (`authd`) has the user
//! authenticated through SecurityAgent, records the granted right and later confirms it.
//!
//! A process in the user's session requests the right with [`Authorization`], which prompts the
//! user for their password or Touch ID, and hands the resulting token over to the daemon.
//! [`AuthorizationService`] then asks `authd` to confirm the right was granted, failing instead of
//! prompting if it was not. The token only refers to state held by `authd`, so it cannot be forged.

#[cfg(target_os = "macos")]
mod authorization;
#[cfg(target_os = "macos")]
mod service;

#[cfg(target_os = "macos")]
pub use authorization::Authorization;
#[cfg(target_os = "macos")]
pub use service::AuthorizationService;

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
