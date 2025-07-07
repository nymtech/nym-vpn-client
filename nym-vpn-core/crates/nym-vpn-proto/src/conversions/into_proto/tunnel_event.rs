// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    BandwidthEvent, ConnectionEvent, ConnectionStatisticsEvent, MixnetEvent, SphinxPacketRates,
    TunnelEvent,
};

use crate::proto;

impl From<TunnelEvent> for proto::TunnelEvent {
    fn from(value: TunnelEvent) -> Self {
        let event = match value {
            TunnelEvent::NewState(tunnel_state) => {
                proto::tunnel_event::Event::TunnelState(proto::TunnelState::from(tunnel_state))
            }
            TunnelEvent::MixnetState(mixnet_event) => {
                proto::tunnel_event::Event::MixnetEvent(proto::MixnetEvent::from(mixnet_event))
            }
        };
        Self { event: Some(event) }
    }
}

impl From<MixnetEvent> for proto::MixnetEvent {
    fn from(value: MixnetEvent) -> Self {
        let event = match value {
            MixnetEvent::Bandwidth(e) => proto::mixnet_event::Event::BandwidthEvent(
                proto::mixnet_event::BandwidthEvent::from(e),
            ),
            MixnetEvent::Connection(e) => proto::mixnet_event::Event::ConnectionEvent(
                proto::mixnet_event::ConnectionEvent::from(e) as i32,
            ),
            MixnetEvent::ConnectionStatistics(e) => {
                proto::mixnet_event::Event::ConnectionStatisticsEvent(
                    proto::mixnet_event::ConnectionStatisticsEvent::from(e),
                )
            }
        };

        Self { event: Some(event) }
    }
}

impl From<BandwidthEvent> for proto::mixnet_event::BandwidthEvent {
    fn from(value: BandwidthEvent) -> Self {
        let event = match value {
            BandwidthEvent::NoBandwidth => {
                proto::mixnet_event::bandwidth_event::Event::NoBandwidth(
                    proto::mixnet_event::bandwidth_event::NoBandwidth {},
                )
            }
            BandwidthEvent::RemainingBandwidth(value) => {
                proto::mixnet_event::bandwidth_event::Event::RemainingBandwidth(
                    proto::mixnet_event::bandwidth_event::RemainingBandwidth { value },
                )
            }
        };
        Self { event: Some(event) }
    }
}

impl From<ConnectionEvent> for proto::mixnet_event::ConnectionEvent {
    fn from(value: ConnectionEvent) -> Self {
        match value {
            ConnectionEvent::EntryGatewayDown => Self::EntryGatewayDown,
            ConnectionEvent::ExitGatewayDownIpv4 => Self::ExitGatewayDownIpv4,
            ConnectionEvent::ExitGatewayDownIpv6 => Self::ExitGatewayDownIpv6,
            ConnectionEvent::ExitGatewayRoutingErrorIpv4 => Self::ExitGatewayRoutingErrorIpv4,
            ConnectionEvent::ExitGatewayRoutingErrorIpv6 => Self::ExitGatewayRoutingErrorIpv6,
            ConnectionEvent::ConnectedIpv4 => Self::ConnectedIpv4,
            ConnectionEvent::ConnectedIpv6 => Self::ConnectedIpv6,
        }
    }
}

impl From<ConnectionStatisticsEvent> for proto::mixnet_event::ConnectionStatisticsEvent {
    fn from(value: ConnectionStatisticsEvent) -> Self {
        Self {
            rates: Some(proto::mixnet_event::SphinxPacketRates::from(value.rates)),
        }
    }
}

impl From<SphinxPacketRates> for proto::mixnet_event::SphinxPacketRates {
    fn from(value: SphinxPacketRates) -> Self {
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
