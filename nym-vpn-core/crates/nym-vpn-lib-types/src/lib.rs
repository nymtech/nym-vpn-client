// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Types shared between nym-vpn-lib and other crates in the workspace.

mod account;
mod bridges;
mod connection_data;
mod gateway;
mod log_path;
mod rpc_requests;
mod service;
mod tunnel_event;
mod tunnel_state;

pub use account::{
    AccountCommandError, RegisterAccountResponse, VpnApiError, VpnApiErrorResponse,
    controller_error::{
        AccountControllerError, ErrorStateReason as AccountControllerErrorStateReason,
    },
    controller_event::AccountControllerEvent,
    controller_state::AccountControllerState,
    request_zknym::{RequestZkNymError, RequestZkNymErrorReason, RequestZkNymSuccess},
    ticketbooks::AvailableTickets,
};
pub use bridges::{BridgeInformation, BridgeParameters, QuicClientOptions};
pub use connection_data::{
    ConnectionData, EstablishConnectionData, EstablishConnectionState, GatewayId,
    MixnetConnectionData, NymAddress, TunnelConnectionData, WireguardConnectionData, WireguardNode,
};
pub use gateway::{
    Asn, AsnKind, Country, Entry, Exit, Gateway, Location, Performance, Probe, ProbeOutcome, Score,
};
pub use log_path::LogPath;
pub use nym_validator_client::nyxd::Coin;
pub use rpc_requests::{
    AccountBalanceResponse, AccountCommandResponse, ConnectArgs, ConnectOptions,
    DecentralisedObtainTicketbooksRequest, ListGatewaysOptions, StoreAccountRequest,
};
pub use service::{TargetState, VpnServiceConfig, VpnServiceInfo};
pub use tunnel_event::{
    BandwidthEvent, ConnectionEvent, ConnectionStatisticsEvent, MixnetEvent, SphinxPacketRates,
    TunnelEvent,
};
pub use tunnel_state::{ActionAfterDisconnect, ErrorStateReason, TunnelState, TunnelType};
