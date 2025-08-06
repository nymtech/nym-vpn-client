// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use crate::{
    AccountControllerErrorStateReason, RequestZkNymErrorReason, VpnApiErrorResponse,
    account::VpnApiError,
};

use super::connection_data::{ConnectionData, TunnelConnectionData};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TunnelType {
    Mixnet,
    Wireguard,
}

/// Public enum describing the tunnel state
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TunnelState {
    /// Tunnel is disconnected and network connectivity is available.
    Disconnected,

    /// Tunnel connection is being established.
    Connecting {
        retry_attempt: u32,
        connection_data: Option<ConnectionData>,
    },

    /// Tunnel is connected.
    Connected { connection_data: ConnectionData },

    /// Tunnel is disconnecting.
    Disconnecting {
        after_disconnect: ActionAfterDisconnect,
    },

    /// Tunnel is disconnected due to failure.
    Error(ClientErrorReason),

    /// Tunnel is disconnected, network connectivity is unavailable.
    Offline {
        /// Whether tunnel will be reconnected upon gaining the network connectivity.
        reconnect: bool,
    },
}

impl fmt::Display for TunnelState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => f.write_str("Disconnected"),
            Self::Connecting {
                retry_attempt,
                connection_data,
            } => match connection_data {
                Some(connection_data) => match connection_data.tunnel {
                    TunnelConnectionData::Mixnet(ref data) => {
                        write!(
                            f,
                            "Connecting mixnet tunnel to {} → {} (entry: {} → exit: {}), attempt {}",
                            data.entry_ip,
                            data.exit_ip,
                            data.nym_address.gateway_id(),
                            data.exit_ipr.gateway_id(),
                            retry_attempt
                        )
                    }
                    TunnelConnectionData::Wireguard(ref data) => {
                        write!(
                            f,
                            "Connecting wireguard tunnel to {} → {} (entry: {} → exit: {}), attempt {}",
                            data.entry.endpoint,
                            data.exit.endpoint,
                            connection_data.entry_gateway.id,
                            connection_data.exit_gateway.id,
                            retry_attempt
                        )
                    }
                },
                None => write!(f, "Connecting, attempt {retry_attempt}"),
            },
            Self::Connected { connection_data } => match connection_data.tunnel {
                TunnelConnectionData::Mixnet(ref data) => {
                    write!(
                        f,
                        "Connected mixnet tunnel to {} → {} (entry: {} → exit: {})",
                        data.entry_ip,
                        data.exit_ip,
                        data.nym_address.gateway_id(),
                        data.exit_ipr.gateway_id(),
                    )
                }
                TunnelConnectionData::Wireguard(ref data) => {
                    write!(
                        f,
                        "Connected wireguard tunnel {} → {} (entry: {} → exit: {})",
                        data.entry.endpoint,
                        data.exit.endpoint,
                        connection_data.entry_gateway.id,
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
    /// Issues related to firewall configuration.
    Firewall,

    /// Failure to configure routing.
    Routing,

    /// Failure to configure dns.
    SetDns,

    /// Failure to configure tunnel device.
    TunDevice,

    /// Failure to configure packet tunnel provider.
    TunnelProvider,

    /// Failure to resolve API addresses.
    ResolveGatewayAddrs,

    /// Failure to start local dns resolver.
    StartLocalDnsResolver,

    /// Same entry and exit gateway are unsupported.
    SameEntryAndExitGateway,

    /// Invalid country set for entry gateway
    InvalidEntryGatewayCountry,

    /// Invalid country set for exit gateway
    InvalidExitGatewayCountry,

    /// Gateway is not responding or responding badly to a bandwidth
    /// increase request, causing credential waste
    BadBandwidthIncrease,

    /// Failure to duplicate tunnel file descriptor.
    DuplicateTunFd,

    /// Failure to create mixnet storage.
    CreateMixnetStorage,

    /// IPv6 is disabled in the system.
    Ipv6Unavailable,

    /// Program errors that must not happen.
    Internal(String),

    /// Account controller is in error state.
    AccountControllerError(AccountControllerErrorStateReason),

    /// Account controller is offline
    AccountControllerOffline,

    /// Account controller is logged out
    AccountControllerLoggedOut,
}

impl ErrorStateReason {
    /// Returns true if block reason indicates that filtering resolver cannot be configured.
    #[cfg(target_os = "macos")]
    pub fn prevents_filtering_resolver(&self) -> bool {
        matches!(self, ErrorStateReason::SetDns)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, strum_macros::Display)]
pub enum ClientErrorReason {
    Firewall,
    Routing,
    SameEntryAndExitGateway,
    InvalidEntryGatewayCountry,
    InvalidExitGatewayCountry,
    Dns(Option<String>),
    Api(Option<String>),
    CreateMixnetStorage,
    Ipv6Unavailable,
    Internal(Option<String>),
    AccountControl(Option<String>),
}

impl From<ErrorStateReason> for ClientErrorReason {
    fn from(value: ErrorStateReason) -> Self {
        match value {
            ErrorStateReason::CreateMixnetStorage => Self::CreateMixnetStorage,
            ErrorStateReason::SameEntryAndExitGateway => Self::SameEntryAndExitGateway,
            ErrorStateReason::InvalidEntryGatewayCountry => Self::InvalidEntryGatewayCountry,
            ErrorStateReason::InvalidExitGatewayCountry => Self::InvalidExitGatewayCountry,
            ErrorStateReason::BadBandwidthIncrease => Self::Api(Some(value.to_string())),
            ErrorStateReason::Firewall => Self::Firewall,
            ErrorStateReason::TunDevice
            | ErrorStateReason::TunnelProvider
            | ErrorStateReason::DuplicateTunFd => Self::Internal(Some(value.to_string())),
            ErrorStateReason::Internal(message) => Self::Internal(Some(message)),
            ErrorStateReason::Routing => Self::Routing,
            ErrorStateReason::ResolveGatewayAddrs => Self::Dns(Some(value.to_string())),
            ErrorStateReason::StartLocalDnsResolver => Self::Dns(Some(value.to_string())),
            ErrorStateReason::SetDns => Self::Dns(Some(value.to_string())),
            ErrorStateReason::Ipv6Unavailable => Self::Ipv6Unavailable,
            ErrorStateReason::AccountControllerError(reason) => {
                Self::AccountControl(Some(reason.to_string()))
            }
            ErrorStateReason::AccountControllerOffline => {
                Self::AccountControl(Some("offline".into()))
            }
            ErrorStateReason::AccountControllerLoggedOut => {
                Self::AccountControl(Some("logged out".into()))
            }
        }
    }
}

impl From<RequestZkNymErrorReason> for ClientErrorReason {
    fn from(error: RequestZkNymErrorReason) -> Self {
        match error {
            RequestZkNymErrorReason::VpnApi(e) => e.into(),
            RequestZkNymErrorReason::UnexpectedVpnApiResponse(message) => Self::Api(Some(message)),
            reason => Self::Internal(Some(reason.to_string())),
        }
    }
}

impl From<VpnApiError> for ClientErrorReason {
    fn from(error: VpnApiError) -> Self {
        match error {
            VpnApiError::Response(e) => e.into(),
            VpnApiError::StatusCode { .. } => Self::Api(Some(error.to_string())),
            VpnApiError::Timeout(..) => Self::Api(Some(error.to_string())),
        }
    }
}

impl From<VpnApiErrorResponse> for ClientErrorReason {
    fn from(error: VpnApiErrorResponse) -> Self {
        let message = match error.message_id {
            None => error.message,
            Some(id) => format!("{}, ID [{}]", error.message, id),
        };
        Self::Api(Some(message))
    }
}
