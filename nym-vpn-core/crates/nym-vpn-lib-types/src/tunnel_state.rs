// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use crate::{RequestZkNymError, RequestZkNymErrorReason, VpnApiErrorResponse};

use super::{
    account::{
        register_device::RegisterDeviceError, request_zknym::RequestZkNymSuccess,
        sync_account::SyncAccountError, sync_device::SyncDeviceError,
    },
    connection_data::{ConnectionData, TunnelConnectionData},
};

const MAX_DEVICES_REACHED_MESSAGE_ID: &str =
    "nym-vpn-website.public-api.register-device.max-devices-exceeded";
const SUBSCRIPTION_EXPIRED_MESSAGE_ID: &str =
    "nym-vpn-website.public-api.device.zk-nym.request_failed.no_active_subscription";
const BANDWIDTH_LIMIT_REACHED_MESSAGE_ID: &str =
    "nym-vpn-website.public-api.device.zk-nym.request_failed.fair_usage_used_for_month";

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
            Self::Connecting { connection_data } => match connection_data {
                Some(connection_data) => match connection_data.tunnel {
                    TunnelConnectionData::Mixnet(ref data) => {
                        write!(
                            f,
                            "Connecting mixnet tunnel to {} → {} (entry: {} → exit: {})",
                            data.entry_ip,
                            data.exit_ip,
                            data.nym_address.gateway_id(),
                            data.exit_ipr.gateway_id(),
                        )
                    }
                    TunnelConnectionData::Wireguard(ref data) => {
                        write!(
                            f,
                            "Connecting wireguard tunnel to {} → {} (entry: {} → exit: {})",
                            data.entry.endpoint,
                            data.exit.endpoint,
                            connection_data.entry_gateway.id,
                            connection_data.exit_gateway.id,
                        )
                    }
                },
                None => f.write_str("Connecting"),
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
                write!(f, "Error state: {:?}", reason)
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
    Dns,

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

    /// Failure to sync account with the VPN API.
    SyncAccount(SyncAccountError),

    /// Failure to sync device with the VPN API.
    SyncDevice(SyncDeviceError),

    /// Failure to register device with the VPN API.
    RegisterDevice(RegisterDeviceError),

    /// Failure to request a zknym from the VPN API.
    RequestZkNym(RequestZkNymErrorReason),

    /// Zknym ticketbooks were requested, some succeeded and some failed.
    RequestZkNymBundle {
        successes: Vec<RequestZkNymSuccess>,
        failed: Vec<RequestZkNymErrorReason>,
    },

    /// Program errors that must not happen.
    Internal(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ClientErrorReason {
    Firewall,
    Routing,
    SameEntryAndExitGateway,
    InvalidEntryGatewayCountry,
    InvalidExitGatewayCountry,
    MaxDevicesReached,
    BandwidthExceeded,
    SubscriptionExpired,
    Dns(Option<String>),
    Api(Option<String>),
    Internal(Option<String>),
}

impl From<ErrorStateReason> for ClientErrorReason {
    fn from(value: ErrorStateReason) -> Self {
        match value {
            ErrorStateReason::SameEntryAndExitGateway => Self::SameEntryAndExitGateway,
            ErrorStateReason::InvalidEntryGatewayCountry => Self::InvalidEntryGatewayCountry,
            ErrorStateReason::InvalidExitGatewayCountry => Self::InvalidExitGatewayCountry,
            ErrorStateReason::BadBandwidthIncrease => Self::Api(Some(value.to_string())),
            ErrorStateReason::SyncAccount(err) => err.into(),
            ErrorStateReason::SyncDevice(err) => err.into(),
            ErrorStateReason::RegisterDevice(err) => err.into(),
            ErrorStateReason::RequestZkNym(err) => err.into(),
            ErrorStateReason::RequestZkNymBundle {
                successes: _,
                failed,
            } => {
                // Return the first error if it exists, otherwise return a default error
                if let Some(first_error) = failed.first() {
                    ClientErrorReason::from(first_error.clone())
                } else {
                    Self::Api(Some("Empty failure list in RequestZkNymBundle".to_string()))
                }
            }
            ErrorStateReason::Firewall => Self::Firewall,
            ErrorStateReason::TunDevice
            | ErrorStateReason::TunnelProvider
            | ErrorStateReason::DuplicateTunFd => Self::Internal(Some(value.to_string())),
            ErrorStateReason::Internal(message) => Self::Internal(Some(message)),
            ErrorStateReason::Routing => Self::Routing,
            ErrorStateReason::ResolveGatewayAddrs => Self::Dns(Some(value.to_string())),
            ErrorStateReason::StartLocalDnsResolver => Self::Dns(Some(value.to_string())),
            ErrorStateReason::Dns => Self::Dns(Some(value.to_string())),
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

impl From<VpnApiErrorResponse> for ClientErrorReason {
    fn from(error: VpnApiErrorResponse) -> Self {
        match error.message_id.as_ref() {
            Some(id) if id.contains(BANDWIDTH_LIMIT_REACHED_MESSAGE_ID) => Self::BandwidthExceeded,
            Some(id) if id.contains(SUBSCRIPTION_EXPIRED_MESSAGE_ID) => Self::SubscriptionExpired,
            _ => {
                let message = match error.message_id {
                    None => error.message,
                    Some(id) => format!("{}, ID [{}]", error.message, id),
                };
                Self::Api(Some(message))
            }
        }
    }
}

impl From<RegisterDeviceError> for ClientErrorReason {
    fn from(value: RegisterDeviceError) -> Self {
        if value
            .message_id()
            .is_some_and(|id| id.contains(MAX_DEVICES_REACHED_MESSAGE_ID))
        {
            Self::MaxDevicesReached
        } else {
            Self::Api(Some(value.to_string()))
        }
    }
}

impl From<SyncAccountError> for ClientErrorReason {
    fn from(value: SyncAccountError) -> Self {
        match value {
            SyncAccountError::NoAccountStored => Self::Internal(Some(value.to_string())),
            SyncAccountError::SyncAccountEndpointFailure(response) => response.into(),
            SyncAccountError::UnexpectedResponse(message) => Self::Internal(Some(message)),
            SyncAccountError::Internal(message) => Self::Internal(Some(message)),
        }
    }
}
impl From<SyncDeviceError> for ClientErrorReason {
    fn from(value: SyncDeviceError) -> Self {
        match value {
            SyncDeviceError::NoAccountStored => Self::Internal(Some(value.to_string())),
            SyncDeviceError::NoDeviceStored => Self::Internal(Some(value.to_string())),
            SyncDeviceError::SyncDeviceEndpointFailure(response) => response.into(),
            SyncDeviceError::UnexpectedResponse(message) => Self::Internal(Some(message)),
            SyncDeviceError::Internal(message) => Self::Internal(Some(message)),
        }
    }
}

impl From<SyncAccountError> for ErrorStateReason {
    fn from(value: SyncAccountError) -> Self {
        ErrorStateReason::SyncAccount(value)
    }
}

impl From<SyncDeviceError> for ErrorStateReason {
    fn from(value: SyncDeviceError) -> Self {
        ErrorStateReason::SyncDevice(value)
    }
}

impl From<RegisterDeviceError> for ErrorStateReason {
    fn from(value: RegisterDeviceError) -> Self {
        ErrorStateReason::RegisterDevice(value)
    }
}

impl From<RequestZkNymError> for ErrorStateReason {
    fn from(value: RequestZkNymError) -> Self {
        ErrorStateReason::RequestZkNym(value.into())
    }
}
