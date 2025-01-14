// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use si_scale::helpers::bibytes2;

use nym_bandwidth_controller::BandwidthStatusMessage;
use nym_connection_monitor::ConnectionMonitorStatus;
use nym_statistics_common::clients::packet_statistics::{
    MixnetBandwidthStatisticsEvent, PacketRates,
};

use super::tunnel_state::TunnelState;

#[derive(Debug, Clone)]
pub enum TunnelEvent {
    NewState(TunnelState),
    MixnetState(MixnetEvent),
}

impl fmt::Display for TunnelEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NewState(new_state) => new_state.fmt(f),
            Self::MixnetState(event) => event.fmt(f),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MixnetEvent {
    Bandwidth(BandwidthEvent),
    Connection(ConnectionEvent),
    ConnectionStatistics(ConnectionStatisticsEvent),
}

impl fmt::Display for MixnetEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bandwidth(event) => write!(f, "{}", event),
            Self::Connection(event) => write!(f, "{}", event),
            Self::ConnectionStatistics(event) => write!(f, "{}", event),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BandwidthEvent {
    NoBandwidth,
    RemainingBandwidth(i64),
}

impl fmt::Display for BandwidthEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoBandwidth => f.write_str("No bandwidth"),
            Self::RemainingBandwidth(value) => {
                write!(
                    f,
                    "Remaining bandwidth: {}",
                    si_scale::helpers::bibytes2(*value as f64)
                )
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ConnectionEvent {
    EntryGatewayDown,
    ExitGatewayDownIpv4,
    ExitGatewayDownIpv6,
    ExitGatewayRoutingErrorIpv4,
    ExitGatewayRoutingErrorIpv6,
    ConnectedIpv4,
    ConnectedIpv6,
}

impl fmt::Display for ConnectionEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ConnectedIpv4 => "Connected with IPv4",
            Self::ConnectedIpv6 => "Connected with IPv6",
            Self::EntryGatewayDown => {
                "Entry gateway appears down - it's not routing our mixnet traffic"
            }
            Self::ExitGatewayDownIpv4 => "Exit gateway (or ipr) appears down - it's not responding to IPv4 traffic",
            Self::ExitGatewayDownIpv6 => "Exit gateway (or ipr) appears down - it's not responding to IPv6 traffic",
            Self::ExitGatewayRoutingErrorIpv4 => "Exit gateway (or ipr) appears to be having issues routing and forwarding our external IPv4 traffic",
            Self::ExitGatewayRoutingErrorIpv6 => "Exit gateway (or ipr) appears to be having issues routing and forwarding our external IPv6 traffic",
        };

        f.write_str(s)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ConnectionStatisticsEvent {
    pub rates: SphinxPacketRates,
}

impl fmt::Display for ConnectionStatisticsEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rates)
    }
}

#[derive(Debug, Copy, Clone)]
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

impl fmt::Display for SphinxPacketRates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

impl SphinxPacketRates {
    pub fn summary(&self) -> String {
        format!(
            "down: {}/s, up: {}/s (cover down: {}/s, cover up: {}/s)",
            bibytes2(self.real_packets_received_size),
            bibytes2(self.real_packets_sent_size),
            bibytes2(self.cover_packets_received_size),
            bibytes2(self.cover_packets_sent_size),
        )
    }

    pub fn real_received(&self) -> String {
        bibytes2(self.real_packets_received_size)
    }

    pub fn real_sent(&self) -> String {
        bibytes2(self.real_packets_sent_size)
    }

    pub fn cover_received(&self) -> String {
        bibytes2(self.cover_packets_received_size)
    }

    pub fn cover_sent(&self) -> String {
        bibytes2(self.cover_packets_sent_size)
    }
}

impl From<&BandwidthStatusMessage> for BandwidthEvent {
    fn from(value: &BandwidthStatusMessage) -> Self {
        match value {
            BandwidthStatusMessage::NoBandwidth => Self::NoBandwidth,
            BandwidthStatusMessage::RemainingBandwidth(value) => Self::RemainingBandwidth(*value),
        }
    }
}

impl From<&ConnectionMonitorStatus> for ConnectionEvent {
    fn from(value: &ConnectionMonitorStatus) -> Self {
        match value {
            ConnectionMonitorStatus::ConnectedIpv4 => Self::ConnectedIpv4,
            ConnectionMonitorStatus::ConnectedIpv6 => Self::ConnectedIpv6,
            ConnectionMonitorStatus::EntryGatewayDown => Self::EntryGatewayDown,
            ConnectionMonitorStatus::ExitGatewayDownIpv4 => Self::ExitGatewayDownIpv4,
            ConnectionMonitorStatus::ExitGatewayDownIpv6 => Self::ExitGatewayDownIpv6,
            ConnectionMonitorStatus::ExitGatewayRoutingErrorIpv4 => {
                Self::ExitGatewayRoutingErrorIpv4
            }
            ConnectionMonitorStatus::ExitGatewayRoutingErrorIpv6 => {
                Self::ExitGatewayRoutingErrorIpv6
            }
        }
    }
}

impl From<&MixnetBandwidthStatisticsEvent> for ConnectionStatisticsEvent {
    fn from(value: &MixnetBandwidthStatisticsEvent) -> Self {
        Self {
            rates: SphinxPacketRates::from(value.rates.clone()),
        }
    }
}

impl From<PacketRates> for SphinxPacketRates {
    fn from(value: PacketRates) -> Self {
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
