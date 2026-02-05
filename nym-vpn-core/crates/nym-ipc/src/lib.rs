// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
mod auth_result;
pub mod client;
#[cfg(feature = "daemon")]
pub mod server;

#[cfg(windows)]
#[cfg(feature = "daemon")]
mod named_pipe;

#[cfg(unix)]
#[cfg(feature = "daemon")]
mod authentication;
#[cfg(unix)]
#[cfg(feature = "daemon")]
mod uds;
