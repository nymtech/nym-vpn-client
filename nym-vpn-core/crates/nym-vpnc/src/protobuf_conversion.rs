// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{EntryPoint, ExitPoint, GatewayType};

pub fn into_entry_point(entry: EntryPoint) -> nym_vpn_proto::EntryNode {
    match entry {
        EntryPoint::Gateway { identity } => nym_vpn_proto::EntryNode::from(&identity),
        EntryPoint::Location { location } => nym_vpn_proto::EntryNode::new_from_location(&location),
        EntryPoint::RandomLowLatency => nym_vpn_proto::EntryNode::new_random_low_latency(),
        EntryPoint::Random => nym_vpn_proto::EntryNode::new_random(),
    }
}

pub fn into_exit_point(exit: ExitPoint) -> nym_vpn_proto::ExitNode {
    match exit {
        ExitPoint::Address { address } => nym_vpn_proto::ExitNode::from(&address),
        ExitPoint::Gateway { identity } => nym_vpn_proto::ExitNode::from(&identity),
        ExitPoint::Location { location } => nym_vpn_proto::ExitNode::new_from_location(&location),
        ExitPoint::Random => nym_vpn_proto::ExitNode::new_random(),
    }
}

pub fn into_gateway_type(gateway_type: GatewayType) -> nym_vpn_proto::GatewayType {
    match gateway_type {
        GatewayType::MixnetEntry => nym_vpn_proto::GatewayType::MixnetEntry,
        GatewayType::MixnetExit => nym_vpn_proto::GatewayType::MixnetExit,
        GatewayType::Wg => nym_vpn_proto::GatewayType::Wg,
    }
}
