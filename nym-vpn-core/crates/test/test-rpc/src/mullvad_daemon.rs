// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub const MULLVAD_SOCKET_PATH: &str = "/var/run/mullvad-vpn";
pub const NYMVPN_SOCKET_PATH: &str = "/var/run/nym-vpn.sock";

#[cfg(windows)]
pub const SOCKET_PATH: &str = "//./pipe/Mullvad VPN";

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
