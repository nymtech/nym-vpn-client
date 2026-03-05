// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod auth_result;
pub mod client;
#[cfg(feature = "daemon")]
pub mod server;

#[cfg(target_os = "windows")]
#[cfg(feature = "daemon")]
mod named_pipe;

#[cfg(feature = "daemon")]
mod authentication;
#[cfg(feature = "daemon")]
pub use authentication::AuthenticationMaterial;
#[cfg(all(target_os = "macos", feature = "daemon"))]
pub use authentication::SigningRequirements;

#[cfg(any(
    target_os = "linux",
    all(target_os = "macos", debug_assertions, not(feature = "xpc"))
))]
#[cfg(feature = "daemon")]
mod uds;
#[cfg(all(target_os = "macos", any(not(debug_assertions), feature = "xpc")))]
mod xpc;
