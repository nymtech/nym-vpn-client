use super::*;
use crate::BlacklistReason;

#[test]
fn test_matching_mixnet_score() {
    let gateway = Gateway::builder()
        .identity(
            NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                .unwrap(),
        )
        .performance(Performance {
            last_updated_utc: "".to_owned(),
            score: ScoreValue::Offline,
            mixnet_score: ScoreValue::High,
            load: ScoreValue::Medium,
            uptime_percentage_last_24_hours: 1f32,
        })
        .build();

    for gw_type in [GatewayType::MixnetEntry, GatewayType::MixnetExit] {
        assert!(gateway.matches_filter(Some(gw_type), &GatewayFilter::MinScore(ScoreValue::Low)));
    }

    let gateway = Gateway::builder()
        .identity(
            NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                .unwrap(),
        )
        .performance(Performance {
            last_updated_utc: "".to_owned(),
            score: ScoreValue::Offline,
            mixnet_score: ScoreValue::Low,
            load: ScoreValue::Medium,
            uptime_percentage_last_24_hours: 1f32,
        })
        .build();

    for gw_type in [GatewayType::MixnetEntry, GatewayType::MixnetExit] {
        assert!(!gateway.matches_filter(Some(gw_type), &GatewayFilter::MinScore(ScoreValue::High)));
    }
}

#[test]
fn test_matching_wg_score() {
    let gateway = Gateway::builder()
        .identity(
            NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                .unwrap(),
        )
        .performance(Performance {
            last_updated_utc: "".to_owned(),
            score: ScoreValue::High,
            mixnet_score: ScoreValue::Offline,
            load: ScoreValue::Medium,
            uptime_percentage_last_24_hours: 1f32,
        })
        .build();

    assert!(gateway.matches_filter(
        Some(GatewayType::Wg),
        &GatewayFilter::MinScore(ScoreValue::Low)
    ));

    let gateway = Gateway::builder()
        .identity(
            NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                .unwrap(),
        )
        .performance(Performance {
            last_updated_utc: "".to_owned(),
            score: ScoreValue::Low,
            mixnet_score: ScoreValue::Offline,
            load: ScoreValue::Medium,
            uptime_percentage_last_24_hours: 1f32,
        })
        .build();

    assert!(!gateway.matches_filter(
        Some(GatewayType::Wg),
        &GatewayFilter::MinScore(ScoreValue::High)
    ));
}

#[test]
fn test_matching_exit_node() {
    let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .ipr_address(IpPacketRouterAddress::try_from_base58_string(
                "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
            ).unwrap())
            .build();

    assert!(gateway.matches_filter(None, &GatewayFilter::Exit));

    let gateway = Gateway::builder()
        .identity(
            NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                .unwrap(),
        )
        .build();
    assert!(!gateway.matches_filter(None, &GatewayFilter::Exit));
}

#[test]
fn test_matching_residential() {
    let gateway = Gateway::builder()
        .identity(
            NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                .unwrap(),
        )
        .location(Location {
            asn: Some(Asn {
                kind: AsnKind::Residential,
                asn: "".to_owned(),
                name: "".to_owned(),
                route: "10.10.10.10/16".parse().unwrap(),
            }),
            ..Default::default()
        })
        .build();

    assert!(gateway.matches_filter(None, &GatewayFilter::Residential));

    let gateway = Gateway::builder()
        .identity(
            NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                .unwrap(),
        )
        .location(Location {
            asn: Some(Asn {
                kind: AsnKind::Other,
                asn: "".to_owned(),
                name: "".to_owned(),
                route: "10.10.10.10/16".parse().unwrap(),
            }),
            ..Default::default()
        })
        .build();

    assert!(!gateway.matches_filter(None, &GatewayFilter::Residential));
}

#[test]
fn test_matching_quic_enabled() {
    let gateway = Gateway::builder()
        .identity(
            NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                .unwrap(),
        )
        .bridge_params(Some(BridgeInformation {
            version: String::from("1"),
            transports: vec![BridgeParameters::QuicPlain(
                nym_vpn_api_client::response::QuicClientOptions {
                    addresses: vec!["1.2.3.4:5".parse().unwrap()],
                    host: Some(String::from("test.host")),
                    id_pubkey: String::from("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"),
                },
            )],
        }))
        .build();

    assert!(gateway.matches_filter(None, &GatewayFilter::QuicEnabled));

    let gateway = Gateway::builder()
        .identity(
            NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                .unwrap(),
        )
        .bridge_params(Some(BridgeInformation {
            version: String::from("1"),
            transports: vec![],
        }))
        .build();

    assert!(!gateway.matches_filter(None, &GatewayFilter::QuicEnabled));
}

