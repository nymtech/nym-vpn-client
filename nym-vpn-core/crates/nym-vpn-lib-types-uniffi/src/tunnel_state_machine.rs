// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use time::OffsetDateTime;

#[derive(uniffi::Enum)]
pub enum TunnelEvent {
    NewState(TunnelState),
    MixnetState(MixnetEvent),
}

impl From<nym_vpn_lib_types::TunnelEvent> for TunnelEvent {
    fn from(value: nym_vpn_lib_types::TunnelEvent) -> Self {
        match value {
            nym_vpn_lib_types::TunnelEvent::NewState(new_state) => {
                Self::NewState(TunnelState::from(new_state))
            }
            nym_vpn_lib_types::TunnelEvent::MixnetState(event) => {
                Self::MixnetState(MixnetEvent::from(event))
            }
        }
    }
}

#[derive(uniffi::Enum)]
pub enum TunnelType {
    Mixnet,
    Wireguard,
}

impl From<nym_vpn_lib_types::TunnelType> for TunnelType {
    fn from(value: nym_vpn_lib_types::TunnelType) -> Self {
        match value {
            nym_vpn_lib_types::TunnelType::Mixnet => Self::Mixnet,
            nym_vpn_lib_types::TunnelType::Wireguard => Self::Wireguard,
        }
    }
}

