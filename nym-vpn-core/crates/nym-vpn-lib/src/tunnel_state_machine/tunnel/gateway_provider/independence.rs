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
    // node family not present is assumed that they are independent, as no node family is the default node configuration
    if criteria.different_node_family
        && let (Some(nf1), Some(nf2)) = (&gw1.family_data, &gw2.family_data)
        && nf1.id == nf2.id
    {
        return false;
    }
    if criteria.different_subnet {
        let (Some(route1), Some(route2)) = (
            gw1.location
                .as_ref()
                .and_then(|l| l.asn.as_ref())
                .map(|asn| &asn.route),
            gw2.location
                .as_ref()
                .and_then(|l| l.asn.as_ref())
                .map(|asn| &asn.route),
        ) else {
            // all gateways should have a ASN with a route, if they don't we assume they can't be independent
            return false;
        };
        if match (route1, route2) {
            (ipnetwork::IpNetwork::V4(v4_route1), ipnetwork::IpNetwork::V4(v4_route2)) => {
                v4_route1.overlaps(*v4_route2)
            }
            (ipnetwork::IpNetwork::V6(v6_route1), ipnetwork::IpNetwork::V6(v6_route2)) => {
                v6_route1.overlaps(*v6_route2)
            }
            _ => false,
        } {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use ipnetwork::IpNetwork;
    use nym_gateway_directory::{Asn, AsnKind, Gateway, Location};
    use nym_vpn_api_client::response::NodeFamily;

    use super::*;

    const GW_ID_1: &str = "2zHiExNRKiCXVKS35SNKtK4apGfZELMpA1jJ2gVevJoz";
    const GW_ID_2: &str = "38zcSsvjXsAX7C28ko2H3Lt55X4TYxfZYkPADxKXZHUj";

    fn make_gateway(id: &str) -> Gateway {
        Gateway::builder().identity(id.parse().unwrap()).build()
    }

    fn make_gateway_with_asn(id: &str, asn_number: &str) -> Gateway {
        Gateway::builder()
            .identity(id.parse().unwrap())
            .location(Location {
                asn: Some(Asn {
                    asn: asn_number.to_string(),
                    name: "Test ISP".to_string(),
                    route: "10.10.10.10/16".parse().unwrap(),
                    kind: AsnKind::Other,
                }),
                ..Default::default()
            })
            .build()
    }

    fn make_gateway_with_family(id: &str, family_id: u32) -> Gateway {
        Gateway::builder()
            .identity(id.parse().unwrap())
            .family_data(Some(NodeFamily {
                id: family_id,
                name: "Test Family".to_string(),
                description: String::new(),
                family_stake: 0,
                members: 0,
            }))
            .build()
    }

    fn make_gateway_with_subnet(id: &str, route: IpNetwork) -> Gateway {
        Gateway::builder()
            .identity(id.parse().unwrap())
            .location(Location {
                asn: Some(Asn {
                    asn: "ASTEST".to_string(),
                    name: "Test ISP".to_string(),
                    route,
                    kind: AsnKind::Other,
                }),
                ..Default::default()
            })
            .build()
    }

    fn asn_only() -> GatewayIndependence {
        GatewayIndependence {
            different_asn: true,
            different_node_family: false,
            different_subnet: false,
            ..Default::default()
        }
    }

    fn family_only() -> GatewayIndependence {
        GatewayIndependence {
            different_asn: false,
            different_node_family: true,
            different_subnet: false,
            ..Default::default()
        }
    }

    fn subnet_only() -> GatewayIndependence {
        GatewayIndependence {
            different_asn: false,
            different_node_family: false,
            different_subnet: true,
            ..Default::default()
        }
    }

    #[test]
    fn same_identity_not_independent_regardless_of_criteria() {
        let gw = make_gateway(GW_ID_1);
        assert!(!gateways_are_independent(
            &gw,
            &gw,
            GatewayIndependence {
                different_node_family: false,
                different_asn: false,
                different_subnet: false,
                ..Default::default()
            }
        ));
        assert!(!gateways_are_independent(&gw, &gw, asn_only()));
        assert!(!gateways_are_independent(&gw, &gw, family_only()));
        assert!(!gateways_are_independent(&gw, &gw, subnet_only()));
        assert!(!gateways_are_independent(
            &gw,
            &gw,
            GatewayIndependence::default()
        ));
    }

    #[test]
    fn different_identity_no_criteria_is_independent() {
        let gw1 = make_gateway(GW_ID_1);
        let gw2 = make_gateway(GW_ID_2);
        assert!(gateways_are_independent(
            &gw1,
            &gw2,
            GatewayIndependence {
                different_node_family: false,
                different_asn: false,
                different_subnet: false,
                ..Default::default()
            }
        ));
    }

    #[test]
    fn same_asn_not_independent_when_asn_criterion_active() {
        let gw1 = make_gateway_with_asn(GW_ID_1, "AS12345");
        let gw2 = make_gateway_with_asn(GW_ID_2, "AS12345");
        assert!(!gateways_are_independent(&gw1, &gw2, asn_only()));
    }

    #[test]
    fn different_subnets_independent_when_subnet_criterion_active() {
        let gw1 = make_gateway_with_subnet(GW_ID_1, "10.10.10.10/16".parse().unwrap());
        let gw2 = make_gateway_with_subnet(GW_ID_2, "10.11.10.10/16".parse().unwrap());
        assert!(gateways_are_independent(&gw1, &gw2, subnet_only()));
    }

    #[test]
    fn missing_subnet_not_independent_when_subnet_criterion_active() {
        let gw1 = make_gateway(GW_ID_1);
        let gw2 = make_gateway(GW_ID_2);
        assert!(!gateways_are_independent(&gw1, &gw2, subnet_only()));
    }

    #[test]
    fn one_missing_subnet_not_independent_when_subnet_criterion_active() {
        let gw1 = make_gateway_with_asn(GW_ID_1, "AS12345");
        let gw2 = make_gateway(GW_ID_2);
        assert!(!gateways_are_independent(&gw1, &gw2, subnet_only()));
    }

    #[test]
    fn same_subnet_not_independent_when_subnet_criterion_active() {
        let gw1 = make_gateway_with_asn(GW_ID_1, "AS12345");
        let gw2 = make_gateway_with_asn(GW_ID_2, "AS12345");
        assert!(!gateways_are_independent(&gw1, &gw2, subnet_only()));
    }

    #[test]
    fn different_asns_independent_when_asn_criterion_active() {
        let gw1 = make_gateway_with_asn(GW_ID_1, "AS12345");
        let gw2 = make_gateway_with_asn(GW_ID_2, "AS99999");
        assert!(gateways_are_independent(&gw1, &gw2, asn_only()));
    }

    #[test]
    fn missing_asn_not_independent_when_asn_criterion_active() {
        let gw1 = make_gateway(GW_ID_1);
        let gw2 = make_gateway(GW_ID_2);
        assert!(!gateways_are_independent(&gw1, &gw2, asn_only()));
    }

    #[test]
    fn one_missing_asn_not_independent_when_asn_criterion_active() {
        let gw1 = make_gateway_with_asn(GW_ID_1, "AS12345");
        let gw2 = make_gateway(GW_ID_2);
        assert!(!gateways_are_independent(&gw1, &gw2, asn_only()));
    }

    #[test]
    fn same_family_not_independent_when_family_criterion_active() {
        let gw1 = make_gateway_with_family(GW_ID_1, 42);
        let gw2 = make_gateway_with_family(GW_ID_2, 42);
        assert!(!gateways_are_independent(&gw1, &gw2, family_only()));
    }

    #[test]
    fn different_family_independent_when_family_criterion_active() {
        let gw1 = make_gateway_with_family(GW_ID_1, 42);
        let gw2 = make_gateway_with_family(GW_ID_2, 99);
        assert!(gateways_are_independent(&gw1, &gw2, family_only()));
    }

    #[test]
    fn one_missing_family_independent_when_family_criterion_active() {
        let gw1 = make_gateway_with_family(GW_ID_1, 42);
        let gw2 = make_gateway(GW_ID_2);
        assert!(gateways_are_independent(&gw1, &gw2, family_only()));
    }

    #[test]
    fn full_criteria_passes_when_all_differ() {
        let gw1 = Gateway::builder()
            .identity(GW_ID_1.parse().unwrap())
            .location(Location {
                asn: Some(Asn {
                    asn: "AS100".to_string(),
                    name: "ISP A".to_string(),
                    route: "10.10.10.10/16".parse().unwrap(),
                    kind: AsnKind::Other,
                }),
                ..Default::default()
            })
            .family_data(Some(NodeFamily {
                id: 1,
                name: String::new(),
                description: String::new(),
                family_stake: 0,
                members: 0,
            }))
            .build();
        let gw2 = Gateway::builder()
            .identity(GW_ID_2.parse().unwrap())
            .location(Location {
                asn: Some(Asn {
                    asn: "AS200".to_string(),
                    name: "ISP B".to_string(),
                    route: "10.11.10.10/16".parse().unwrap(),
                    kind: AsnKind::Other,
                }),
                ..Default::default()
            })
            .family_data(Some(NodeFamily {
                id: 2,
                name: String::new(),
                description: String::new(),
                family_stake: 0,
                members: 0,
            }))
            .build();
        assert!(gateways_are_independent(
            &gw1,
            &gw2,
            GatewayIndependence::default()
        ));
    }

    #[test]
    fn full_criteria_fails_when_asn_matches_despite_rest_different() {
        let gw1 = Gateway::builder()
            .identity(GW_ID_1.parse().unwrap())
            .location(Location {
                asn: Some(Asn {
                    asn: "AS100".to_string(),
                    name: "ISP".to_string(),
                    route: "10.10.10.10/16".parse().unwrap(),
                    kind: AsnKind::Other,
                }),
                ..Default::default()
            })
            .family_data(Some(NodeFamily {
                id: 1,
                name: String::new(),
                description: String::new(),
                family_stake: 0,
                members: 0,
            }))
            .build();
        let gw2 = Gateway::builder()
            .identity(GW_ID_2.parse().unwrap())
            .location(Location {
                asn: Some(Asn {
                    asn: "AS100".to_string(),
                    name: "ISP".to_string(),
                    route: "10.11.10.10/16".parse().unwrap(),
                    kind: AsnKind::Other,
                }),
                ..Default::default()
            })
            .family_data(Some(NodeFamily {
                id: 2,
                name: String::new(),
                description: String::new(),
                family_stake: 0,
                members: 0,
            }))
            .build();
        assert!(!gateways_are_independent(
            &gw1,
            &gw2,
            GatewayIndependence::default()
        ));
    }

    #[test]
    fn full_criteria_fails_when_subnet_matches_despite_rest_different() {
        let gw1 = Gateway::builder()
            .identity(GW_ID_1.parse().unwrap())
            .location(Location {
                asn: Some(Asn {
                    asn: "AS100".to_string(),
                    name: "ISP1".to_string(),
                    route: "10.10.11.10/16".parse().unwrap(),
                    kind: AsnKind::Other,
                }),
                ..Default::default()
            })
            .family_data(Some(NodeFamily {
                id: 1,
                name: String::new(),
                description: String::new(),
                family_stake: 0,
                members: 0,
            }))
            .build();
        let gw2 = Gateway::builder()
            .identity(GW_ID_2.parse().unwrap())
            .location(Location {
                asn: Some(Asn {
                    asn: "AS101".to_string(),
                    name: "ISP2".to_string(),
                    route: "10.10.10.10/16".parse().unwrap(),
                    kind: AsnKind::Other,
                }),
                ..Default::default()
            })
            .family_data(Some(NodeFamily {
                id: 2,
                name: String::new(),
                description: String::new(),
                family_stake: 0,
                members: 0,
            }))
            .build();
        assert!(!gateways_are_independent(
            &gw1,
            &gw2,
            GatewayIndependence::default()
        ));
    }
}
