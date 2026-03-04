// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

// VERY IMPORTANT: this socket path is defined in nym-vpnd, and the value here
// always needs to be the same value as in that crate
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub const NYMVPN_SOCKET_PATH: &str = "/var/run/nym-vpn.sock";

#[cfg(windows)]
pub const NYMVPN_SOCKET_PATH: &str = "//./pipe/NymVPN";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Error {
    ConnectError,
    DisconnectError,
    DaemonError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum ServiceStatus {
    NotRunning,
    Running,
}

impl ServiceStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, ServiceStatus::Running)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Verbosity {
    Info,
    Debug,
    Trace,
}