#[test]
fn test_matching_vpn_nodes() {
    let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .authenticator_address(
                AuthAddress::try_from_base58_string(
                    "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
                ).unwrap()
            )
            .build();

    assert!(gateway.matches_filter(None, &GatewayFilter::Vpn));

    let gateway = Gateway::builder()
        .identity(
            NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                .unwrap(),
        )
        .build();

    assert!(!gateway.matches_filter(None, &GatewayFilter::Vpn));
}

#[test]
fn test_matching_country() {
    let gateway = Gateway::builder()
        .identity(
            NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                .unwrap(),
        )
        .location(Location {
            two_letter_iso_country_code: "CA".to_owned(),
            ..Default::default()
        })
        .build();

    assert!(gateway.matches_filter(None, &GatewayFilter::Country("CA".to_owned())));
    assert!(!gateway.matches_filter(None, &GatewayFilter::Country("US".to_owned())));
}

#[test]
fn test_matching_region() {
    let gateway = Gateway::builder()
        .identity(
            NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                .unwrap(),
        )
        .location(Location {
            two_letter_iso_country_code: "US".to_owned(),
            region: "CA".to_owned(),
            ..Default::default()
        })
        .build();

    assert!(gateway.matches_filter(None, &GatewayFilter::Region("CA".to_owned())));
    assert!(!gateway.matches_filter(None, &GatewayFilter::Region("FL".to_owned())));
}

#[test]
fn test_gateway_random_country() {
    let gateway_list = sample_gateway_list(GatewayType::MixnetEntry);

    assert!(
        gateway_list
            .choose_random(&GatewayFilters::from(&[GatewayFilter::Country(
                "US".into()
            )]))
            .unwrap()
            .is_in_country("US")
    );

    assert!(
        gateway_list
            .choose_random(&GatewayFilters::from(&[GatewayFilter::Country(
                "DE".into()
            )]))
            .unwrap()
            .is_in_country("DE")
    );

    assert!(
        gateway_list
            .choose_random(&GatewayFilters::from(&[GatewayFilter::Country(
                "BE".into()
            )]))
            .is_none()
    );
}

#[test]
fn test_gateway_random_region() {
    let gateway_list = sample_gateway_list(GatewayType::MixnetExit);

    assert!(
        gateway_list
            .choose_random(&GatewayFilters::from(&[
                GatewayFilter::Country("US".into()),
                GatewayFilter::Region("CA".into())
            ]))
            .unwrap()
            .is_in_region("CA")
    );

    assert!(
        gateway_list
            .choose_random(&GatewayFilters::from(&[
                GatewayFilter::Country("GB".into()),
                GatewayFilter::Region("Hampshire".into())
            ]))
            .unwrap()
            .is_in_region("Hampshire")
    );

    assert!(
        gateway_list
            .choose_random(&GatewayFilters::from(&[
                GatewayFilter::Country("DE".into()),
                GatewayFilter::Region("XZ".into())
            ]))
            .is_none()
    );
}

#[test]
fn test_gateway_non_blacklisted() {
    let gateway_list = sample_gateway_list(GatewayType::MixnetExit);

    let blacklisted = gateway_list.gateways[3].identity;
    let blacklisted_gateways = BlacklistedGateways::new();
    blacklisted_gateways
        .add(blacklisted, BlacklistReason::ConnectionFailed)
        .unwrap();

    for _ in 0..64 {
        let chosen = gateway_list
            .choose_random(&GatewayFilters::from(&[GatewayFilter::NotBlacklisted(
                blacklisted_gateways.clone(),
            )]))
            .unwrap();
        assert_ne!(chosen.identity, blacklisted);
    }
}

#[test]
fn test_pinned_entry_gateway_respects_blacklist() {
    let gateway_list = sample_gateway_list(GatewayType::Wg);
    let pinned = gateway_list.gateways[0].identity;

    let blacklisted_gateways = BlacklistedGateways::new();
    blacklisted_gateways
        .add(pinned, BlacklistReason::ConnectionFailed)
        .unwrap();
    let filters = GatewayFilters::from(&[GatewayFilter::NotBlacklisted(blacklisted_gateways)]);

    let entry_point = EntryPoint::Gateway {
        identity: pinned.into(),
    };
    let err = gateway_list
        .find_best_entry_point_gateway(&entry_point, &filters)
        .unwrap_err();
    assert!(
        matches!(err, Error::GatewayFilteredOut { .. }),
        "expected GatewayFilteredOut, got {err:?}"
    );
}

