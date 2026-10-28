// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

use crate::SelectorFallbackState;

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
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
#[cfg_attr(feature = "cbor", cbor(index_only))]
pub enum TunnelType {
    #[cfg_attr(feature = "cbor", n(0))]
    Mixnet,
    #[cfg_attr(feature = "cbor", n(1))]
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
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub enum TunnelState {
    /// Tunnel is disconnected and network connectivity is available.
    #[cfg_attr(feature = "cbor", n(0))]
    Disconnected,

    /// Tunnel connection is being established.
    #[cfg_attr(feature = "cbor", n(1))]
    Connecting {
        #[cfg_attr(feature = "cbor", n(0))]
        retry_attempt: u32,
        #[cfg_attr(feature = "cbor", n(1))]
        state: EstablishConnectionState,
        #[cfg_attr(feature = "cbor", n(2))]
        selector_fallback_state: SelectorFallbackState,
        #[cfg_attr(feature = "cbor", n(3))]
        tunnel_type: TunnelType,
        #[cfg_attr(feature = "cbor", n(4))]
        connection_data: Option<EstablishConnectionData>,
    },

    /// Tunnel is connected.
    #[cfg_attr(feature = "cbor", n(2))]
    Connected {
        #[cfg_attr(feature = "cbor", n(0))]
        connection_data: ConnectionData,
    },

    /// Tunnel is disconnecting.
    #[cfg_attr(feature = "cbor", n(3))]
    Disconnecting {
        #[cfg_attr(feature = "cbor", n(0))]
        after_disconnect: ActionAfterDisconnect,
    },

    /// Tunnel is disconnected due to failure.
    #[cfg_attr(feature = "cbor", n(4))]
    Error(#[cfg_attr(feature = "cbor", n(0))] ErrorStateReason),

    /// Tunnel is disconnected, network connectivity is unavailable.
    #[cfg_attr(feature = "cbor", n(5))]
    Offline {
        /// Whether tunnel will be reconnected upon gaining the network connectivity.
        #[cfg_attr(feature = "cbor", n(0))]
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
                selector_fallback_state,
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
                    "Connecting {}, {}, fallback entry: {}, fallback exit: {}, try #{}",
                    tunnel_type.short_name(),
                    state,
                    selector_fallback_state.entry_fallback,
                    selector_fallback_state.exit_fallback,
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
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
#[cfg_attr(feature = "cbor", cbor(index_only))]
pub enum ActionAfterDisconnect {
    /// Do nothing after disconnect
    #[cfg_attr(feature = "cbor", n(0))]
    Nothing,

    /// Reconnect after disconnect
    #[cfg_attr(feature = "cbor", n(1))]
    Reconnect,

    /// Enter offline after disconnect
    #[cfg_attr(feature = "cbor", n(2))]
    Offline,

    /// Enter error state
    #[cfg_attr(feature = "cbor", n(3))]
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
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub enum ErrorStateReason {
    /// Failure to set firewall policy.
    #[cfg_attr(feature = "cbor", n(0))]
    SetFirewallPolicy,

    /// Failure to configure routing.
    #[cfg_attr(feature = "cbor", n(1))]
    SetRouting,

    /// Failure to configure dns.
    #[cfg_attr(feature = "cbor", n(2))]
    SetDns,

    /// Failure to configure tunnel device.
    #[cfg_attr(feature = "cbor", n(3))]
    TunDevice,

    /// Failure to configure packet tunnel provider (iOS and Android only)
    #[cfg_attr(feature = "cbor", n(4))]
    TunnelProvider,

    /// Failure to configure local DNS resolver
    #[cfg_attr(feature = "cbor", n(5))]
    SetLocalDnsResolverConfig,

    /// IPv6 is disabled in the system.
    #[cfg_attr(feature = "cbor", n(6))]
    Ipv6Unavailable,

    /// Same entry and exit gateway are unsupported.
    #[cfg_attr(feature = "cbor", n(7))]
    SameEntryAndExitGateway,

    /// Failure to select any entry gateway after trying all performance tiers.
    #[cfg_attr(feature = "cbor", n(8))]
    PerformantEntryGatewayUnavailable,

    /// Failure to select any exit gateway after trying all performance tiers.
    #[cfg_attr(feature = "cbor", n(9))]
    PerformantExitGatewayUnavailable,

    /// Invalid identity set for entry gateway
    #[cfg_attr(feature = "cbor", n(10))]
    InvalidEntryGatewayIdentity,

    /// Invalid identity set for exit gateway
    #[cfg_attr(feature = "cbor", n(11))]
    InvalidExitGatewayIdentity,

    /// Invalid country set for entry gateway
    #[cfg_attr(feature = "cbor", n(12))]
    InvalidEntryGatewayCountry,

    /// Invalid country set for exit gateway
    #[cfg_attr(feature = "cbor", n(13))]
    InvalidExitGatewayCountry,

    /// Entry gateway is not responding or responding badly to a bandwidth
    /// increase request, causing credential waste
    #[cfg_attr(feature = "cbor", n(14))]
    CredentialWastedOnEntryGateway,

    /// Exit gateway is not responding or responding badly to a bandwidth
    /// increase request, causing credential waste
    #[cfg_attr(feature = "cbor", n(15))]
    CredentialWastedOnExitGateway,

    /// Bandwidth Exceeded
    #[cfg_attr(feature = "cbor", n(16))]
    BandwidthExceeded,

    /// Failed to fetch a credential from the credential source.
    #[cfg_attr(feature = "cbor", n(17))]
    CredentialFetchingFailed,

    /// No credential is available and none is being fetched.
    #[cfg_attr(feature = "cbor", n(18))]
    NoCredentialAvailable,

    /// Account status is not "Active"
    #[cfg_attr(feature = "cbor", n(19))]
    InactiveAccount,

    /// Inactive Subscription
    #[cfg_attr(feature = "cbor", n(20))]
    InactiveSubscription,

    /// Max device numbers reached
    #[cfg_attr(feature = "cbor", n(21))]
    MaxDevicesReached,

    /// Device is logged out
    #[cfg_attr(feature = "cbor", n(22))]
    DeviceLoggedOut,

    /// Split tunnel needs full disk permissions (macOS only)
    #[cfg_attr(feature = "cbor", n(23))]
    NeedFullDiskPermissions,

    /// Internal split tunnel error
    #[cfg_attr(feature = "cbor", n(24))]
    SplitTunnel,

    /// Gateway pair can be found if user agrees to relax the gateway independence criteria
    #[cfg_attr(feature = "cbor", n(25))]
    NeedsRelaxedIndependenceCriteria,

    /// Selector chosen needs device location data
    #[cfg_attr(feature = "cbor", n(26))]
    NeedsDeviceLocation,

    /// Gave up connecting after exhausting the maximum number of reconnect attempts.
    #[cfg_attr(feature = "cbor", n(27))]
    ConnectionAttemptsExceeded,

    /// Program errors that must not happen.
    #[cfg_attr(feature = "cbor", n(28))]
    Internal(#[cfg_attr(feature = "cbor", n(0))] String),
}

impl std::fmt::Display for ErrorStateReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetFirewallPolicy => f.write_str("SetFirewallPolicy"),
            Self::SetRouting => f.write_str("SetRouting"),
            Self::SetDns => f.write_str("SetDns"),
            Self::TunDevice => f.write_str("TunDevice"),
            Self::TunnelProvider => f.write_str("TunnelProvider"),
            Self::SetLocalDnsResolverConfig => f.write_str("SetLocalDnsResolverConfig"),
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
    #[cfg(target_os = "macos")]
    pub fn is_dns_error(&self) -> bool {
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
