// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_bandwidth_controller::BandwidthStatusMessage;
use nym_bandwidth_controller_pre_ecash::BandwidthStatusMessage as LegacyBandwidthStatusMessage;
use nym_vpn_lib::{connection_monitor::ConnectionMonitorStatus, NymVpnStatusMessage};
use nym_vpn_proto::{connection_status_update::StatusType, ConnectionStatusUpdate};

pub(crate) fn status_update_from_status_message(
    status: &NymVpnStatusMessage,
) -> ConnectionStatusUpdate {
    match status {
        NymVpnStatusMessage::MixConnectionInfo {
            mixnet_connection_info,
            mixnet_exit_connection_info,
        } => ConnectionStatusUpdate {
            kind: StatusType::TunnelEndToEndConnectionEstablished as i32,
            message: status.to_string(),
            details: maplit::hashmap! {
                "nym_address".to_string() => mixnet_connection_info.nym_address.to_string(),
                "entry_gateway".to_string() => mixnet_connection_info.entry_gateway.to_base58_string(),
                "exit_gateway".to_string() => mixnet_exit_connection_info.exit_gateway.to_base58_string(),
                "exit_ipr".to_string() => mixnet_exit_connection_info.exit_ipr.to_string(),
                "ipv4".to_string() => mixnet_exit_connection_info.ips.ipv4.to_string(),
                "ipv6".to_string() => mixnet_exit_connection_info.ips.ipv6.to_string(),
            },
        },
        NymVpnStatusMessage::WgConnectionInfo {
            entry_connection_info,
            exit_connection_info,
        } => ConnectionStatusUpdate {
            kind: StatusType::TunnelEndToEndConnectionEstablished as i32,
            message: status.to_string(),
            details: maplit::hashmap! {
                "entry_gateway".to_string() => entry_connection_info.gateway_id.to_base58_string(),
                "exit_gateway".to_string() => exit_connection_info.gateway_id.to_base58_string(),
                "entry_public_key".to_string() => entry_connection_info.public_key.clone(),
                "exit_public_key".to_string() => exit_connection_info.public_key.clone(),
                "entry_private_ipv4".to_string() => entry_connection_info.private_ipv4.to_string(),
                "exit_private_ipv4".to_string() => exit_connection_info.private_ipv4.to_string(),
            },
        },
    }
}

pub(crate) fn status_update_from_monitor_status(
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

pub(crate) fn status_update_from_bandwidth_status_message(
    status: &BandwidthStatusMessage,
) -> ConnectionStatusUpdate {
    match status {
        BandwidthStatusMessage::RemainingBandwidth(amount) => ConnectionStatusUpdate {
            kind: StatusType::RemainingBandwidth as i32,
            message: status.to_string(),
            details: maplit::hashmap! {
                "amount".to_string() => amount.to_string(),
            },
        },
        BandwidthStatusMessage::NoBandwidth => ConnectionStatusUpdate {
            kind: StatusType::NoBandwidth as i32,
            message: status.to_string(),
            details: Default::default(),
        },
    }
}

// Temporary while we depend on a pre-cash rev of the bandwidth controller
pub(crate) fn status_update_from_bandwidth_status_message_legacy(
    status: &LegacyBandwidthStatusMessage,
) -> ConnectionStatusUpdate {
    match status {
        LegacyBandwidthStatusMessage::RemainingBandwidth(amount) => ConnectionStatusUpdate {
            kind: StatusType::RemainingBandwidth as i32,
            message: status.to_string(),
            details: maplit::hashmap! {
                "amount".to_string() => amount.to_string(),
            },
        },
        LegacyBandwidthStatusMessage::NoBandwidth => ConnectionStatusUpdate {
            kind: StatusType::NoBandwidth as i32,
            message: status.to_string(),
            details: Default::default(),
        },
    }
}

pub(crate) fn status_update_from_wg_tunnel_error_event(
    status: &nym_vpn_lib::WgTunnelErrorEvent,
) -> ConnectionStatusUpdate {
    ConnectionStatusUpdate {
        kind: StatusType::WgTunnelError as i32,
        message: status.to_string(),
        details: Default::default(),
    }
}
