// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Types shared between nym-vpn-lib and other crates in the workspace.

mod account;
mod connection_data;
mod device;
mod gateway;
mod log_path;
mod network;
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
pub use connection_data::{
    ConnectionData, EstablishConnectionData, EstablishConnectionState, GatewayId,
    MixnetConnectionData, NymAddress, TunnelConnectionData, WireguardConnectionData, WireguardNode,
};
pub use device::{NymVpnDevice, NymVpnDeviceStatus, NymVpnUsage};
pub use gateway::{
    Asn, AsnKind, Country, Entry, EntryPoint, Exit, ExitPoint, Gateway, GatewayFilter,
    GatewayFilters, GatewayType, Location, NodeIdentity, ParseRecipientError, Performance, Probe,
    ProbeOutcome, Recipient, Score,
};
pub use log_path::LogPath;
pub use network::{
    ApiUrl, ChainDetails, DenomDetailsOwned, FeatureFlags, FlagValue, Network,
    NetworkCompatibility, NymContracts, NymNetworkDetails, NymVpnNetwork, ParsedAccountLinks,
    SystemConfiguration, SystemMessage, ValidatorDetails,
};
pub use nym_validator_client::nyxd::Coin;
pub use rpc_requests::{
    AccountBalanceResponse, AccountCommandResponse, ConnectArgs, ConnectOptions,
    DecentralisedObtainTicketbooksRequest, ListGatewaysOptions, StoreAccountRequest, UserAgent,
};
pub use service::{TargetState, VpnServiceConfig, VpnServiceInfo};
pub use tunnel_event::{
    BandwidthEvent, ConnectionEvent, ConnectionStatisticsEvent, MixnetEvent, SphinxPacketRates,
    TunnelEvent,
};
pub use tunnel_state::{ActionAfterDisconnect, ErrorStateReason, TunnelState, TunnelType};
