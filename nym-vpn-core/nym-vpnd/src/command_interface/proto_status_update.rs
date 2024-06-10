// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::connection_monitor::ConnectionMonitorStatus;
use nym_vpn_proto::{connection_status_update::StatusType, ConnectionStatusUpdate};

pub(super) fn status_update_from_task_status(
    status: &nym_vpn_lib::TaskStatus,
) -> ConnectionStatusUpdate {
    match status {
        nym_vpn_lib::TaskStatus::Ready => ConnectionStatusUpdate {
            kind: StatusType::TunnelEndToEndConnectionEstablished as i32,
            message: status.to_string(),
            details: Default::default(),
        },
        nym_vpn_lib::TaskStatus::ReadyWithGateway(ref gateway) => ConnectionStatusUpdate {
            kind: StatusType::TunnelEndToEndConnectionEstablished as i32,
            message: status.to_string(),
            details: maplit::hashmap! {
                "entry_gateway".to_string() => gateway.to_string(),
            },
        },
    }
}

pub(super) fn status_update_from_monitor_status(
    status: &ConnectionMonitorStatus,
) -> ConnectionStatusUpdate {
    match status {
        ConnectionMonitorStatus::EntryGatewayDown => ConnectionStatusUpdate {
            kind: StatusType::EntryGatewayNotRoutingMixnetMessages as i32,
            message: status.to_string(),
            details: Default::default(),
        },
        ConnectionMonitorStatus::ExitGatewayDownIpv4 => ConnectionStatusUpdate {
            kind: StatusType::ExitRouterNotRespondingToIpv4Ping as i32,
            message: status.to_string(),
            details: Default::default(),
        },
        ConnectionMonitorStatus::ExitGatewayDownIpv6 => ConnectionStatusUpdate {
            kind: StatusType::ExitRouterNotRespondingToIpv6Ping as i32,
            message: status.to_string(),
            details: Default::default(),
        },
        ConnectionMonitorStatus::ExitGatewayRoutingErrorIpv4 => ConnectionStatusUpdate {
            kind: StatusType::ExitRouterNotRoutingIpv4Traffic as i32,
            message: status.to_string(),
            details: Default::default(),
        },
        ConnectionMonitorStatus::ExitGatewayRoutingErrorIpv6 => ConnectionStatusUpdate {
            kind: StatusType::ExitRouterNotRoutingIpv6Traffic as i32,
            message: status.to_string(),
            details: Default::default(),
        },
        ConnectionMonitorStatus::ConnectedIpv4 => ConnectionStatusUpdate {
            kind: StatusType::ConnectionOkIpv4 as i32,
            message: status.to_string(),
            details: Default::default(),
        },
        ConnectionMonitorStatus::ConnectedIpv6 => ConnectionStatusUpdate {
            kind: StatusType::ConnectionOkIpv6 as i32,
            message: status.to_string(),
            details: Default::default(),
        },
    }
}