#[derive(uniffi::Enum)]
pub enum TunnelState {
    Disconnected,
    Connecting {
        retry_attempt: u32,
        state: EstablishConnectionState,
        tunnel_type: TunnelType,
        connection_data: Option<EstablishConnectionData>,
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

impl From<nym_vpn_lib_types::TunnelState> for TunnelState {
    fn from(value: nym_vpn_lib_types::TunnelState) -> Self {
        match value {
            nym_vpn_lib_types::TunnelState::Connected { connection_data } => {
                TunnelState::Connected {
                    connection_data: ConnectionData::from(connection_data),
                }
            }
            nym_vpn_lib_types::TunnelState::Connecting {
                retry_attempt,
                state,
                tunnel_type,
                connection_data,
            } => {
                let connection_data = connection_data.map(EstablishConnectionData::from);
                let state = EstablishConnectionState::from(state);
                let tunnel_type = TunnelType::from(tunnel_type);
                TunnelState::Connecting {
                    retry_attempt,
                    state,
                    tunnel_type,
                    connection_data,
                }
            }
            nym_vpn_lib_types::TunnelState::Disconnecting { after_disconnect } => {
                TunnelState::Disconnecting {
                    after_disconnect: ActionAfterDisconnect::from(after_disconnect),
                }
            }
            nym_vpn_lib_types::TunnelState::Disconnected => TunnelState::Disconnected,
            nym_vpn_lib_types::TunnelState::Error(reason) => {
                TunnelState::Error(ErrorStateReason::from(reason))
            }
            nym_vpn_lib_types::TunnelState::Offline { reconnect } => {
                TunnelState::Offline { reconnect }
            }
        }
    }
}

#[derive(uniffi::Enum)]
pub enum MixnetEvent {
    Bandwidth(BandwidthEvent),
    Connection(ConnectionEvent),
    ConnectionStatistics(ConnectionStatisticsEvent),
}

impl From<nym_vpn_lib_types::MixnetEvent> for MixnetEvent {
    fn from(value: nym_vpn_lib_types::MixnetEvent) -> Self {
        match value {
            nym_vpn_lib_types::MixnetEvent::Bandwidth(event) => {
                Self::Bandwidth(BandwidthEvent::from(event))
            }
            nym_vpn_lib_types::MixnetEvent::Connection(event) => {
                Self::Connection(ConnectionEvent::from(event))
            }
            nym_vpn_lib_types::MixnetEvent::ConnectionStatistics(event) => {
                Self::ConnectionStatistics(ConnectionStatisticsEvent::from(event))
            }
        }
    }
}

#[derive(uniffi::Record)]
pub struct ConnectionStatisticsEvent {
    pub rates: SphinxPacketRates,
}

impl From<nym_vpn_lib_types::ConnectionStatisticsEvent> for ConnectionStatisticsEvent {
    fn from(value: nym_vpn_lib_types::ConnectionStatisticsEvent) -> Self {
        Self {
            rates: SphinxPacketRates::from(value.rates),
        }
    }
}

impl From<nym_vpn_lib_types::SphinxPacketRates> for SphinxPacketRates {
    fn from(value: nym_vpn_lib_types::SphinxPacketRates) -> Self {
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

impl From<nym_vpn_lib_types::BandwidthEvent> for BandwidthEvent {
    fn from(value: nym_vpn_lib_types::BandwidthEvent) -> Self {
        match value {
            nym_vpn_lib_types::BandwidthEvent::NoBandwidth => BandwidthEvent::NoBandwidth,
            nym_vpn_lib_types::BandwidthEvent::RemainingBandwidth(value) => {
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

impl From<nym_vpn_lib_types::ConnectionEvent> for ConnectionEvent {
    fn from(value: nym_vpn_lib_types::ConnectionEvent) -> Self {
        match value {
            nym_vpn_lib_types::ConnectionEvent::EntryGatewayDown => Self::EntryGatewayDown,
            nym_vpn_lib_types::ConnectionEvent::ExitGatewayDownIpv4 => Self::ExitGatewayDownIpv4,
            nym_vpn_lib_types::ConnectionEvent::ExitGatewayDownIpv6 => Self::ExitGatewayDownIpv6,
            nym_vpn_lib_types::ConnectionEvent::ExitGatewayRoutingErrorIpv4 => {
                Self::ExitGatewayRoutingErrorIpv4
            }
            nym_vpn_lib_types::ConnectionEvent::ExitGatewayRoutingErrorIpv6 => {
                Self::ExitGatewayRoutingErrorIpv6
            }
            nym_vpn_lib_types::ConnectionEvent::ConnectedIpv4 => Self::ConnectedIpv4,
            nym_vpn_lib_types::ConnectionEvent::ConnectedIpv6 => Self::ConnectedIpv6,
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

impl From<nym_vpn_lib_types::ActionAfterDisconnect> for ActionAfterDisconnect {
    fn from(value: nym_vpn_lib_types::ActionAfterDisconnect) -> Self {
        match value {
            nym_vpn_lib_types::ActionAfterDisconnect::Nothing => Self::Nothing,
            nym_vpn_lib_types::ActionAfterDisconnect::Reconnect => Self::Reconnect,
            nym_vpn_lib_types::ActionAfterDisconnect::Error => Self::Error,
            nym_vpn_lib_types::ActionAfterDisconnect::Offline => Self::Offline,
        }
    }
}

#[derive(uniffi::Enum)]
pub enum ErrorStateReason {
    /// Failure to set firewall policy.
    SetFirewallPolicy,

    /// Failure to configure routing.
    SetRouting,

    /// Failure to configure dns.
    SetDns,

    /// Failure to configure tunnel device.
    TunDevice,

    /// Failure to configure tunnel provider.
    TunnelProvider,

    /// IPv6 is disabled in the system.
    Ipv6Unavailable,

    /// Same entry and exit gateway are unsupported.
    SameEntryAndExitGateway,

    /// Invalid country set for entry gateway
    InvalidEntryGatewayCountry,

    /// Invalid country set for exit gateway
    InvalidExitGatewayCountry,

    /// Gateway is not responding or responding badly to a bandwidth
    /// increase request, causing credential waste
    BadBandwidthIncrease,

    /// Bandwidth Exceeded
    BandwidthExceeded,

    /// Account status is not "Active"
    InactiveAccount,

    /// Inactive Subscription
    InactiveSubscription,

    /// Max device numbers reached
    MaxDevicesReached,

    /// Device time is off by too much, Zk-nyms use will fail
    DeviceTimeOutOfSync,

    /// Device is logged out
    DeviceLoggedOut,

    /// Program errors that must not happen.
    Internal(String),
}

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct RequestZkNymSuccess {
    pub id: String,
}

impl From<nym_vpn_lib_types::RequestZkNymSuccess> for RequestZkNymSuccess {
    fn from(success: nym_vpn_lib_types::RequestZkNymSuccess) -> Self {
        Self { id: success.id }
    }
}

#[derive(uniffi::Enum, thiserror::Error, Clone, Debug, PartialEq, Eq)]
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

impl From<nym_vpn_lib_types::RequestZkNymErrorReason> for RequestZkNymError {
    fn from(error: nym_vpn_lib_types::RequestZkNymErrorReason) -> Self {
        match error {
            nym_vpn_lib_types::RequestZkNymErrorReason::VpnApi(err) => Self::VpnApi(err.into()),
            nym_vpn_lib_types::RequestZkNymErrorReason::UnexpectedVpnApiResponse(response) => {
                Self::UnexpectedVpnApiResponse(response)
            }
            nym_vpn_lib_types::RequestZkNymErrorReason::Storage(err) => Self::Storage(err),
            nym_vpn_lib_types::RequestZkNymErrorReason::Internal(err) => Self::Internal(err),
        }
    }
}

impl From<nym_vpn_lib_types::RequestZkNymError> for RequestZkNymError {
    fn from(error: nym_vpn_lib_types::RequestZkNymError) -> Self {
        nym_vpn_lib_types::RequestZkNymErrorReason::from(error).into()
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

impl From<nym_vpn_lib_types::VpnApiError> for VpnApiError {
    fn from(value: nym_vpn_lib_types::VpnApiError) -> Self {
        match value {
            nym_vpn_lib_types::VpnApiError::Timeout(..) => Self::Timeout,
            nym_vpn_lib_types::VpnApiError::StatusCode { code, .. } => Self::StatusCode(code),
            nym_vpn_lib_types::VpnApiError::Response(response) => Self::Response(response.into()),
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

impl From<nym_vpn_lib_types::VpnApiErrorResponse> for VpnApiErrorResponse {
    fn from(value: nym_vpn_lib_types::VpnApiErrorResponse) -> Self {
        Self {
            message: value.message,
            message_id: value.message_id,
            code_reference_id: value.code_reference_id,
        }
    }
}

impl From<nym_vpn_api_client::response::NymErrorResponse> for VpnApiErrorResponse {
    fn from(value: nym_vpn_api_client::response::NymErrorResponse) -> Self {
        Self {
            message: value.message,
            message_id: value.message_id,
            code_reference_id: value.code_reference_id,
        }
    }
}

impl From<nym_vpn_lib_types::ErrorStateReason> for ErrorStateReason {
    fn from(value: nym_vpn_lib_types::ErrorStateReason) -> Self {
        match value {
            nym_vpn_lib_types::ErrorStateReason::SetFirewallPolicy => Self::SetFirewallPolicy,
            nym_vpn_lib_types::ErrorStateReason::SetRouting => Self::SetRouting,
            nym_vpn_lib_types::ErrorStateReason::SetDns => Self::SetDns,
            nym_vpn_lib_types::ErrorStateReason::TunDevice => Self::TunDevice,
            nym_vpn_lib_types::ErrorStateReason::TunnelProvider => Self::TunnelProvider,
            nym_vpn_lib_types::ErrorStateReason::Ipv6Unavailable => Self::Ipv6Unavailable,
            nym_vpn_lib_types::ErrorStateReason::SameEntryAndExitGateway => {
                Self::SameEntryAndExitGateway
            }
            nym_vpn_lib_types::ErrorStateReason::InvalidEntryGatewayCountry => {
                Self::InvalidEntryGatewayCountry
            }
            nym_vpn_lib_types::ErrorStateReason::InvalidExitGatewayCountry => {
                Self::InvalidExitGatewayCountry
            }
            nym_vpn_lib_types::ErrorStateReason::BadBandwidthIncrease => Self::BadBandwidthIncrease,
            nym_vpn_lib_types::ErrorStateReason::BandwidthExceeded => Self::BandwidthExceeded,
            nym_vpn_lib_types::ErrorStateReason::InactiveAccount => Self::InactiveAccount,
            nym_vpn_lib_types::ErrorStateReason::InactiveSubscription => Self::InactiveSubscription,
            nym_vpn_lib_types::ErrorStateReason::MaxDevicesReached => Self::MaxDevicesReached,
            nym_vpn_lib_types::ErrorStateReason::DeviceTimeOutOfSync => Self::DeviceTimeOutOfSync,
            nym_vpn_lib_types::ErrorStateReason::DeviceLoggedOut => Self::DeviceLoggedOut,
            nym_vpn_lib_types::ErrorStateReason::Internal(msg) => Self::Internal(msg),
        }
    }
}

#[derive(uniffi::Record)]
pub struct Gateway {
    /// Gateway id in base58.
    pub id: String,
}

impl From<nym_vpn_lib_types::Gateway> for Gateway {
    fn from(value: nym_vpn_lib_types::Gateway) -> Self {
        Self { id: value.id }
    }
}

#[derive(uniffi::Record)]
pub struct NymAddress {
    pub nym_address: String,
    pub gateway_id: String,
}

impl From<nym_vpn_lib_types::NymAddress> for NymAddress {
    fn from(value: nym_vpn_lib_types::NymAddress) -> Self {
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

impl From<nym_vpn_lib_types::ConnectionData> for ConnectionData {
    fn from(value: nym_vpn_lib_types::ConnectionData) -> Self {
        Self {
            entry_gateway: Gateway::from(value.entry_gateway),
            exit_gateway: Gateway::from(value.exit_gateway),
            connected_at: Some(value.connected_at),
            tunnel: TunnelConnectionData::from(value.tunnel),
        }
    }
}

#[derive(uniffi::Enum)]
pub enum EstablishConnectionState {
    ResolvingApiAddresses,
    AwaitingAccountReadiness,
    RefreshingGateways,
    SelectingGateways,
    ConnectingMixnetClient,
    ConnectingTunnel,
}

impl From<nym_vpn_lib_types::EstablishConnectionState> for EstablishConnectionState {
    fn from(value: nym_vpn_lib_types::EstablishConnectionState) -> Self {
        match value {
            nym_vpn_lib_types::EstablishConnectionState::ResolvingApiAddresses => {
                EstablishConnectionState::ResolvingApiAddresses
            }
            nym_vpn_lib_types::EstablishConnectionState::AwaitingAccountReadiness => {
                EstablishConnectionState::AwaitingAccountReadiness
            }
            nym_vpn_lib_types::EstablishConnectionState::RefreshingGateways => {
                EstablishConnectionState::RefreshingGateways
            }
            nym_vpn_lib_types::EstablishConnectionState::SelectingGateways => {
                EstablishConnectionState::SelectingGateways
            }
            nym_vpn_lib_types::EstablishConnectionState::ConnectingMixnetClient => {
                EstablishConnectionState::ConnectingMixnetClient
            }
            nym_vpn_lib_types::EstablishConnectionState::ConnectingTunnel => {
                EstablishConnectionState::ConnectingTunnel
            }
        }
    }
}

#[derive(uniffi::Record)]
pub struct EstablishConnectionData {
    pub entry_gateway: Gateway,
    pub exit_gateway: Gateway,
    pub tunnel: Option<TunnelConnectionData>,
}

impl From<nym_vpn_lib_types::EstablishConnectionData> for EstablishConnectionData {
    fn from(value: nym_vpn_lib_types::EstablishConnectionData) -> Self {
        Self {
            entry_gateway: Gateway::from(value.entry_gateway),
            exit_gateway: Gateway::from(value.exit_gateway),
            tunnel: value.tunnel.map(TunnelConnectionData::from),
        }
    }
}

impl From<nym_vpn_lib_types::TunnelConnectionData> for TunnelConnectionData {
    fn from(value: nym_vpn_lib_types::TunnelConnectionData) -> Self {
        match value {
            nym_vpn_lib_types::TunnelConnectionData::Mixnet(data) => {
                TunnelConnectionData::Mixnet(MixnetConnectionData::from(data))
            }
            nym_vpn_lib_types::TunnelConnectionData::Wireguard(data) => {
                TunnelConnectionData::Wireguard(WireguardConnectionData::from(data))
            }
            nym_vpn_lib_types::TunnelConnectionData::WrappedWireguard(data) => {
                TunnelConnectionData::Wireguard(WireguardConnectionData::from(data))
            }
        }
    }
}

impl From<nym_vpn_lib_types::MixnetConnectionData> for MixnetConnectionData {
    fn from(value: nym_vpn_lib_types::MixnetConnectionData) -> Self {
        Self {
            nym_address: NymAddress::from(value.nym_address),
            exit_ipr: NymAddress::from(value.exit_ipr),
            ipv4: value.ipv4,
            ipv6: value.ipv6,
        }
    }
}

impl From<nym_vpn_lib_types::WireguardConnectionData> for WireguardConnectionData {
    fn from(value: nym_vpn_lib_types::WireguardConnectionData) -> Self {
        Self {
            entry: WireguardNode::from(value.entry),
            exit: WireguardNode::from(value.exit),
        }
    }
}

impl From<nym_vpn_lib_types::WrappedWireguardConnectionData> for WireguardConnectionData {
    fn from(value: nym_vpn_lib_types::WrappedWireguardConnectionData) -> Self {
        let mut entry = WireguardNode::from(value.entry);
        entry.endpoint = value.entry_bridge_addr;
        Self {
            entry,
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

impl From<nym_vpn_lib_types::WireguardNode> for WireguardNode {
    fn from(value: nym_vpn_lib_types::WireguardNode) -> Self {
        Self {
            endpoint: value.endpoint,
            public_key: value.public_key,
            private_ipv4: value.private_ipv4,
            private_ipv6: value.private_ipv6,
        }
    }
}
