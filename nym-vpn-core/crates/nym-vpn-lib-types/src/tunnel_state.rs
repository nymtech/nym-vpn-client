// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::connection_data::{
    ConnectionData, EstablishConnectionData, EstablishConnectionState, TunnelConnectionData,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TunnelType {
    Mixnet,
    Wireguard,
}

impl TunnelType {
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Mixnet => "mix",
            Self::Wireguard => "wg",
        }
    }
}

/// Public enum describing the tunnel state
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TunnelState {
    /// Tunnel is disconnected and network connectivity is available.
    Disconnected,

    /// Tunnel connection is being established.
    Connecting {
        retry_attempt: u32,
        state: EstablishConnectionState,
        tunnel_type: TunnelType,
        connection_data: Option<EstablishConnectionData>,
    },

    /// Tunnel is connected.
    Connected { connection_data: ConnectionData },

    /// Tunnel is disconnecting.
    Disconnecting {
        after_disconnect: ActionAfterDisconnect,
    },

    /// Tunnel is disconnected due to failure.
    Error(ErrorStateReason),

    /// Tunnel is disconnected, network connectivity is unavailable.
    Offline {
        /// Whether tunnel will be reconnected upon gaining the network connectivity.
        reconnect: bool,
    },
}

impl std::fmt::Display for TunnelState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => f.write_str("Disconnected"),
            Self::Connecting {
                retry_attempt,
                state,
                tunnel_type,
                connection_data,
            } => match connection_data {
                Some(connection_data) => match connection_data.tunnel {
                    Some(TunnelConnectionData::Mixnet(ref data)) => {
                        write!(
                            f,
                            "Connecting {} to {} [{}] → {} [{}], {}, try #{}",
                            tunnel_type.short_name(),
                            data.entry_ip,
                            data.nym_address.gateway_id(),
                            data.exit_ip,
                            data.exit_ipr.gateway_id(),
                            state,
                            retry_attempt,
                        )
                    }
                    Some(TunnelConnectionData::Wireguard(ref data)) => {
                        write!(
                            f,
                            "Connecting {} to {} [{}] → {} [{}], {}, try #{}",
                            tunnel_type.short_name(),
                            data.entry.endpoint,
                            connection_data.entry_gateway.id,
                            data.exit.endpoint,
                            connection_data.exit_gateway.id,
                            state,
                            retry_attempt,
                        )
                    }
                    None => {
                        write!(
                            f,
                            "Connecting {} [{}] → [{}], {}, try #{}",
                            tunnel_type.short_name(),
                            connection_data.entry_gateway.id,
                            connection_data.exit_gateway.id,
                            state,
                            retry_attempt,
                        )
                    }
                },
                None => write!(
                    f,
                    "Connecting {}, {}, try #{}",
                    tunnel_type.short_name(),
                    state,
                    retry_attempt
                ),
            },
            Self::Connected { connection_data } => match connection_data.tunnel {
                TunnelConnectionData::Mixnet(ref data) => {
                    write!(
                        f,
                        "Connected {} to {} [{}] → {} [{}]",
                        connection_data.tunnel.tunnel_type().short_name(),
                        data.entry_ip,
                        data.nym_address.gateway_id(),
                        data.exit_ip,
                        data.exit_ipr.gateway_id(),
                    )
                }
                TunnelConnectionData::Wireguard(ref data) => {
                    write!(
                        f,
                        "Connected {} to {} [{}] → {} [{}]",
                        connection_data.tunnel.tunnel_type().short_name(),
                        data.entry.endpoint,
                        connection_data.entry_gateway.id,
                        data.exit.endpoint,
                        connection_data.exit_gateway.id,
                    )
                }
            },
            Self::Disconnecting { after_disconnect } => match after_disconnect {
                ActionAfterDisconnect::Nothing => f.write_str("Disconnecting"),
                ActionAfterDisconnect::Reconnect => f.write_str("Disconnecting to reconnect"),
                ActionAfterDisconnect::Error => f.write_str("Disconnecting because of an error"),
                ActionAfterDisconnect::Offline => {
                    f.write_str("Disconnecting because device is offline")
                }
            },
            Self::Error(reason) => {
                write!(f, "Error state: {reason:?}")
            }
            Self::Offline { reconnect } => {
                if *reconnect {
                    write!(f, "Offline, auto-connect once back online")
                } else {
                    write!(f, "Offline")
                }
            }
        }
    }
}

/// Public enum describing action to perform after disconnect
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActionAfterDisconnect {
    /// Do nothing after disconnect
    Nothing,

    /// Reconnect after disconnect
    Reconnect,

    /// Enter offline after disconnect
    Offline,

    /// Enter error state
    Error,
}

#[derive(Debug, Clone, Eq, PartialEq, strum_macros::Display)]
pub enum ErrorStateReason {
    /// Failure to set firewall policy.
    SetFirewallPolicy,

    /// Failure to configure routing.
    SetRouting,

    /// Failure to configure dns.
    SetDns,

    /// Failure to configure tunnel device.
    TunDevice,

    /// Failure to configure packet tunnel provider (iOS and Android only)
    TunnelProvider,

    /// IPv6 is disabled in the system.
    Ipv6Unavailable,

    /// Same entry and exit gateway are unsupported.
    SameEntryAndExitGateway,

    /// Failure to select the entry gateway that meets the performance requirements.
    PerformantEntryGatewayUnavailable,

    /// Failure to select the exit gateway that meets the performance requirements.
    PerformantExitGatewayUnavailable,

    /// Invalid country set for entry gateway
    InvalidEntryGatewayCountry,

    /// Invalid country set for exit gateway
    InvalidExitGatewayCountry,

    /// Entry gateway is not responding or responding badly to a bandwidth
    /// increase request, causing credential waste
    CredentialWastedOnEntryGateway,

    /// Exit gateway is not responding or responding badly to a bandwidth
    /// increase request, causing credential waste
    CredentialWastedOnExitGateway,

    /// Bandwidth Exceeded
    BandwidthExceeded,

    /// Account status is not "Active"
    InactiveAccount,

    /// Inactive Subscription
    InactiveSubscription,

    /// Max device numbers reached
    MaxDevicesReached,

    /// Device time is off by too much, Zk-nyms use will fail
    DeviceTimeOutOfSync,

    /// Device is logged out
    DeviceLoggedOut,

    /// Program errors that must not happen.
    Internal(String),
}

impl ErrorStateReason {
    /// Returns true if block reason indicates that filtering resolver cannot be configured.
    #[cfg(target_os = "macos")]
    pub fn prevents_filtering_resolver(&self) -> bool {
        matches!(self, ErrorStateReason::SetDns)
    }
}
