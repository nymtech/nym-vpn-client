// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use crate::{RequestZkNymError, RequestZkNymErrorReason};

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

#[derive(Debug, Clone, Eq, PartialEq)]
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
    Dns(String),
    Api(String),
    Internal(String),
}

impl From<ErrorStateReason> for ClientErrorReason {
    fn from(value: ErrorStateReason) -> Self {
        match value {
            ErrorStateReason::SameEntryAndExitGateway => Self::SameEntryAndExitGateway,
            ErrorStateReason::InvalidEntryGatewayCountry => Self::InvalidEntryGatewayCountry,
            ErrorStateReason::InvalidExitGatewayCountry => Self::InvalidExitGatewayCountry,
            ErrorStateReason::BadBandwidthIncrease => Self::Api(value.to_string()),
            ErrorStateReason::SyncAccount(err) => Self::Api(err.to_string()),
            ErrorStateReason::SyncDevice(err) => Self::Api(err.to_string()),
            ErrorStateReason::RegisterDevice(err) => {
                if err
                    .message_id()
                    .is_some_and(|id| id.contains(MAX_DEVICES_REACHED_MESSAGE_ID))
                {
                    Self::MaxDevicesReached
                } else {
                    Self::Api(err.to_string())
                }
            }
            ErrorStateReason::RequestZkNym(err) => match err {
                RequestZkNymErrorReason::VpnApi(e) => match e.message_id.as_ref() {
                    Some(id) if id.contains(BANDWIDTH_LIMIT_REACHED_MESSAGE_ID) => {
                        Self::BandwidthExceeded
                    }
                    Some(id) if id.contains(SUBSCRIPTION_EXPIRED_MESSAGE_ID) => {
                        Self::SubscriptionExpired
                    }
                    _ => Self::Api(e.message),
                },
                RequestZkNymErrorReason::UnexpectedVpnApiResponse(message) => Self::Api(message),
                reason => Self::Internal(reason.to_string()),
            },
            ErrorStateReason::RequestZkNymBundle {
                successes: _,
                failed,
            } => {
                if let Some(RequestZkNymErrorReason::VpnApi(e)) = failed
                    .iter()
                    .find(|e| matches!(e, RequestZkNymErrorReason::VpnApi { .. }))
                {
                    return match e.message_id.as_ref() {
                        Some(id) if id.contains(BANDWIDTH_LIMIT_REACHED_MESSAGE_ID) => {
                            Self::BandwidthExceeded
                        }
                        Some(id) if id.contains(SUBSCRIPTION_EXPIRED_MESSAGE_ID) => {
                            Self::SubscriptionExpired
                        }
                        _ => Self::Api(e.clone().message),
                    };
                }
                if let Some(err) = failed
                    .iter()
                    .find(|e| matches!(e, RequestZkNymErrorReason::UnexpectedVpnApiResponse { .. }))
                {
                    return Self::Api(err.to_string());
                }
                Self::Internal(failed.into_iter().map(|e| e.to_string()).collect())
            }
            ErrorStateReason::Firewall => Self::Firewall,
            ErrorStateReason::TunDevice
            | ErrorStateReason::TunnelProvider
            | ErrorStateReason::DuplicateTunFd
            | ErrorStateReason::Dns => Self::Internal(value.to_string()),
            ErrorStateReason::Internal(message) => Self::Internal(message),
            ErrorStateReason::Routing => Self::Routing,
            ErrorStateReason::ResolveGatewayAddrs => Self::Dns(value.to_string()),
            ErrorStateReason::StartLocalDnsResolver => Self::Dns(value.to_string()),
        }
    }
}

impl fmt::Display for ErrorStateReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorStateReason::Firewall => write!(f, "Firewall"),
            ErrorStateReason::Routing => write!(f, "Routing"),
            ErrorStateReason::Dns => write!(f, "Dns"),
            ErrorStateReason::TunDevice => write!(f, "TunnelDevice"),
            ErrorStateReason::TunnelProvider => write!(f, "TunnelProvider"),
            ErrorStateReason::SameEntryAndExitGateway => write!(f, "SameEntryAndExitGateway"),
            ErrorStateReason::InvalidEntryGatewayCountry => write!(f, "InvalidEntryGatewayCountry"),
            ErrorStateReason::InvalidExitGatewayCountry => write!(f, "InvalidExitGatewayCountry"),
            ErrorStateReason::BadBandwidthIncrease => write!(f, "BadBandwidthIncrease"),
            ErrorStateReason::DuplicateTunFd => write!(f, "DuplicateTunFd"),
            ErrorStateReason::SyncAccount(_) => write!(f, "SyncAccount"),
            ErrorStateReason::SyncDevice(_) => write!(f, "SyncDevice"),
            ErrorStateReason::RegisterDevice(_) => write!(f, "RequestZkNym"),
            ErrorStateReason::RequestZkNym(_) => write!(f, "InvalidExitGatewayCountry"),
            ErrorStateReason::RequestZkNymBundle { .. } => write!(f, "RequestZkNymBundle "),
            ErrorStateReason::Internal(_) => write!(f, "Internal"),
            ErrorStateReason::ResolveGatewayAddrs => write!(f, "ResolveGatewayAddrs"),
            ErrorStateReason::StartLocalDnsResolver => write!(f, "StartLocalDnsResolver"),
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
