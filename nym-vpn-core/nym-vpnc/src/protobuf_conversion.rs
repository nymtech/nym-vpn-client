// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use nym_gateway_directory::{EntryPoint, ExitPoint, NodeIdentity, Recipient};

fn new_entry_node_gateway(identity: &NodeIdentity) -> nym_vpn_proto::EntryNode {
    nym_vpn_proto::EntryNode {
        entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::Gateway(
            nym_vpn_proto::Gateway {
                id: identity.to_base58_string(),
            },
        )),
    }
}

fn new_entry_node_location(country_code: &str) -> nym_vpn_proto::EntryNode {
    nym_vpn_proto::EntryNode {
        entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::Location(
            nym_vpn_proto::Location {
                two_letter_iso_country_code: country_code.to_string(),
            },
        )),
    }
}

fn new_entry_node_random_low_latency() -> nym_vpn_proto::EntryNode {
    nym_vpn_proto::EntryNode {
        entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::RandomLowLatency(
            nym_vpn_proto::Empty {},
        )),
    }
}

fn new_entry_node_random() -> nym_vpn_proto::EntryNode {
    nym_vpn_proto::EntryNode {
        entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::Random(
            nym_vpn_proto::Empty {},
        )),
    }
}

pub(crate) fn into_entry_point(entry: EntryPoint) -> nym_vpn_proto::EntryNode {
    match entry {
        EntryPoint::Gateway { identity } => new_entry_node_gateway(&identity),
        EntryPoint::Location { location } => new_entry_node_location(&location),
        EntryPoint::RandomLowLatency => new_entry_node_random_low_latency(),
        EntryPoint::Random => new_entry_node_random(),
    }
}

fn new_exit_node_address(address: &Recipient) -> nym_vpn_proto::ExitNode {
    nym_vpn_proto::ExitNode {
        exit_node_enum: Some(nym_vpn_proto::exit_node::ExitNodeEnum::Address(
            nym_vpn_proto::Address {
                nym_address: address.to_string(),
            },
        )),
    }
}

fn new_exit_node_gateway(identity: &NodeIdentity) -> nym_vpn_proto::ExitNode {
    nym_vpn_proto::ExitNode {
        exit_node_enum: Some(nym_vpn_proto::exit_node::ExitNodeEnum::Gateway(
            nym_vpn_proto::Gateway {
                id: identity.to_base58_string(),
            },
        )),
    }
}

fn new_exit_node_location(country_code: &str) -> nym_vpn_proto::ExitNode {
    nym_vpn_proto::ExitNode {
        exit_node_enum: Some(nym_vpn_proto::exit_node::ExitNodeEnum::Location(
            nym_vpn_proto::Location {
                two_letter_iso_country_code: country_code.to_string(),
            },
        )),
    }
}

fn new_exit_node_random() -> nym_vpn_proto::ExitNode {
    nym_vpn_proto::ExitNode {
        exit_node_enum: Some(nym_vpn_proto::exit_node::ExitNodeEnum::Random(
            nym_vpn_proto::Empty {},
        )),
    }
}

pub(crate) fn into_exit_point(exit: ExitPoint) -> nym_vpn_proto::ExitNode {
    match exit {
        ExitPoint::Address { address } => new_exit_node_address(&address),
        ExitPoint::Gateway { identity } => new_exit_node_gateway(&identity),
        ExitPoint::Location { location } => new_exit_node_location(&location),
        ExitPoint::Random => new_exit_node_random(),
    }
}

pub(crate) fn ipaddr_into_string(ip: std::net::IpAddr) -> nym_vpn_proto::Dns {
    nym_vpn_proto::Dns { ip: ip.to_string() }
}

pub(crate) fn into_threshold(performance: u8) -> nym_vpn_proto::Threshold {
    nym_vpn_proto::Threshold {
        min_performance: performance.into(),
    }
}

pub(crate) fn parse_offset_datetime(
    timestamp: prost_types::Timestamp,
) -> Result<time::OffsetDateTime, time::Error> {
    time::OffsetDateTime::from_unix_timestamp(timestamp.seconds)
        .map(|t| t + time::Duration::nanoseconds(timestamp.nanos as i64))
        .map_err(time::Error::from)
}
