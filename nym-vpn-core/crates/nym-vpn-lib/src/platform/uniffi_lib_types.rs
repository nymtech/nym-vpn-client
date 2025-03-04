// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Types providing a bridge between uniffi and nym-vpn-lib-types.

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use nym_vpn_api_client::response::NymErrorResponse;
use nym_vpn_lib_types::{
    ActionAfterDisconnect as CoreActionAfterDisconnect, BandwidthEvent as CoreBandwidthEvent,
    ConnectionData as CoreConnectionData, ConnectionEvent as CoreConnectionEvent,
    ConnectionStatisticsEvent as CoreConnectionStatisticsEvent,
    ErrorStateReason as CoreErrorStateReason, ForgetAccountError as CoreForgetAccountError,
    Gateway as CoreGateway, MixnetConnectionData as CoreMixnetConnectionData,
    MixnetEvent as CoreMixnetEvent, NymAddress as CoreNymAddress,
    RegisterDeviceError as CoreRegisterDeviceError, RequestZkNymError as CoreRequestZkNymError,
    RequestZkNymErrorReason as CoreRequestZkNymErrorReason,
    RequestZkNymSuccess as CoreRequestZkNymSuccess, SphinxPacketRates as CoreSphinxPacketRates,
    StoreAccountError as CoreStoreAccountError, SyncAccountError as CoreSyncAccountError,
    SyncDeviceError as CoreSyncDeviceError, TunnelConnectionData as CoreTunnelConnectionData,
    TunnelEvent as CoreTunnelEvent, TunnelState as CoreTunnelState,
    VpnApiErrorResponse as CoreVpnApiErrorResponse,
    WireguardConnectionData as CoreWireguardConnectionData, WireguardNode as CoreWireguardNode,
};
use time::OffsetDateTime;

#[derive(uniffi::Enum)]
pub enum TunnelEvent {
    NewState(TunnelState),
    MixnetState(MixnetEvent),
}

impl From<CoreTunnelEvent> for TunnelEvent {
    fn from(value: CoreTunnelEvent) -> Self {
        match value {
            CoreTunnelEvent::NewState(new_state) => Self::NewState(TunnelState::from(new_state)),
            CoreTunnelEvent::MixnetState(event) => Self::MixnetState(MixnetEvent::from(event)),
        }
    }
}

#[derive(uniffi::Enum)]
pub enum TunnelState {
    Disconnected,
    Connecting {
        connection_data: Option<ConnectionData>,
    },
    Connected {
        connection_data: ConnectionData,
    },
    Disconnecting {
        after_disconnect: ActionAfterDisconnect,
    },
    Error(ErrorStateReason),
    Offline {
        reconnect: bool,
    },
}

impl From<CoreTunnelState> for TunnelState {
    fn from(value: CoreTunnelState) -> Self {
        match value {
            CoreTunnelState::Connected { connection_data } => TunnelState::Connected {
                connection_data: ConnectionData::from(connection_data),
            },
            CoreTunnelState::Connecting { connection_data } => TunnelState::Connecting {
                connection_data: connection_data.map(ConnectionData::from),
            },
            CoreTunnelState::Disconnecting { after_disconnect } => TunnelState::Disconnecting {
                after_disconnect: ActionAfterDisconnect::from(after_disconnect),
            },
            CoreTunnelState::Disconnected => TunnelState::Disconnected,
            CoreTunnelState::Error(reason) => TunnelState::Error(ErrorStateReason::from(reason)),
            CoreTunnelState::Offline { reconnect } => TunnelState::Offline { reconnect },
        }
    }
}

#[derive(uniffi::Enum)]
pub enum MixnetEvent {
    Bandwidth(BandwidthEvent),
    Connection(ConnectionEvent),
    ConnectionStatistics(ConnectionStatisticsEvent),
}

