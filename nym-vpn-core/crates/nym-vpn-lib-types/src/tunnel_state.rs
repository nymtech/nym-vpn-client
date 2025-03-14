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
    Error(ErrorStateReason),

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
