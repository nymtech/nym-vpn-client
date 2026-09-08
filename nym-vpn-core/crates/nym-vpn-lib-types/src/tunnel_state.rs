// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

use super::connection_data::{
    ConnectionData, EstablishConnectionData, EstablishConnectionState, TunnelConnectionData,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
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
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
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

impl TunnelState {
    pub fn is_error_state(&self) -> bool {
        matches!(self, Self::Error(_))
    }
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
                            "Connecting {} to {} [{}] → {} [{}]",
                            tunnel_type.short_name(),
                            data.entry.endpoint,
                            connection_data.entry_gateway.id,
                            data.exit.endpoint,
                            connection_data.exit_gateway.id,
                        )?;

                        if let Some(bridge_addr) = data.entry_bridge_addr.as_ref() {
                            write!(f, " via bridge {}", bridge_addr.remote_addr,)?;
                        }

                        write!(f, ", {state}, try #{retry_attempt}")
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
                    )?;

                    if let Some(bridge_addr) = data.entry_bridge_addr.as_ref() {
                        write!(f, " via bridge {}", bridge_addr.remote_addr)
                    } else {
                        Ok(())
                    }
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
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
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

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
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

    /// Failure to select any entry gateway after trying all performance tiers.
    PerformantEntryGatewayUnavailable,

    /// Failure to select any exit gateway after trying all performance tiers.
    PerformantExitGatewayUnavailable,

    /// Invalid identity set for entry gateway
    InvalidEntryGatewayIdentity,

    /// Invalid identity set for exit gateway
    InvalidExitGatewayIdentity,

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

    /// Failed to fetch a credential from the credential source.
    CredentialFetchingFailed,

    /// No credential is available and none is being fetched.
    NoCredentialAvailable,

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

    /// Split tunnel needs full disk permissions (macOS only)
    NeedFullDiskPermissions,

    /// Internal split tunnel error
    SplitTunnel,

    /// Gateway pair can be found if user agrees to relax the gateway independence criteria
    NeedsRelaxedIndependenceCriteria,

    /// Selector chosen needs device location data
    NeedsDeviceLocation,

    /// Gave up connecting after exhausting the maximum number of reconnect attempts.
    ConnectionAttemptsExceeded,

    /// Program errors that must not happen.
    Internal(String),
}

impl std::fmt::Display for ErrorStateReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetFirewallPolicy => f.write_str("SetFirewallPolicy"),
            Self::SetRouting => f.write_str("SetRouting"),
            Self::SetDns => f.write_str("SetDns"),
            Self::TunDevice => f.write_str("TunDevice"),
            Self::TunnelProvider => f.write_str("TunnelProvider"),
            Self::Ipv6Unavailable => f.write_str("Ipv6Unavailable"),
            Self::SameEntryAndExitGateway => f.write_str("SameEntryAndExitGateway"),
            Self::PerformantEntryGatewayUnavailable => {
                f.write_str("PerformantEntryGatewayUnavailable")
            }
            Self::PerformantExitGatewayUnavailable => {
                f.write_str("PerformantExitGatewayUnavailable")
            }
            Self::InvalidEntryGatewayIdentity => f.write_str("InvalidEntryGatewayIdentity"),
            Self::InvalidExitGatewayIdentity => f.write_str("InvalidExitGatewayIdentity"),
            Self::InvalidEntryGatewayCountry => f.write_str("InvalidEntryGatewayCountry"),
            Self::InvalidExitGatewayCountry => f.write_str("InvalidExitGatewayCountry"),
            Self::CredentialWastedOnEntryGateway => f.write_str("CredentialWastedOnEntryGateway"),
            Self::CredentialWastedOnExitGateway => f.write_str("CredentialWastedOnExitGateway"),
            Self::BandwidthExceeded => f.write_str("BandwidthExceeded"),
            Self::CredentialFetchingFailed => f.write_str("CredentialFetchingFailed"),
            Self::NoCredentialAvailable => f.write_str("NoCredentialAvailable"),
            Self::InactiveAccount => f.write_str("InactiveAccount"),
            Self::InactiveSubscription => f.write_str("InactiveSubscription"),
            Self::MaxDevicesReached => f.write_str("MaxDevicesReached"),
            Self::DeviceTimeOutOfSync => f.write_str("DeviceTimeOutOfSync"),
            Self::DeviceLoggedOut => f.write_str("DeviceLoggedOut"),
            Self::NeedFullDiskPermissions => f.write_str("NeedFullDiskPermissions"),
            Self::SplitTunnel => f.write_str("SplitTunnel"),
            Self::NeedsRelaxedIndependenceCriteria => {
                f.write_str("NeedsRelaxedIndependenceCriteria")
            }
            Self::NeedsDeviceLocation => f.write_str("NeedsDeviceLocation"),
            Self::ConnectionAttemptsExceeded => f.write_str("ConnectionAttemptsExceeded"),
            Self::Internal(str) => write!(f, "Internal({str})"),
        }
    }
}

impl ErrorStateReason {
    /// Returns true if block reason indicates that filtering resolver cannot be configured.
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub fn prevents_filtering_resolver(&self) -> bool {
        matches!(self, ErrorStateReason::SetDns)
    }

    pub fn suggests_running_diagnostics(&self) -> bool {
        matches!(
            self,
            Self::SetDns
                | Self::SetRouting
                | Self::TunDevice
                | Self::TunnelProvider
                | Self::PerformantEntryGatewayUnavailable
                | Self::PerformantExitGatewayUnavailable
                | Self::CredentialWastedOnEntryGateway
                | Self::CredentialWastedOnExitGateway
                | Self::Internal(_)
        )
    }
}