#[test]
fn test_pinned_exit_gateway_respects_blacklist() {
    let gateway_list = sample_gateway_list(GatewayType::MixnetExit);
    let pinned = gateway_list.gateways[0].identity;

    let blacklisted_gateways = BlacklistedGateways::new();
    blacklisted_gateways
        .add(pinned, BlacklistReason::ConnectionFailed)
        .unwrap();
    let filters = GatewayFilters::from(&[GatewayFilter::NotBlacklisted(blacklisted_gateways)]);

    let exit_point = ExitPoint::Gateway {
        identity: pinned.into(),
    };
    let err = gateway_list
        .find_best_exit_point_gateway(&exit_point, &filters)
        .unwrap_err();
    assert!(
        matches!(err, Error::GatewayFilteredOut { .. }),
        "expected GatewayFilteredOut, got {err:?}"
    );
}

#[test]
fn test_pinned_gateway_not_in_directory_is_no_matching_gateway() {
    let gateway_list = sample_gateway_list(GatewayType::Wg);
    let unknown =
        NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42").unwrap();

    let entry_point = EntryPoint::Gateway {
        identity: unknown.into(),
    };
    let err = gateway_list
        .find_best_entry_point_gateway(&entry_point, &GatewayFilters::default())
        .unwrap_err();
    assert!(
        matches!(err, Error::NoMatchingGateway { .. }),
        "expected NoMatchingGateway, got {err:?}"
    );
}

#[test]
fn test_low_performance_fallback_for_country_selection() {
    // Previously High -> Medium before failing
    // Now tries High -> Medium -> Low which allows connection to more gateways
    let entry_point = EntryPoint::Country {
        two_letter_iso_country_code: "VN".to_string(),
    };

    let gateways = GatewayList::new(
        Some(GatewayType::Wg),
        vec![create_test_gateway(
            "DoezvC92kAVDhFpBbsRj52rErhikj2vtPi1Lup2EhbZ4",
            "VN",
            ScoreValue::Low,
        )],
    );

    // Without Low fallback, this would fail
    let optional_filters = GatewayFilters::from(&[GatewayFilter::MinScore(ScoreValue::Low)]);
    let result =
        gateways.find_entry_gateway(&entry_point, &GatewayFilters::default(), &optional_filters);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().performance.as_ref().unwrap().score,
        ScoreValue::Low
    );
}

#[test]
fn test_socks5_score_from_mixnet_score() {
    for score in &[
        ScoreValue::High,
        ScoreValue::Medium,
        ScoreValue::Low,
        ScoreValue::Offline,
    ] {
        let nym_gw = create_response_nym_gateway(
            "HiVGQq2riqPFoPyYRYCZq3zFmFk15gnJzH4s9mHEbgKH",
            match score {
                ScoreValue::High => nym_vpn_api_client::response::ScoreValue::High,
                ScoreValue::Medium => nym_vpn_api_client::response::ScoreValue::Medium,
                ScoreValue::Low => nym_vpn_api_client::response::ScoreValue::Low,
                ScoreValue::Offline => nym_vpn_api_client::response::ScoreValue::Offline,
            },
        );
        let gw = Gateway::try_from(nym_gw).unwrap();
        assert_eq!(
            gw.last_probe
                .as_ref()
                .unwrap()
                .outcome
                .socks5
                .as_ref()
                .unwrap()
                .score
                .unwrap(),
            *score,
            "Mixnet score should match for score {:?}",
            score
        );
    }
}

