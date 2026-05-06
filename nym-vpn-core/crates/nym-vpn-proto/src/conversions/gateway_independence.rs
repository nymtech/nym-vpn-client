// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::proto;

impl From<nym_vpn_lib_types::GatewayIndependence> for proto::GatewayIndependence {
    fn from(value: nym_vpn_lib_types::GatewayIndependence) -> Self {
        Self {
            different_node_family: value.different_node_family,
            different_asn: value.different_asn,
        }
    }
}

impl From<proto::GatewayIndependence> for nym_vpn_lib_types::GatewayIndependence {
    fn from(value: proto::GatewayIndependence) -> Self {
        Self {
            different_node_family: value.different_node_family,
            different_asn: value.different_asn,
        }
    }
}
