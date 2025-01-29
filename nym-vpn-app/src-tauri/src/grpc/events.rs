use serde::Serialize;
use ts_rs::TS;

use nym_vpn_proto as p;
use p::mixnet_event::{
    bandwidth_event::Event as BandwidthEventEvent, BandwidthEvent, ConnectionEvent, Event,
};
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
        if let Ok(e) = ConnectionEvent::try_from(event)
            .inspect_err(|e| error!("invalid connection event [{}], {}", event, e))
        {
            match e {
                ConnectionEvent::EntryGatewayDown => Some(Self::EntryGwDown),
                ConnectionEvent::ExitGatewayDownIpv4 => Some(Self::ExitGwDownIpv4),
                ConnectionEvent::ExitGatewayDownIpv6 => Some(Self::ExitGwDownIpv6),
                ConnectionEvent::ExitGatewayRoutingErrorIpv4 => Some(Self::ExitGwRoutingErrorIpv4),
                ConnectionEvent::ExitGatewayRoutingErrorIpv6 => Some(Self::ExitGwRoutingErrorIpv6),
                ConnectionEvent::ConnectedIpv4 => Some(Self::ConnectedIpv4),
                ConnectionEvent::ConnectedIpv6 => Some(Self::ConnectedIpv6),
            }
        } else {
            None
        }
    }
}