// Create a list of Gateways with different properties set for testing
fn sample_gateway_list(gw_type: GatewayType) -> GatewayList {
    let asn = Asn {
        asn: "AS12345".to_string(),
        name: "Test ASN".to_string(),
        route: "10.10.10.10/16".parse().unwrap(),
        kind: AsnKind::Residential,
    };
    let addr = "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e";
    let ipr = IpPacketRouterAddress::try_from_base58_string(addr).unwrap();
    let aa = AuthAddress::try_from_base58_string(addr).unwrap();
    let variables = [
        (
            // Gateway 1
            "HiVGQq2riqPFoPyYRYCZq3zFmFk15gnJzH4s9mHEbgKH",
            "US",
            "CA",
            None,
            Some(ipr),
            Some(aa),
        ),
        (
            // Gateway 2
            "B4r2xMJYc4VgoEhPmccmNSawQWdYP9zGp9DJqjcz6PoX",
            "US",
            "NY",
            Some(asn.clone()),
            None,
            None,
        ),
        (
            // Gateway 3
            "6tGNU195QKNMaTxkvm917d3NNGLkpTp8mTfxqLzATbtB",
            "DE",
            "BE",
            None,
            None,
            Some(aa),
        ),
        (
            // Gateway 4
            "F618gw6jZaLR1VdMTeaH11MhHQJY5rdpYEDLrMKEHcjk",
            "FR",
            "Aquitaine",
            Some(asn.clone()),
            None,
            Some(aa),
        ),
        (
            // Gateway 5
            "3UBiq22tkNSRhyRNjL5mnw5Yk4z6FvgvjizT4ukeEaeB",
            "US",
            "TX",
            Some(asn.clone()),
            Some(ipr),
            None,
        ),
        (
            // Gateway 6
            "2djmrzZ62M8jpzpYb7MMq6QjP15CkbnKHf3ZV3kSCXUE",
            "GB",
            "Hampshire",
            None,
            None,
            None,
        ),
    ];

    let mut instance = 0;
    let gateways: Vec<Gateway> = variables
        .into_iter()
        .map(|(identity, country, region, asn, ipr, aa)| {
            instance += 1;
            Gateway {
                identity: NodeIdentity::from_base58_string(identity).unwrap(),
                name: format!("Gateway {instance}"),
                description: None,
                location: Some(Location {
                    two_letter_iso_country_code: country.to_string(),
                    region: region.to_string(),
                    asn,
                    ..Default::default()
                }),
                ipr_address: ipr,
                authenticator_address: aa,
                nr_address: None,
                bridge_params: None,
                last_probe: None,
                ips: Vec::new(),
                host: None,
                clients_ws_port: None,
                clients_wss_port: None,
                mixnet_performance: Some(Percent::from_percentage_value(75).unwrap()),
                performance: Some(Performance {
                    last_updated_utc: "2024-01-01T00:00:00Z".to_string(),
                    score: ScoreValue::High,
                    mixnet_score: ScoreValue::High,
                    load: ScoreValue::Low,
                    uptime_percentage_last_24_hours: 0.75,
                }),
                version: None,
                lewes_protocol_details: None,
                staking_data: None,
                family_data: None,
            }
        })
        .collect();
    GatewayList::new(Some(gw_type), gateways)
}

fn create_test_gateway(identity: &str, country: &str, score: ScoreValue) -> Gateway {
    Gateway {
        identity: NodeIdentity::from_base58_string(identity).unwrap(),
        name: format!("Test Gateway {}", country),
        description: None,
        location: Some(Location {
            two_letter_iso_country_code: country.to_string(),
            ..Default::default()
        }),
        ipr_address: None,
        authenticator_address: None,
        nr_address: None,
        bridge_params: None,
        last_probe: None,
        ips: Vec::new(),
        host: None,
        clients_ws_port: None,
        clients_wss_port: None,
        mixnet_performance: None,
        performance: Some(Performance {
            last_updated_utc: "2025-10-22T00:00:00Z".to_string(),
            score,
            mixnet_score: ScoreValue::High,
            load: ScoreValue::Low,
            uptime_percentage_last_24_hours: 0.99,
        }),
        version: None,
        lewes_protocol_details: None,
        staking_data: None,
        family_data: None,
    }
}

fn create_response_nym_gateway(
    identity: &str,
    mixnet_score: nym_vpn_api_client::response::ScoreValue,
) -> nym_vpn_api_client::response::NymDirectoryGateway {
    nym_vpn_api_client::response::NymDirectoryGateway {
        identity_key: identity.into(),
        name: "test-gateway".into(),
        description: None,
        ip_packet_router: None,
        authenticator: None,
        location: nym_vpn_api_client::response::Location {
            two_letter_iso_country_code: "US".into(),
            latitude: 41.8781,
            longitude: -87.6298,
            city: "Chicago".into(),
            region: "IL".into(),
            asn: None,
        },
        last_probe: Some(nym_vpn_api_client::response::Probe {
            last_updated_utc: "2024-01-01T00:00:00Z".to_string(),
            outcome: nym_vpn_api_client::response::ProbeOutcome {
                as_entry: nym_vpn_api_client::response::Entry {
                    can_connect: true,
                    can_route: true,
                },
                as_exit: Some(nym_vpn_api_client::response::Exit {
                    can_connect: false,
                    can_route_ip_v4: false,
                    can_route_ip_external_v4: false,
                    can_route_ip_v6: false,
                    can_route_ip_external_v6: false,
                    socks5: None,
                }),
                wg: None,
                socks5: None,
                lp: None,
            },
        }),
        ip_addresses: vec![],
        mix_port: 0,
        role: nym_vpn_api_client::response::Role::ExitGateway,
        entry: nym_vpn_api_client::response::EntryInformation {
            hostname: Some("tulips".into()),
            ws_port: 9000,
            wss_port: Some(9001),
        },
        bridges: None,
        performance: Percent::zero(),
        performance_v2: Some(nym_vpn_api_client::response::DVpnGatewayPerformance {
            last_updated_utc: "2024-01-01T00:00:00Z".to_string(),
            score: nym_vpn_api_client::response::ScoreValue::Low,
            mixnet_score,
            load: nym_vpn_api_client::response::ScoreValue::Low,
            uptime_percentage_last_24_hours: 0.75,
        }),
        build_information: None,
        lewes_protocol_details: None,
        staking_data: None,
        family_data: None,
    }
}

