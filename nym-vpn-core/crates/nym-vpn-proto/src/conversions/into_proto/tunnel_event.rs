// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    BandwidthEvent, ConnectionEvent, ConnectionStatisticsEvent, MixnetEvent, SphinxPacketRates,
    TunnelEvent, TunnelState,
};

use crate::{
    conversions::ConversionError,
    mixnet_event::{
        bandwidth_event::{
            Event as ProtoBanwidthEventEnum, NoBandwidth as ProtoNoBandwidth,
            RemainingBandwidth as ProtoRemainingBandwidth,
        },
        BandwidthEvent as ProtoBandwidthEvent, ConnectionEvent as ProtoConnectionEvent,
        ConnectionStatisticsEvent as ProtoConnectionStatisticsEvent, Event as ProtoMixnetEventEnum,
        SphinxPacketRates as ProtoSphinxPacketRates,
    },
    tunnel_event::Event as ProtoTunnelEventEnum,
    MixnetEvent as ProtoMixnetEvent, TunnelEvent as ProtoTunnelEvent,
};

impl TryFrom<ProtoTunnelEvent> for TunnelEvent {
    type Error = ConversionError;

    fn try_from(value: ProtoTunnelEvent) -> Result<Self, Self::Error> {
        let event = value
            .event
            .ok_or(ConversionError::NoValueSet("TunnelEvent.event"))?;

        Ok(match event {
            ProtoTunnelEventEnum::TunnelState(tunnel_state) => {
                TunnelEvent::NewState(TunnelState::try_from(tunnel_state)?)
            }
            ProtoTunnelEventEnum::MixnetEvent(mixnet_event) => {
                TunnelEvent::MixnetState(MixnetEvent::try_from(mixnet_event)?)
            }
        })
    }
}

impl TryFrom<ProtoMixnetEvent> for MixnetEvent {
    type Error = ConversionError;

    fn try_from(value: ProtoMixnetEvent) -> Result<Self, Self::Error> {
        let event = value
            .event
            .ok_or(ConversionError::NoValueSet("MixnetEvent.event"))?;

        Ok(match event {
            ProtoMixnetEventEnum::BandwidthEvent(bandwidth_event) => {
                Self::Bandwidth(BandwidthEvent::try_from(bandwidth_event)?)
            }
            ProtoMixnetEventEnum::ConnectionEvent(connection_event) => {
                let proto_connection_event = ProtoConnectionEvent::try_from(connection_event)
                    .map_err(|e| ConversionError::Decode("ConnectionEvent", e))?;
                Self::Connection(ConnectionEvent::from(proto_connection_event))
            }
            ProtoMixnetEventEnum::ConnectionStatisticsEvent(connection_statistics_event) => {
                Self::ConnectionStatistics(ConnectionStatisticsEvent::try_from(
                    connection_statistics_event,
                )?)
            }
        })
    }
}

impl From<ProtoConnectionEvent> for ConnectionEvent {
    fn from(value: ProtoConnectionEvent) -> Self {
        match value {
            ProtoConnectionEvent::EntryGatewayDown => Self::EntryGatewayDown,
            ProtoConnectionEvent::ExitGatewayDownIpv4 => Self::ExitGatewayDownIpv4,
            ProtoConnectionEvent::ExitGatewayDownIpv6 => Self::ExitGatewayDownIpv6,
            ProtoConnectionEvent::ExitGatewayRoutingErrorIpv4 => Self::ExitGatewayRoutingErrorIpv4,
            ProtoConnectionEvent::ExitGatewayRoutingErrorIpv6 => Self::ExitGatewayRoutingErrorIpv6,
            ProtoConnectionEvent::ConnectedIpv4 => Self::ConnectedIpv4,
            ProtoConnectionEvent::ConnectedIpv6 => Self::ConnectedIpv6,
        }
    }
}

impl TryFrom<ProtoBandwidthEvent> for BandwidthEvent {
    type Error = ConversionError;

    fn try_from(value: ProtoBandwidthEvent) -> Result<Self, Self::Error> {
        let event = value
            .event
            .ok_or(ConversionError::NoValueSet("BandwidthEvent.event"))?;

        Ok(match event {
            ProtoBanwidthEventEnum::NoBandwidth(ProtoNoBandwidth {}) => Self::NoBandwidth,
            ProtoBanwidthEventEnum::RemainingBandwidth(ProtoRemainingBandwidth { value }) => {
                Self::RemainingBandwidth(value)
            }
        })
    }
}

impl TryFrom<ProtoConnectionStatisticsEvent> for ConnectionStatisticsEvent {
    type Error = ConversionError;

    fn try_from(value: ProtoConnectionStatisticsEvent) -> Result<Self, Self::Error> {
        let rates = value.rates.ok_or(ConversionError::NoValueSet(
            "ConnectionStatisticsEvent.rates",
        ))?;
        Ok(Self {
            rates: SphinxPacketRates::from(rates),
        })
    }
}

impl From<ProtoSphinxPacketRates> for SphinxPacketRates {
    fn from(value: ProtoSphinxPacketRates) -> Self {
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
