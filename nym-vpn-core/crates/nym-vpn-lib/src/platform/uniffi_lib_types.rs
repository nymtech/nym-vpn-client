// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Types providing a bridge between uniffi and nym-vpn-lib-types.

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use nym_vpn_api_client::response::NymErrorResponse;
use nym_vpn_lib_types::{
    AccountControllerErrorStateReason as CoreAccountControllerErrorStateReason,
    AccountControllerState as CoreAccountControllerState,
    ActionAfterDisconnect as CoreActionAfterDisconnect, BandwidthEvent as CoreBandwidthEvent,
    ClientErrorReason, ConnectionData as CoreConnectionData,
    ConnectionEvent as CoreConnectionEvent,
    ConnectionStatisticsEvent as CoreConnectionStatisticsEvent, Gateway as CoreGateway,
    MixnetConnectionData as CoreMixnetConnectionData, MixnetEvent as CoreMixnetEvent,
    NymAddress as CoreNymAddress, RegisterAccountResponse as CoreRegisterAccountResponse,
    RequestZkNymError as CoreRequestZkNymError,
    RequestZkNymErrorReason as CoreRequestZkNymErrorReason,
    RequestZkNymSuccess as CoreRequestZkNymSuccess, SphinxPacketRates as CoreSphinxPacketRates,
    TunnelConnectionData as CoreTunnelConnectionData, TunnelEvent as CoreTunnelEvent,
    TunnelState as CoreTunnelState, VpnApiError as CoreVpnApiErrorResponseTop,
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
        retry_attempt: u32,
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
            CoreTunnelState::Connecting {
                retry_attempt,
                connection_data,
            } => TunnelState::Connecting {
                retry_attempt,
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
    SameEntryAndExitGateway,
    InvalidEntryGatewayCountry,
    InvalidExitGatewayCountry,
    MaxDevicesReached,
    BandwidthExceeded,
    InactiveSubscription,
    Dns(Option<String>),
    Api(Option<String>),
    DeviceTimeOutOfSync,
    CreateMixnetStorage,
    Ipv6Unavailable,
    Internal(Option<String>),
    AccountControl(Option<String>),
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
    VpnApi(VpnApiError),
    #[error("nym-vpn-api: unexpected error response: {0}")]
    UnexpectedVpnApiResponse(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("no connectivity")]
    Offline,
    #[error("{0}")]
    Internal(String),
}

impl From<CoreRequestZkNymErrorReason> for RequestZkNymError {
    fn from(error: CoreRequestZkNymErrorReason) -> Self {
        match error {
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

#[derive(uniffi::Enum, thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum VpnApiError {
    #[error("timeout")]
    Timeout,
    #[error("status code: {0}")]
    StatusCode(u16),
    #[error(transparent)]
    Response(#[from] VpnApiErrorResponse),
}

impl From<CoreVpnApiErrorResponseTop> for VpnApiError {
    fn from(value: CoreVpnApiErrorResponseTop) -> Self {
        match value {
            CoreVpnApiErrorResponseTop::Timeout(..) => Self::Timeout,
            CoreVpnApiErrorResponseTop::StatusCode { code, .. } => Self::StatusCode(code),
            CoreVpnApiErrorResponseTop::Response(response) => Self::Response(response.into()),
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

impl From<ClientErrorReason> for ErrorStateReason {
    fn from(value: ClientErrorReason) -> Self {
        match value {
            ClientErrorReason::Firewall => Self::Firewall,
            ClientErrorReason::Routing => Self::Routing,
            ClientErrorReason::SameEntryAndExitGateway => Self::SameEntryAndExitGateway,
            ClientErrorReason::InvalidEntryGatewayCountry => Self::InvalidEntryGatewayCountry,
            ClientErrorReason::InvalidExitGatewayCountry => Self::InvalidExitGatewayCountry,
            ClientErrorReason::MaxDevicesReached => Self::MaxDevicesReached,
            ClientErrorReason::BandwidthExceeded => Self::BandwidthExceeded,
            ClientErrorReason::InactiveSubscription => Self::InactiveSubscription,
            ClientErrorReason::Dns(message) => Self::Dns(message),
            ClientErrorReason::Api(message) => Self::Api(message),
            ClientErrorReason::DeviceTimeOutOfSync => Self::DeviceTimeOutOfSync,
            ClientErrorReason::CreateMixnetStorage => Self::CreateMixnetStorage,
            ClientErrorReason::Ipv6Unavailable => Self::Ipv6Unavailable,
            ClientErrorReason::Internal(message) => Self::Internal(message),
            ClientErrorReason::AccountControl(message) => Self::AccountControl(message),
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
    pub ipv6: Option<Ipv6Addr>,
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
    pub private_ipv6: Option<Ipv6Addr>,
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

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum AccountControllerState {
    Offline,
    Syncing,
    LoggedOut,
    ReadyToConnect,
    Error(AccountControllerErrorStateReason),
}

impl From<CoreAccountControllerState> for AccountControllerState {
    fn from(value: CoreAccountControllerState) -> Self {
        match value {
            CoreAccountControllerState::Offline => AccountControllerState::Offline,
            CoreAccountControllerState::Syncing => Self::Syncing,
            CoreAccountControllerState::LoggedOut => Self::LoggedOut,
            CoreAccountControllerState::ReadyToConnect => Self::ReadyToConnect,
            CoreAccountControllerState::Error(error_state_reason) => {
                Self::Error(error_state_reason.into())
            }
        }
    }
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum AccountControllerErrorStateReason {
    Storage { context: String },
    ApiFailure { context: String, details: String },
    Internal { context: String, details: String },
    BandwidthExceeded { context: String },
    AccountStatusNotActive { status: String },
    InactiveSubscription,
    MaxDeviceReached,
    DeviceTimeDesynced,
}

impl From<CoreAccountControllerErrorStateReason> for AccountControllerErrorStateReason {
    fn from(value: CoreAccountControllerErrorStateReason) -> Self {
        match value {
            CoreAccountControllerErrorStateReason::Storage { context } => Self::Storage { context },
            CoreAccountControllerErrorStateReason::ApiFailure { context, details } => {
                Self::ApiFailure { context, details }
            }
            CoreAccountControllerErrorStateReason::Internal { context, details } => {
                Self::Internal { context, details }
            }
            CoreAccountControllerErrorStateReason::BandwidthExceeded { context } => {
                Self::BandwidthExceeded { context }
            }
            CoreAccountControllerErrorStateReason::AccountStatusNotActive { status } => {
                Self::AccountStatusNotActive { status }
            }
            CoreAccountControllerErrorStateReason::InactiveSubscription => {
                Self::InactiveSubscription
            }
            CoreAccountControllerErrorStateReason::MaxDeviceReached => Self::MaxDeviceReached,
            CoreAccountControllerErrorStateReason::DeviceTimeDesynced => Self::DeviceTimeDesynced,
        }
    }
}

#[derive(uniffi::Record, Clone, Default, PartialEq)]
pub struct RegisterAccountResponse {
    pub account_token: String,
}

impl From<CoreRegisterAccountResponse> for RegisterAccountResponse {
    fn from(value: CoreRegisterAccountResponse) -> Self {
        RegisterAccountResponse {
            account_token: value.account_token,
        }
    }
}