fn gw_at_latitude(id: &str, latitude: f64, score: ScoreValue) -> Gateway {
    Gateway::builder()
        .identity(NodeIdentity::from_base58_string(id).unwrap())
        .location(Location {
            two_letter_iso_country_code: "XX".to_owned(),
            latitude,
            longitude: 0.0,
            city: String::new(),
            region: String::new(),
            asn: None,
        })
        .performance(Performance {
            last_updated_utc: String::new(),
            score,
            mixnet_score: score,
            load: ScoreValue::Low,
            uptime_percentage_last_24_hours: 1f32,
        })
        .build()
}

fn latitude_distance_to_equator(gw1: &Gateway, gw2: &Gateway) -> std::cmp::Ordering {
    let lat = |gw: &Gateway| {
        gw.location
            .as_ref()
            .map(|l| l.latitude.abs())
            .unwrap_or(f64::MAX)
    };
    lat(gw1).total_cmp(&lat(gw2))
}

const CLOSEST_IDS: [&str; 6] = [
    "24h2yanCFU5iy7xNQmW6RowFa6EzmAYQdM1bs8Y1X6iH",
    "26ZmTxTVBKHZg8MTKwypHkXZVJhDC7QHuv3BdsyRyTuk",
    "27GwHdmXLULVieyXmxZ6v9DHzRJtTEjfode1dzbptEAK",
    "28tXg9mEW4mifgU1TdetVVAN5PvmhtLpHzFRMfJBT6ND",
    "29U3LythwEaqigL5YajXALw1c7DE7YNcRW7Vn7KcYMQL",
    "2aZj5UjC4N3SMjfJNjFiaHPqg1sgKDBYwaLozxGePQxW",
];

#[test]
fn closest_pick_is_spread_over_the_nearest_candidates_only() {
    // Gateways sit at increasing distance from the reference (the equator).
    let gateways = CLOSEST_IDS
        .iter()
        .enumerate()
        .map(|(i, id)| gw_at_latitude(id, i as f64 * 10.0, ScoreValue::High))
        .collect();
    let list = GatewayList::new(Some(GatewayType::Wg), gateways);

    let mut picked = std::collections::HashSet::new();
    for _ in 0..300 {
        let gw = list
            .find_best_ordering_criteria_gateway(
                latitude_distance_to_equator,
                &GatewayFilters::default(),
            )
            .unwrap();
        picked.insert(gw.identity().to_base58_string());
    }

    let nearest: std::collections::HashSet<String> = CLOSEST_IDS[..CLOSEST_GATEWAY_CANDIDATES]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert!(
        picked.is_subset(&nearest),
        "only the {CLOSEST_GATEWAY_CANDIDATES} nearest gateways may be picked, got {picked:?}"
    );
    assert!(
        picked.len() > 1,
        "the pick must be spread across candidates so one dead node does not hit every user"
    );
}

#[test]
fn closest_pick_still_prefers_the_better_score_tier_over_distance() {
    let mut gateways: Vec<Gateway> = CLOSEST_IDS[..3]
        .iter()
        .enumerate()
        .map(|(i, id)| gw_at_latitude(id, i as f64, ScoreValue::Medium))
        .collect();
    gateways.push(gw_at_latitude(CLOSEST_IDS[5], 80.0, ScoreValue::High));
    let list = GatewayList::new(Some(GatewayType::Wg), gateways);

    for _ in 0..50 {
        let gw = list
            .find_best_ordering_criteria_gateway(
                latitude_distance_to_equator,
                &GatewayFilters::default(),
            )
            .unwrap();
        assert_eq!(gw.identity().to_base58_string(), CLOSEST_IDS[5]);
    }
}
