// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod path_monitor;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "ios")]
mod ios;

#[cfg(target_os = "macos")]
pub use macos::{spawn_monitor, ConnectivityHandle};

#[cfg(target_os = "ios")]
pub use ios::{spawn_monitor, ConnectivityHandle};
