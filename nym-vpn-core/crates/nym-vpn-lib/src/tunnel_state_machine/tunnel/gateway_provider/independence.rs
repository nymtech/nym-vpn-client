// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::Gateway;
use nym_vpn_lib_types::GatewayIndependence;

pub(crate) fn gateways_are_independent(
    gw1: &Gateway,
    gw2: &Gateway,
    criteria: GatewayIndependence,
) -> bool {
    if gw1.identity() == gw2.identity() {
        return false;
    }
    if criteria.different_asn {
        let (Some(asn1), Some(asn2)) = (
            gw1.location.as_ref().and_then(|l| l.asn.as_ref()),
            gw2.location.as_ref().and_then(|l| l.asn.as_ref()),
        ) else {
            // all gateways should have a ASN, if they don't we assume they can't be independent
            return false;
        };
        if asn1.asn == asn2.asn {
            return false;
        }
    }
    if criteria.different_node_family
        && let (Some(nf1), Some(nf2)) = (&gw1.node_family, &gw2.node_family)
        && nf1.id == nf2.id
    {
        return false;
    }

    true
}
