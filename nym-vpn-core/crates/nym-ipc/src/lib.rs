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
#[cfg(target_os = "linux")]
#[cfg(feature = "daemon")]
mod uds;
#[cfg(target_os = "macos")]
mod xpc;
