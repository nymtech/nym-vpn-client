// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, net::IpAddr};

use ipnet::{Ipv4Net, Ipv6Net};

use super::{
    CountryGeoData, CountryIpSet, GeoIpDatabase, Ipv4RangeSet, Ipv6RangeSet, RoutingDecision,
    decompress, embedded_geoip_gz, parse_ipv4_cidrs, parse_ipv6_cidrs,
};
use crate::routing::decide_route;

fn make_v4_set(cidrs: &[&str]) -> Ipv4RangeSet {
    let nets = cidrs.iter().map(|s| s.parse::<Ipv4Net>().unwrap());
    Ipv4RangeSet::from_cidrs(nets)
}

fn make_v6_set(cidrs: &[&str]) -> Ipv6RangeSet {
    let nets = cidrs.iter().map(|s| s.parse::<Ipv6Net>().unwrap());
    Ipv6RangeSet::from_cidrs(nets)
}

/// Decompress and deserialise the embedded JSON for `country_code`.
async fn embedded_country_data(country_code: &str) -> CountryGeoData {
    let gz = embedded_geoip_gz(country_code)
        .unwrap_or_else(|| panic!("No embedded GeoIP data for {country_code}"));
    let json = decompress(gz)
        .await
        .unwrap_or_else(|e| panic!("Decompression failed for {country_code}: {e}"));
    serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("JSON parse failed for {country_code}: {e}"))
}

#[test]
fn ipv4_contains_basic() {
    let set = make_v4_set(&["192.168.1.0/24", "10.0.0.0/8"]);
    assert!(set.contains("192.168.1.1".parse().unwrap()));
    assert!(set.contains("192.168.1.255".parse().unwrap()));
    assert!(set.contains("10.255.255.255".parse().unwrap()));
    assert!(!set.contains("192.168.2.0".parse().unwrap()));
    assert!(!set.contains("172.16.0.1".parse().unwrap()));
}

#[test]
fn ipv4_merge_adjacent() {
    // Two adjacent /25s should merge into a single range.
    let set = make_v4_set(&["192.168.1.0/25", "192.168.1.128/25"]);
    assert_eq!(set.starts.len(), 1, "Adjacent ranges must be merged");
    assert!(set.contains("192.168.1.0".parse().unwrap()));
    assert!(set.contains("192.168.1.255".parse().unwrap()));
}

#[test]
fn ipv4_boundary() {
    let set = make_v4_set(&["1.0.1.0/24"]);
    assert!(!set.contains("1.0.0.255".parse().unwrap()));
    assert!(set.contains("1.0.1.0".parse().unwrap()));
    assert!(set.contains("1.0.1.255".parse().unwrap()));
    assert!(!set.contains("1.0.2.0".parse().unwrap()));
}

#[test]
fn ipv4_gap_between_ranges() {
    // Ensure an IP in the gap between two non-adjacent ranges is not matched.
    let set = make_v4_set(&["1.0.1.0/24", "1.0.3.0/24"]);
    assert!(set.contains("1.0.1.1".parse().unwrap()));
    assert!(!set.contains("1.0.2.1".parse().unwrap())); // gap
    assert!(set.contains("1.0.3.1".parse().unwrap()));
}

#[test]
fn ipv6_contains_basic() {
    let set = make_v6_set(&["2001:250::/35"]);
    assert!(set.contains("2001:250::1".parse().unwrap()));
    assert!(!set.contains("2001:300::1".parse().unwrap()));
}

#[tokio::test]
async fn embedded_cn_ipv4_parses() {
    let geo = embedded_country_data("CN").await;
    let set = parse_ipv4_cidrs(&geo.ipv4).expect("CN IPv4 CIDRs should build");
    assert!(
        set.len() > 1000,
        "Expected >1000 CN IPv4 ranges after merge, got {}",
        set.len()
    );
    // Known Chinese IP block (Tencent cloud, APNIC CN allocation).
    assert!(set.contains("1.0.1.1".parse().unwrap()));
    // Known non-Chinese IP.
    assert!(!set.contains("8.8.8.8".parse().unwrap()));
}

#[tokio::test]
async fn embedded_cn_ipv6_parses() {
    let geo = embedded_country_data("CN").await;
    let set = parse_ipv6_cidrs(&geo.ipv6).expect("CN IPv6 CIDRs should build");
    assert!(
        set.len() > 100,
        "Expected >100 CN IPv6 ranges after merge, got {}",
        set.len()
    );
}

#[test]
fn decide_route_no_tunnel() {
    let db = GeoIpDatabase {
        excluded_countries: HashMap::new(),
    };
    assert_eq!(
        decide_route("1.0.1.1".parse().unwrap(), None, &db),
        RoutingDecision::DefaultInterface,
    );
}

#[test]
fn decide_route_excluded_country() {
    let mut countries = HashMap::new();
    let set = CountryIpSet {
        v4: make_v4_set(&["1.0.1.0/24"]),
        v6: Ipv6RangeSet::default(),
    };
    countries.insert("CN".to_string(), set);
    let db = GeoIpDatabase {
        excluded_countries: countries,
    };

    let tunnel: IpAddr = "10.0.0.1".parse().unwrap();

    // Chinese IP → bypass tunnel.
    assert_eq!(
        decide_route("1.0.1.1".parse().unwrap(), Some(tunnel), &db),
        RoutingDecision::DefaultInterface,
    );

    // Non-Chinese IP → use tunnel.
    assert_eq!(
        decide_route("8.8.8.8".parse().unwrap(), Some(tunnel), &db),
        RoutingDecision::VpnTunnelInterface,
    );
}
