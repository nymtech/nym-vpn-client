use serde::Serialize;
use ts_rs::TS;

use nym_vpn_proto as p;
use p::mixnet_event::{bandwidth_event::Event as BandwidthEventEvent, BandwidthEvent, Event};
use tracing::{error, instrument};

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum MixnetEvent {
    EntryGwDown,
    ExitGwDownIpv4,
    ExitGwDownIpv6,
    ExitGwRoutingErrorIpv4,
    ExitGwRoutingErrorIpv6,
    ConnectedIpv4,
    ConnectedIpv6,
    NoBandwidth,
    RemainingBandwidth(i64),
    SphinxPacketMetrics, // TODO include metrics
}

impl MixnetEvent {
    #[instrument(skip(mixnet_event))]
    pub fn from_proto(mixnet_event: p::MixnetEvent) -> Option<Self> {
        let Some(event) = mixnet_event.event else {
            error!("no event data");
            return None;
        };
        match event {
            Event::BandwidthEvent(b) => MixnetEvent::from_bandwidth_event(b),
            Event::ConnectionEvent(event) => MixnetEvent::from_connection_event(event),
            Event::ConnectionStatisticsEvent(_) => Some(MixnetEvent::SphinxPacketMetrics),
        }
    }

    #[instrument]
    pub fn from_bandwidth_event(bandwidth_event: BandwidthEvent) -> Option<Self> {
        let Some(event) = bandwidth_event.event else {
            error!("no event data");
            return None;
        };
        match event {
            BandwidthEventEvent::NoBandwidth(_) => Some(Self::NoBandwidth),
            BandwidthEventEvent::RemainingBandwidth(b) => Some(Self::RemainingBandwidth(b.value)),
        }
    }

    #[instrument]
    pub fn from_connection_event(event: i32) -> Option<Self> {
        if !(0..=6).contains(&event) {
            error!("invalid connection event");
            return None;
        }
        match event {
            0 => Some(Self::EntryGwDown),
            1 => Some(Self::ExitGwDownIpv4),
            2 => Some(Self::ExitGwDownIpv6),
            3 => Some(Self::ExitGwRoutingErrorIpv4),
            4 => Some(Self::ExitGwRoutingErrorIpv6),
            5 => Some(Self::ConnectedIpv4),
            6 => Some(Self::ConnectedIpv6),
            _ => unreachable!(),
        }
    }
}