impl From<CoreMixnetEvent> for MixnetEvent {
    fn from(value: CoreMixnetEvent) -> Self {
        match value {
            CoreMixnetEvent::Bandwidth(event) => Self::Bandwidth(BandwidthEvent::from(event)),
            CoreMixnetEvent::Connection(event) => Self::Connection(ConnectionEvent::from(event)),
            CoreMixnetEvent::ConnectionStatistics(event) => {
                Self::ConnectionStatistics(ConnectionStatisticsEvent::from(event))
            }
        }
    }
}

#[derive(uniffi::Record)]
pub struct ConnectionStatisticsEvent {
    pub rates: SphinxPacketRates,
}

impl From<CoreConnectionStatisticsEvent> for ConnectionStatisticsEvent {
    fn from(value: CoreConnectionStatisticsEvent) -> Self {
        Self {
            rates: SphinxPacketRates::from(value.rates),
        }
    }
}

impl From<CoreSphinxPacketRates> for SphinxPacketRates {
    fn from(value: CoreSphinxPacketRates) -> Self {
        Self {
            real_packets_sent: value.real_packets_sent,
            real_packets_sent_size: value.real_packets_sent_size,
            cover_packets_sent: value.cover_packets_sent,
            cover_packets_sent_size: value.cover_packets_sent_size,
            real_packets_received: value.real_packets_received,
            real_packets_received_size: value.real_packets_received_size,
            cover_packets_received: value.cover_packets_received,
            cover_packets_received_size: value.cover_packets_received_size,
            total_acks_received: value.total_acks_received,
            total_acks_received_size: value.total_acks_received_size,
            real_acks_received: value.real_acks_received,
            real_acks_received_size: value.real_acks_received_size,
            cover_acks_received: value.cover_acks_received,
            cover_acks_received_size: value.cover_acks_received_size,
            real_packets_queued: value.real_packets_queued,
            retransmissions_queued: value.retransmissions_queued,
            reply_surbs_queued: value.reply_surbs_queued,
            additional_reply_surbs_queued: value.additional_reply_surbs_queued,
        }
    }
}

#[derive(uniffi::Record)]
pub struct SphinxPacketRates {
    pub real_packets_sent: f64,
    pub real_packets_sent_size: f64,
    pub cover_packets_sent: f64,
    pub cover_packets_sent_size: f64,

    pub real_packets_received: f64,
    pub real_packets_received_size: f64,
    pub cover_packets_received: f64,
    pub cover_packets_received_size: f64,

    pub total_acks_received: f64,
    pub total_acks_received_size: f64,
    pub real_acks_received: f64,
    pub real_acks_received_size: f64,
    pub cover_acks_received: f64,
    pub cover_acks_received_size: f64,

    pub real_packets_queued: f64,
    pub retransmissions_queued: f64,
    pub reply_surbs_queued: f64,
    pub additional_reply_surbs_queued: f64,
}

#[derive(uniffi::Enum)]
pub enum BandwidthEvent {
    NoBandwidth,
    RemainingBandwidth(i64),
}

impl From<CoreBandwidthEvent> for BandwidthEvent {
    fn from(value: CoreBandwidthEvent) -> Self {
        match value {
            CoreBandwidthEvent::NoBandwidth => BandwidthEvent::NoBandwidth,
            CoreBandwidthEvent::RemainingBandwidth(value) => {
                BandwidthEvent::RemainingBandwidth(value)
            }
        }
    }
}

#[derive(uniffi::Enum)]
pub enum ConnectionEvent {
    EntryGatewayDown,
    ExitGatewayDownIpv4,
    ExitGatewayDownIpv6,
    ExitGatewayRoutingErrorIpv4,
    ExitGatewayRoutingErrorIpv6,
    ConnectedIpv4,
    ConnectedIpv6,
}

