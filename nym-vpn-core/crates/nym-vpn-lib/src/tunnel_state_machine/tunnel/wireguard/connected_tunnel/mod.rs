// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod desktop;

#[cfg(any(target_os = "ios", target_os = "android"))]
mod mobile;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub use desktop::{ConnectedTunnel, TunnelHandle};

#[cfg(any(target_os = "ios", target_os = "android"))]
pub use mobile::{ConnectedTunnel, TunnelHandle};
