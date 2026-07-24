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

/// Payload-insensitive tunnel-state selector used to wait for a target discriminant
/// on the guest without sending the full state (or its message) as a matcher.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObservedTunnelStateKind {
    Connected,
    Disconnected,
    Connecting,
    Disconnecting,
    Offline,
    Error,
}

impl ObservedTunnelStateKind {
    pub fn matches(self, state: &ObservedTunnelState) -> bool {
        matches!(
            (self, state),
            (
                ObservedTunnelStateKind::Connected,
                ObservedTunnelState::Connected { .. }
            ) | (
                ObservedTunnelStateKind::Disconnected,
                ObservedTunnelState::Disconnected
            ) | (
                ObservedTunnelStateKind::Connecting,
                ObservedTunnelState::Connecting
            ) | (
                ObservedTunnelStateKind::Disconnecting,
                ObservedTunnelState::Disconnecting
            ) | (
                ObservedTunnelStateKind::Offline,
                ObservedTunnelState::Offline
            ) | (
                ObservedTunnelStateKind::Error,
                ObservedTunnelState::Error(_)
            )
        )
    }
}

/// Payload-insensitive account-state selector (mirrors [`ObservedTunnelStateKind`]).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObservedAccountStateKind {
    Offline,
    Syncing,
    LoggedOut,
    ReadyToConnect,
    Decentralised,
    PendingSubscription,
    Error,
}

impl ObservedAccountStateKind {
    pub fn matches(self, state: &ObservedAccountState) -> bool {
        matches!(
            (self, state),
            (
                ObservedAccountStateKind::Offline,
                ObservedAccountState::Offline
            ) | (
                ObservedAccountStateKind::Syncing,
                ObservedAccountState::Syncing
            ) | (
                ObservedAccountStateKind::LoggedOut,
                ObservedAccountState::LoggedOut
            ) | (
                ObservedAccountStateKind::ReadyToConnect,
                ObservedAccountState::ReadyToConnect
            ) | (
                ObservedAccountStateKind::Decentralised,
                ObservedAccountState::Decentralised
            ) | (
                ObservedAccountStateKind::PendingSubscription,
                ObservedAccountState::PendingSubscription
            ) | (
                ObservedAccountStateKind::Error,
                ObservedAccountState::Error(_)
            )
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WaitOutcome<S> {
    Reached(S),
    TimedOut { last_observed: Option<S> },
}

#[cfg(test)]
mod observed_contract_tests {
    use super::{
        ObservedAccountState, ObservedAccountStateKind, ObservedTunnelState,
        ObservedTunnelStateKind, ObservedTunnelType, WaitOutcome,
    };

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

    #[test]
    fn tunnel_kind_matches_only_its_discriminant() {
        assert!(
            ObservedTunnelStateKind::Connected.matches(&ObservedTunnelState::Connected {
                tunnel_type: ObservedTunnelType::Wireguard,
            })
        );
        assert!(
            ObservedTunnelStateKind::Connected.matches(&ObservedTunnelState::Connected {
                tunnel_type: ObservedTunnelType::Mixnet,
            })
        );
        assert!(!ObservedTunnelStateKind::Connected.matches(&ObservedTunnelState::Connecting));
        assert!(ObservedTunnelStateKind::Error.matches(&ObservedTunnelState::Error("any".into())));
        assert!(
            !ObservedTunnelStateKind::Disconnected.matches(&ObservedTunnelState::Disconnecting)
        );
        assert!(ObservedTunnelStateKind::Disconnected.matches(&ObservedTunnelState::Disconnected));
    }

    #[test]
    fn account_kind_matches_only_its_discriminant() {
        assert!(
            ObservedAccountStateKind::ReadyToConnect.matches(&ObservedAccountState::ReadyToConnect)
        );
        assert!(!ObservedAccountStateKind::ReadyToConnect.matches(&ObservedAccountState::Syncing));
        assert!(ObservedAccountStateKind::Error.matches(&ObservedAccountState::Error("x".into())));
        assert!(!ObservedAccountStateKind::LoggedOut.matches(&ObservedAccountState::Offline));
    }

    #[test]
    fn wait_outcome_roundtrips() {
        let reached = WaitOutcome::Reached(ObservedTunnelState::Connected {
            tunnel_type: ObservedTunnelType::Mixnet,
        });
        let timed_out: WaitOutcome<ObservedTunnelState> = WaitOutcome::TimedOut {
            last_observed: Some(ObservedTunnelState::Connecting),
        };
        let empty: WaitOutcome<ObservedAccountState> = WaitOutcome::TimedOut {
            last_observed: None,
        };

        for sample in [reached, timed_out] {
            let bytes = serde_json::to_vec(&sample).expect("serialize");
            let decoded: WaitOutcome<ObservedTunnelState> =
                serde_json::from_slice(&bytes).expect("deserialize");
            assert_eq!(decoded, sample);
        }
        let bytes = serde_json::to_vec(&empty).expect("serialize");
        let decoded: WaitOutcome<ObservedAccountState> =
            serde_json::from_slice(&bytes).expect("deserialize");
        assert_eq!(decoded, empty);
    }
}
