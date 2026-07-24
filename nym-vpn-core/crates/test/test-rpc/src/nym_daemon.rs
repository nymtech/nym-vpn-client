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

/// Tunnel type carried on `ObservedTunnelState::Connected` (guest-local UDS observation).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObservedTunnelType {
    Mixnet,
    Wireguard,
}

/// Tunnel state discriminant observed via guest-local daemon UDS (tarpc), not serial gRPC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObservedTunnelState {
    Connected { tunnel_type: ObservedTunnelType },
    Disconnected,
    Connecting,
    Disconnecting,
    Offline,
    Error(String),
}

/// Account controller discriminant observed via guest-local daemon UDS (tarpc).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObservedAccountState {
    Offline,
    Syncing,
    LoggedOut,
    ReadyToConnect,
    Decentralised,
    PendingSubscription,
    Error(String),
}

#[cfg(test)]
mod observed_contract_tests {
    use super::{ObservedAccountState, ObservedTunnelState, ObservedTunnelType};

    #[test]
    fn observed_tunnel_state_roundtrips() {
        let samples = [
            ObservedTunnelState::Connected {
                tunnel_type: ObservedTunnelType::Mixnet,
            },
            ObservedTunnelState::Connected {
                tunnel_type: ObservedTunnelType::Wireguard,
            },
            ObservedTunnelState::Disconnected,
            ObservedTunnelState::Connecting,
            ObservedTunnelState::Disconnecting,
            ObservedTunnelState::Offline,
            ObservedTunnelState::Error("reason".into()),
        ];
        for sample in samples {
            let bytes = serde_json::to_vec(&sample).expect("serialize");
            let decoded: ObservedTunnelState = serde_json::from_slice(&bytes).expect("deserialize");
            assert_eq!(decoded, sample);
        }
    }

    #[test]
    fn observed_account_state_roundtrips() {
        let samples = [
            ObservedAccountState::Offline,
            ObservedAccountState::Syncing,
            ObservedAccountState::LoggedOut,
            ObservedAccountState::ReadyToConnect,
            ObservedAccountState::Decentralised,
            ObservedAccountState::PendingSubscription,
            ObservedAccountState::Error("reason".into()),
        ];
        for sample in samples {
            let bytes = serde_json::to_vec(&sample).expect("serialize");
            let decoded: ObservedAccountState =
                serde_json::from_slice(&bytes).expect("deserialize");
            assert_eq!(decoded, sample);
        }
    }
}
