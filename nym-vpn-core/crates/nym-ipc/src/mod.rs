// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod client;
mod error;
#[cfg(windows)]
mod named_pipe;
pub mod server;
#[cfg(unix)]
mod uds;
