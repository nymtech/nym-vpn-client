// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Types shared between nym-vpn-lib and other crates in the workspace.
//!
//! ## Supported bindings
//!
//! 1. [uniffi](https://mozilla.github.io/uniffi-rs/latest/) bindings (feature flag: uniffi-bindings). The following limitations apply:
//! - Namespaces are not supported, all exported types should have unique names.
//! - Not all types are supported or can be bridged. Keep exported types simple.
//!
//! 2. TypeScript bindings using [ts-rs](https://docs.rs/ts-rs) (feature flag: typescript-bindings). Serialization (using serde) uses camelCase for compatibility with TypeScript/Tauri.
//!    Run the following command to generate TypeScript bindings:
//!    ```sh
//!    cargo test -p nym-vpn-lib-types -F typescript-bindings
//!    ```
//!
//! ## Serde support
//!
//! Serde can be enabled using `serde` feature flag. Note that TypeScript adds camelCase transformation for keys. Do not mix both feature flags in the same workspace.

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
#[cfg(feature = "uniffi-bindings")]
mod uniffi_std_types;

pub use account::{
    AccountCommandError, RegisterAccountResponse, VpnApiError, VpnApiErrorResponse,
    controller_error::{AccountControllerError, AccountControllerErrorStateReason},
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
pub use rpc_requests::{
    AccountBalanceResponse, AccountCommandResponse, Coin, ConnectArgs, ConnectOptions,
    DecentralisedObtainTicketbooksRequest, ListGatewaysOptions, StoreAccountRequest, UserAgent,
};
pub use service::{TargetState, VpnServiceConfig, VpnServiceInfo};
pub use tunnel_event::{
    BandwidthEvent, ConnectionEvent, ConnectionStatisticsEvent, MixnetEvent, SphinxPacketRates,
    TunnelEvent,
};
pub use tunnel_state::{ActionAfterDisconnect, ErrorStateReason, TunnelState, TunnelType};

#[cfg(feature = "uniffi-bindings")]
uniffi::setup_scaffolding!();
