// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod client;
pub mod server;

#[cfg(windows)]
mod named_pipe;

#[cfg(unix)]
mod uds;
