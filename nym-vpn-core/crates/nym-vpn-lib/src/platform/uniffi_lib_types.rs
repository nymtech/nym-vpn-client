// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Types providing a bridge between uniffi and nym-vpn-lib-types.

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use nym_gateway_directory::{NodeIdentity, Recipient};
use nym_vpn_lib_types::{
    ActionAfterDisconnect as CoreActionAfterDisconnect, BandwidthEvent as CoreBandwidthEvent,
    ConnectionData as CoreConnectionData, ConnectionEvent as CoreConnectionEvent,
    ConnectionStatisticsEvent as CoreConnectionStatisticsEvent,
    ErrorStateReason as CoreErrorStateReason, MixnetConnectionData as CoreMixnetConnectionData,
    MixnetEvent as CoreMixnetEvent, SphinxPacketRates as CoreSphinxPacketRates,
    TunnelConnectionData as CoreTunnelConnectionData, TunnelEvent as CoreTunnelEvent,
    TunnelState as CoreTunnelState, WireguardConnectionData as CoreWireguardConnectionData,
    WireguardNode as CoreWireguardNode,
};
use nym_wg_go::PublicKey;
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
    SameEntryAndExitGateway,
    InvalidEntryGatewayCountry,
    InvalidExitGatewayCountry,
    BadBandwidthIncrease,
    DuplicateTunFd,
    Internal,
}

impl From<CoreErrorStateReason> for ErrorStateReason {
    fn from(value: CoreErrorStateReason) -> Self {
        match value {
            CoreErrorStateReason::Firewall => Self::Firewall,
            CoreErrorStateReason::Routing => Self::Routing,
            CoreErrorStateReason::Dns => Self::Dns,
            CoreErrorStateReason::TunDevice => Self::TunDevice,
            CoreErrorStateReason::TunnelProvider => Self::TunnelProvider,
            CoreErrorStateReason::SameEntryAndExitGateway => Self::SameEntryAndExitGateway,
            CoreErrorStateReason::InvalidEntryGatewayCountry => Self::InvalidEntryGatewayCountry,
            CoreErrorStateReason::InvalidExitGatewayCountry => Self::InvalidExitGatewayCountry,
            CoreErrorStateReason::BadBandwidthIncrease => Self::BadBandwidthIncrease,
            CoreErrorStateReason::DuplicateTunFd => Self::DuplicateTunFd,
            CoreErrorStateReason::Internal => Self::Internal,
        }
    }
}

#[derive(uniffi::Record)]
pub struct ConnectionData {
    pub entry_gateway: Box<NodeIdentity>,
    pub exit_gateway: Box<NodeIdentity>,
    pub connected_at: Option<OffsetDateTime>,
    pub tunnel: TunnelConnectionData,
}

impl From<CoreConnectionData> for ConnectionData {
    fn from(value: CoreConnectionData) -> Self {
        Self {
            entry_gateway: value.entry_gateway,
            exit_gateway: value.exit_gateway,
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
            nym_address: value.nym_address,
            exit_ipr: value.exit_ipr,
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
    pub nym_address: Box<Recipient>,
    pub exit_ipr: Box<Recipient>,
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
    pub public_key: Box<PublicKey>,
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