impl From<CoreConnectionEvent> for ConnectionEvent {
    fn from(value: CoreConnectionEvent) -> Self {
        match value {
            CoreConnectionEvent::EntryGatewayDown => Self::EntryGatewayDown,
            CoreConnectionEvent::ExitGatewayDownIpv4 => Self::ExitGatewayDownIpv4,
            CoreConnectionEvent::ExitGatewayDownIpv6 => Self::ExitGatewayDownIpv6,
            CoreConnectionEvent::ExitGatewayRoutingErrorIpv4 => Self::ExitGatewayRoutingErrorIpv4,
            CoreConnectionEvent::ExitGatewayRoutingErrorIpv6 => Self::ExitGatewayRoutingErrorIpv6,
            CoreConnectionEvent::ConnectedIpv4 => Self::ConnectedIpv4,
            CoreConnectionEvent::ConnectedIpv6 => Self::ConnectedIpv6,
        }
    }
}

#[derive(uniffi::Enum)]
pub enum ActionAfterDisconnect {
    Nothing,
    Reconnect,
    Offline,
    Error,
}

impl From<CoreActionAfterDisconnect> for ActionAfterDisconnect {
    fn from(value: CoreActionAfterDisconnect) -> Self {
        match value {
            CoreActionAfterDisconnect::Nothing => Self::Nothing,
            CoreActionAfterDisconnect::Reconnect => Self::Reconnect,
            CoreActionAfterDisconnect::Error => Self::Error,
            CoreActionAfterDisconnect::Offline => Self::Offline,
        }
    }
}

#[derive(uniffi::Enum)]
pub enum ErrorStateReason {
    Firewall,
    Routing,
    Dns,
    TunDevice,
    TunnelProvider,
    ResolveGatewayAddrs,
    StartLocalDnsResolver,
    SameEntryAndExitGateway,
    InvalidEntryGatewayCountry,
    InvalidExitGatewayCountry,
    BadBandwidthIncrease,
    DuplicateTunFd,
    SyncAccount(SyncAccountError),
    SyncDevice(SyncDeviceError),
    RegisterDevice(RegisterDeviceError),
    RequestZkNym(RequestZkNymError),
    RequestZkNymBundle {
        successes: Vec<RequestZkNymSuccess>,
        failed: Vec<RequestZkNymError>,
    },
    Internal(String),
}

#[derive(thiserror::Error, uniffi::Error, Debug, Clone, PartialEq, Eq)]
pub enum StoreAccountError {
    #[error("storage: {0}")]
    Storage(String),
    #[error("vpn api endpoint failure: {0}")]
    GetAccountEndpointFailure(VpnApiErrorResponse),
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
}

impl From<CoreStoreAccountError> for StoreAccountError {
    fn from(value: CoreStoreAccountError) -> Self {
        match value {
            CoreStoreAccountError::Storage(err) => Self::Storage(err),
            CoreStoreAccountError::GetAccountEndpointFailure(failure) => {
                Self::GetAccountEndpointFailure(failure.into())
            }
            CoreStoreAccountError::UnexpectedResponse(response) => {
                Self::UnexpectedResponse(response)
            }
        }
    }
}

