// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::{
    connection_monitor::ConnectionMonitorStatus,
    tunnel_state_machine::{BandwidthEvent, ConnectionEvent, MixnetEvent},
};
use nym_vpn_proto::{connection_status_update::StatusType, ConnectionStatusUpdate};

pub fn status_update_from_event(event: MixnetEvent) -> ConnectionStatusUpdate {
    match event {
        MixnetEvent::Bandwidth(sub_event) => convert_bandwidth_event(sub_event),
        MixnetEvent::Connection(sub_event) => convert_connection_event(sub_event),
    }
}

fn convert_connection_event(event: ConnectionEvent) -> ConnectionStatusUpdate {
    // todo: cut reliance on ConnectionMonitorStatus for producing messages.
    match event {
        ConnectionEvent::EntryGatewayDown => ConnectionStatusUpdate {
            kind: StatusType::EntryGatewayNotRoutingMixnetMessages as i32,
            message: ConnectionMonitorStatus::EntryGatewayDown.to_string(),
            details: Default::default(),
        },
        ConnectionEvent::ExitGatewayDownIpv4 => ConnectionStatusUpdate {
            kind: StatusType::ExitRouterNotRespondingToIpv4Ping as i32,
            message: ConnectionMonitorStatus::ExitGatewayDownIpv4.to_string(),
            details: Default::default(),
        },
        ConnectionEvent::ExitGatewayDownIpv6 => ConnectionStatusUpdate {
            kind: StatusType::ExitRouterNotRespondingToIpv6Ping as i32,
            message: ConnectionMonitorStatus::ExitGatewayDownIpv6.to_string(),
            details: Default::default(),
        },
        ConnectionEvent::ExitGatewayRoutingErrorIpv4 => ConnectionStatusUpdate {
            kind: StatusType::ExitRouterNotRoutingIpv4Traffic as i32,
            message: ConnectionMonitorStatus::ExitGatewayRoutingErrorIpv4.to_string(),
            details: Default::default(),
        },
        ConnectionEvent::ExitGatewayRoutingErrorIpv6 => ConnectionStatusUpdate {
            kind: StatusType::ExitRouterNotRoutingIpv6Traffic as i32,
            message: ConnectionMonitorStatus::ExitGatewayRoutingErrorIpv6.to_string(),
            details: Default::default(),
        },
        ConnectionEvent::ConnectedIpv4 => ConnectionStatusUpdate {
            kind: StatusType::ConnectionOkIpv4 as i32,
            message: ConnectionMonitorStatus::ConnectedIpv4.to_string(),
            details: Default::default(),
        },
        ConnectionEvent::ConnectedIpv6 => ConnectionStatusUpdate {
            kind: StatusType::ConnectionOkIpv6 as i32,
            message: ConnectionMonitorStatus::ConnectedIpv6.to_string(),
            details: Default::default(),
        },
    }
}

fn convert_bandwidth_event(event: BandwidthEvent) -> ConnectionStatusUpdate {
    match event {
        BandwidthEvent::RemainingBandwidth(amount) => ConnectionStatusUpdate {
            kind: StatusType::RemainingBandwidth as i32,
            message: format!("remaining bandwidth: {}", amount),
            details: maplit::hashmap! {
                "amount".to_string() => amount.to_string(),
            },
        },
        BandwidthEvent::NoBandwidth => ConnectionStatusUpdate {
            kind: StatusType::NoBandwidth as i32,
            message: "no bandwidth left".to_owned(),
            details: Default::default(),
        },
    }
}
