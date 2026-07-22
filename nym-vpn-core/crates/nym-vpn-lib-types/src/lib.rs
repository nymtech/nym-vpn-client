// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Types shared between nym-vpn-lib and other crates in the workspace.
//!
//! ## Abstract
//!
//! - This crate contains all types necessary for interaction with nym-vpn-lib and daemon.
//! - Please do not re-export types from other crates, instead define them here.
//!   When mirroring external types, consider providing `From` implementations for easier conversions. Feature-gate conversions to `nym-type-conversions`.
//! - Types visible via bindings should contain proper attributes and feature gated to `uniffi-bindings` for uniffi, `typescript-bindings` for TypeScript bindings.
//! - TypeScript bindings use serde for conversion from Rust to TS and feature-gated to `typescript-bindings`. Camel case is preferred for compatibility with TypeScript/Tauri.
//! - Be mindful of limitations of TypeScript and uniffi limitations. Keep exported types simple.
//!
//! ## Dependency considerations
//!
//! Please keep direct dependencies to other crates to a minimum to avoid dependency conflicts which can happen, especially when using it in other large projects such as Tauri.

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
mod diagnostic;
mod gateway;
mod gateway_independence;
mod gateway_selection_algorithm;
mod network;
mod network_stats;
mod paths;
mod privy;
mod rpc_requests;
mod service;
mod socks5;
mod split_tunnel;
mod tunnel_event;
mod tunnel_state;
#[cfg(feature = "uniffi-bindings")]
mod uniffi_std_types;
mod user_agent;

#[cfg(any(target_os = "android", target_os = "ios"))]
pub use account::RegisterAccountRequest;
pub use account::{
    AccountCommandError, NymVpnSubscription, NymVpnSubscriptionKind, NymVpnSubscriptionStatus,
    RegisterAccountResponse, StoredAccountMode, Subscription, VpnAccountAuthMethod,
    VpnAccountStatus, VpnAccountSummary, VpnApiError, VpnApiErrorResponse,
    controller_error::{AccountControllerError, AccountControllerErrorStateReason},
    controller_event::AccountControllerEvent,
    controller_state::AccountControllerState,
    deeplink::{AutologinResponse, DeeplinkClient, DeeplinkKind, GetDeeplinkParams},
    storage::{Mnemonic, StorableAccount},
    ticketbooks::AvailableTickets,
};
pub use connection_data::{
    BridgeAddress, ConnectionData, EstablishConnectionData, EstablishConnectionState, GatewayId,
    GatewayLightInfo, MixnetConnectionData, NymAddress, TunnelConnectionData,
    WireguardConnectionData, WireguardNode,
};
pub use device::{NymVpnDevice, NymVpnDeviceStatus, NymVpnUsage};
pub use diagnostic::{
    ApiTimeSkew, CompleteDnsReport, DiagnosticEndpointResponse, DiagnosticRegisterParams,
    DiagnosticReport, DiagnosticResult, DiagnosticRunParams, DnsResolution, GatewayReport,
    HttpReport, HybridTransportReport, PingReport, RegistrationMode, RegistrationReport,
};
pub use gateway::{
    Asn, AsnKind, BridgeInformation, BridgeParameters, Country, Entry, EntryPoint, Exit, ExitPoint,
    Gateway, GatewayFilter, GatewayType, LewesProtocolDetails, LewesProtocolDetailsData, Location,
    LookupGatewayFilters, Lp, NodeIdentity, ParseRecipientError, Performance, Probe, ProbeOutcome,
    QuicClientOptions, Recipient, Score, Socks5, TentativeGateways,
};
pub use gateway_independence::GatewayIndependence;
pub use gateway_selection_algorithm::{GatewaySelectionAlgorithm, GatewaySelectionAlgorithmConfig};
pub use network::{
    ApiUrl, ChainDetails, DenomDetailsOwned, FeatureFlags, FlagValue, Network,
    NetworkCompatibility, NymContracts, NymNetworkDetails, NymVpnNetwork, ParsedAccountLinks,
    SystemConfiguration, SystemMessage, ValidatorDetails,
};
pub use network_stats::{NetworkStatisticsConfig, NetworkStatisticsIdentity};
pub use paths::LogPath;
pub use privy::PrivyDerivationMessage;
pub use rpc_requests::{
    AccountBalanceResponse, AccountCommandResponse, Coin, ListGatewaysOptions, StoreAccountRequest,
};
#[cfg(feature = "uniffi-bindings")]
pub use service::default_vpn_service_config;
pub use service::{
    BackgroundCoverTrafficRate, ContinuousTrafficSendingRate, FrontingMode, GeoExclusionSettings,
    MixingDelay, MixnetTrafficConfig, MixnetTrafficConfigValidationError, MixnetTrafficDefaults,
    SplitApp, SplitTunnelSettings, TargetState, VpnServiceConfig, VpnServiceInfo,
};
pub use socks5::{EnableSocks5Request, HttpRpcSettings, Socks5Settings, Socks5State, Socks5Status};
pub use split_tunnel::{SplitTunnelExcludedProcess, SplitTunnelExcludedProcessList};
pub use tunnel_event::{
    BandwidthEvent, ConnectionEvent, ConnectionStatisticsEvent, MixnetEvent, SphinxPacketRates,
    TunnelEvent,
};
pub use tunnel_state::{ActionAfterDisconnect, ErrorStateReason, TunnelState, TunnelType};
pub use user_agent::UserAgent;

#[cfg(feature = "uniffi-bindings")]
uniffi::setup_scaffolding!();