#[derive(thiserror::Error, uniffi::Error, Debug, Clone, PartialEq)]
pub enum SyncAccountError {
    #[error("no account stored")]
    NoAccountStored,
    #[error("vpn api endpoint failure: {0}")]
    ErrorResponse(VpnApiErrorResponse),
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<CoreSyncAccountError> for SyncAccountError {
    fn from(value: CoreSyncAccountError) -> Self {
        match value {
            CoreSyncAccountError::NoAccountStored => Self::NoAccountStored,
            CoreSyncAccountError::SyncAccountEndpointFailure(failure) => {
                Self::ErrorResponse(failure.into())
            }
            CoreSyncAccountError::UnexpectedResponse(response) => {
                Self::UnexpectedResponse(response)
            }
            CoreSyncAccountError::Internal(err) => Self::Internal(err),
        }
    }
}

#[derive(thiserror::Error, uniffi::Error, Debug, Clone, PartialEq)]
pub enum SyncDeviceError {
    #[error("no account stored")]
    NoAccountStored,
    #[error("no device stored")]
    NoDeviceStored,
    #[error("vpn api endpoint failure: {0}")]
    ErrorResponse(VpnApiErrorResponse),
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<CoreSyncDeviceError> for SyncDeviceError {
    fn from(value: CoreSyncDeviceError) -> Self {
        match value {
            CoreSyncDeviceError::NoAccountStored => Self::NoAccountStored,
            CoreSyncDeviceError::NoDeviceStored => Self::NoDeviceStored,
            CoreSyncDeviceError::SyncDeviceEndpointFailure(failure) => {
                Self::ErrorResponse(failure.into())
            }
            CoreSyncDeviceError::UnexpectedResponse(response) => Self::UnexpectedResponse(response),
            CoreSyncDeviceError::Internal(err) => Self::Internal(err),
        }
    }
}

#[derive(thiserror::Error, uniffi::Error, Debug, Clone, PartialEq)]
pub enum RegisterDeviceError {
    #[error("no account stored")]
    NoAccountStored,
    #[error("no device stored")]
    NoDeviceStored,
    #[error("vpn api endpoint failure: {0}")]
    ErrorResponse(VpnApiErrorResponse),
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<CoreRegisterDeviceError> for RegisterDeviceError {
    fn from(value: CoreRegisterDeviceError) -> Self {
        match value {
            CoreRegisterDeviceError::NoAccountStored => Self::NoAccountStored,
            CoreRegisterDeviceError::NoDeviceStored => Self::NoDeviceStored,
            CoreRegisterDeviceError::RegisterDeviceEndpointFailure(failure) => {
                Self::ErrorResponse(failure.into())
            }
            CoreRegisterDeviceError::UnexpectedResponse(response) => {
                Self::UnexpectedResponse(response)
            }
            CoreRegisterDeviceError::Internal(err) => Self::Internal(err),
        }
    }
}

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct RequestZkNymSuccess {
    pub id: String,
}

impl From<CoreRequestZkNymSuccess> for RequestZkNymSuccess {
    fn from(success: CoreRequestZkNymSuccess) -> Self {
        Self { id: success.id }
    }
}

#[derive(uniffi::Error, thiserror::Error, Clone, Debug, PartialEq, Eq)]
pub enum RequestZkNymError {
    #[error("no account stored")]
    NoAccountStored,
    #[error("no device stored")]
    NoDeviceStored,
    #[error(transparent)]
    VpnApi(VpnApiErrorResponse),
    #[error("nym-vpn-api: unexpected error response: {0}")]
    UnexpectedVpnApiResponse(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("{0}")]
    Internal(String),
}

impl From<CoreRequestZkNymErrorReason> for RequestZkNymError {
    fn from(error: CoreRequestZkNymErrorReason) -> Self {
        match error {
            CoreRequestZkNymErrorReason::NoAccountStored => Self::NoAccountStored,
            CoreRequestZkNymErrorReason::NoDeviceStored => Self::NoDeviceStored,
            CoreRequestZkNymErrorReason::VpnApi(err) => Self::VpnApi(err.into()),
            CoreRequestZkNymErrorReason::UnexpectedVpnApiResponse(response) => {
                Self::UnexpectedVpnApiResponse(response)
            }
            CoreRequestZkNymErrorReason::Storage(err) => Self::Storage(err),
            CoreRequestZkNymErrorReason::Internal(err) => Self::Internal(err),
        }
    }
}

impl From<CoreRequestZkNymError> for RequestZkNymError {
    fn from(error: CoreRequestZkNymError) -> Self {
        CoreRequestZkNymErrorReason::from(error).into()
    }
}

#[derive(thiserror::Error, uniffi::Error, Debug, Clone, PartialEq, Eq)]
pub enum ForgetAccountError {
    #[error("registration is in progress")]
    RegistrationInProgress,
    #[error("failed to remove device from nym vpn api: {0}")]
    UpdateDeviceErrorResponse(VpnApiErrorResponse),
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
    #[error("failed to remove account: {0}")]
    RemoveAccount(String),
    #[error("failed to remove device keys: {0}")]
    RemoveDeviceKeys(String),
    #[error("failed to reset credential storage: {0}")]
    ResetCredentialStorage(String),
    #[error("failed to remove account files: {0}")]
    RemoveAccountFiles(String),
    #[error("failed to init device keys: {0}")]
    InitDeviceKeys(String),
}

impl From<CoreForgetAccountError> for ForgetAccountError {
    fn from(value: CoreForgetAccountError) -> Self {
        match value {
            CoreForgetAccountError::RegistrationInProgress => Self::RegistrationInProgress,
            CoreForgetAccountError::UpdateDeviceErrorResponse(failure) => {
                Self::UpdateDeviceErrorResponse(failure.into())
            }
            CoreForgetAccountError::UnexpectedResponse(response) => {
                Self::UnexpectedResponse(response)
            }
            CoreForgetAccountError::RemoveAccount(err) => Self::RemoveAccount(err),
            CoreForgetAccountError::RemoveDeviceKeys(err) => Self::RemoveDeviceKeys(err),
            CoreForgetAccountError::ResetCredentialStorage(err) => {
                Self::ResetCredentialStorage(err)
            }
            CoreForgetAccountError::RemoveAccountFiles(err) => Self::RemoveAccountFiles(err),
            CoreForgetAccountError::InitDeviceKeys(err) => Self::InitDeviceKeys(err),
        }
    }
}

#[derive(uniffi::Record, thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error(
    "nym-vpn-api: message: {message}, message_id: {message_id:?}, code_reference_id: {code_reference_id:?}"
)]
pub struct VpnApiErrorResponse {
    pub message: String,
    pub message_id: Option<String>,
    pub code_reference_id: Option<String>,
}

impl From<CoreVpnApiErrorResponse> for VpnApiErrorResponse {
    fn from(value: CoreVpnApiErrorResponse) -> Self {
        Self {
            message: value.message,
            message_id: value.message_id,
            code_reference_id: value.code_reference_id,
        }
    }
}

impl From<NymErrorResponse> for VpnApiErrorResponse {
    fn from(value: NymErrorResponse) -> Self {
        Self {
            message: value.message,
            message_id: value.message_id,
            code_reference_id: value.code_reference_id,
        }
    }
}

impl From<CoreErrorStateReason> for ErrorStateReason {
    fn from(value: CoreErrorStateReason) -> Self {
        match value {
            CoreErrorStateReason::Firewall => Self::Firewall,
            CoreErrorStateReason::Routing => Self::Routing,
            CoreErrorStateReason::Dns => Self::Dns,
            CoreErrorStateReason::TunDevice => Self::TunDevice,
            CoreErrorStateReason::TunnelProvider => Self::TunnelProvider,
            CoreErrorStateReason::ResolveGatewayAddrs => Self::ResolveGatewayAddrs,
            CoreErrorStateReason::StartLocalDnsResolver => Self::StartLocalDnsResolver,
            CoreErrorStateReason::SameEntryAndExitGateway => Self::SameEntryAndExitGateway,
            CoreErrorStateReason::InvalidEntryGatewayCountry => Self::InvalidEntryGatewayCountry,
            CoreErrorStateReason::InvalidExitGatewayCountry => Self::InvalidExitGatewayCountry,
            CoreErrorStateReason::BadBandwidthIncrease => Self::BadBandwidthIncrease,
            CoreErrorStateReason::DuplicateTunFd => Self::DuplicateTunFd,
            CoreErrorStateReason::SyncAccount(err) => Self::SyncAccount(err.into()),
            CoreErrorStateReason::SyncDevice(err) => Self::SyncDevice(err.into()),
            CoreErrorStateReason::RegisterDevice(err) => Self::RegisterDevice(err.into()),
            CoreErrorStateReason::RequestZkNym(err) => Self::RequestZkNym(err.into()),
            CoreErrorStateReason::RequestZkNymBundle { successes, failed } => {
                Self::RequestZkNymBundle {
                    successes: successes
                        .into_iter()
                        .map(RequestZkNymSuccess::from)
                        .collect(),
                    failed: failed.into_iter().map(RequestZkNymError::from).collect(),
                }
            }
            CoreErrorStateReason::Internal(message) => Self::Internal(message),
        }
    }
}

#[derive(uniffi::Record)]
pub struct Gateway {
    /// Gateway id in base58.
    pub id: String,
}

impl From<CoreGateway> for Gateway {
    fn from(value: CoreGateway) -> Self {
        Self { id: value.id }
    }
}

#[derive(uniffi::Record)]
pub struct NymAddress {
    pub nym_address: String,
    pub gateway_id: String,
}

impl From<CoreNymAddress> for NymAddress {
    fn from(value: CoreNymAddress) -> Self {
        Self {
            nym_address: value.nym_address,
            gateway_id: value.gateway_id,
        }
    }
}

#[derive(uniffi::Record)]
pub struct ConnectionData {
    pub entry_gateway: Gateway,
    pub exit_gateway: Gateway,
    pub connected_at: Option<OffsetDateTime>,
    pub tunnel: TunnelConnectionData,
}

impl From<CoreConnectionData> for ConnectionData {
    fn from(value: CoreConnectionData) -> Self {
        Self {
            entry_gateway: Gateway::from(value.entry_gateway),
            exit_gateway: Gateway::from(value.exit_gateway),
            connected_at: value.connected_at,
            tunnel: TunnelConnectionData::from(value.tunnel),
        }
    }
}

impl From<CoreTunnelConnectionData> for TunnelConnectionData {
    fn from(value: CoreTunnelConnectionData) -> Self {
        match value {
            CoreTunnelConnectionData::Mixnet(data) => {
                TunnelConnectionData::Mixnet(MixnetConnectionData::from(data))
            }
            CoreTunnelConnectionData::Wireguard(data) => {
                TunnelConnectionData::Wireguard(WireguardConnectionData::from(data))
            }
        }
    }
}

impl From<CoreMixnetConnectionData> for MixnetConnectionData {
    fn from(value: CoreMixnetConnectionData) -> Self {
        Self {
            nym_address: NymAddress::from(value.nym_address),
            exit_ipr: NymAddress::from(value.exit_ipr),
            ipv4: value.ipv4,
            ipv6: value.ipv6,
        }
    }
}

impl From<CoreWireguardConnectionData> for WireguardConnectionData {
    fn from(value: CoreWireguardConnectionData) -> Self {
        Self {
            entry: WireguardNode::from(value.entry),
            exit: WireguardNode::from(value.exit),
        }
    }
}

#[derive(uniffi::Enum)]
pub enum TunnelConnectionData {
    Mixnet(MixnetConnectionData),
    Wireguard(WireguardConnectionData),
}

#[derive(uniffi::Record)]
pub struct MixnetConnectionData {
    pub nym_address: NymAddress,
    pub exit_ipr: NymAddress,
    pub ipv4: Ipv4Addr,
    pub ipv6: Ipv6Addr,
}

#[derive(uniffi::Record)]
pub struct WireguardConnectionData {
    pub entry: WireguardNode,
    pub exit: WireguardNode,
}

#[derive(uniffi::Record)]
pub struct WireguardNode {
    pub endpoint: SocketAddr,
    pub public_key: String,
    pub private_ipv4: Ipv4Addr,
    pub private_ipv6: Ipv6Addr,
}

impl From<CoreWireguardNode> for WireguardNode {
    fn from(value: CoreWireguardNode) -> Self {
        Self {
            endpoint: value.endpoint,
            public_key: value.public_key,
            private_ipv4: value.private_ipv4,
            private_ipv6: value.private_ipv6,
        }
    }
}
