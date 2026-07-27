// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
};

use super::{
    RoutingDecision, decide_route_for_addrs, decompress_gz,
    domain::DomainSet,
    ip::{
        CountryGeoData, CountryIpSet, GeoIpDatabase, embedded_geoip_gz, parse_ipv4_cidrs,
        parse_ipv6_cidrs,
    },
};

use ipnet::{Ipv4Net, Ipv6Net};
use iprange::IpRange;
use nym_socks5_proxy_ipc::InterfaceAddresses;

/// Helper: wrap a single IP address as a one-element SocketAddr slice for decide_route_for_addrs.
fn sa(ip: &str) -> Vec<SocketAddr> {
    let ip: IpAddr = ip.parse().unwrap();
    vec![SocketAddr::new(ip, 80)]
}

fn make_v4_set(cidrs: &[&str]) -> IpRange<Ipv4Net> {
    let mut range = IpRange::new();
    for s in cidrs {
        range.add(s.parse::<Ipv4Net>().unwrap());
    }
    range
}

fn make_v6_set(cidrs: &[&str]) -> IpRange<Ipv6Net> {
    let mut range = IpRange::new();
    for s in cidrs {
        range.add(s.parse::<Ipv6Net>().unwrap());
    }
    range
}

/// Decompress and deserialise the embedded JSON for `country_code`.
async fn embedded_country_data(country_code: &str) -> CountryGeoData {
    let gz = embedded_geoip_gz(country_code)
        .unwrap_or_else(|| panic!("No embedded GeoIP data for {country_code}"));
    let json = decompress_gz(gz)
        .await
        .unwrap_or_else(|e| panic!("Decompression failed for {country_code}: {e}"));
    serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("JSON parse failed for {country_code}: {e}"))
}

#[test]
fn ipv4_contains_basic() {
    let set = make_v4_set(&["192.168.1.0/24", "10.0.0.0/8"]);
    assert!(set.contains(&"192.168.1.1/32".parse::<Ipv4Net>().unwrap()));
    assert!(set.contains(&"192.168.1.255/32".parse::<Ipv4Net>().unwrap()));
    assert!(set.contains(&"10.255.255.255/32".parse::<Ipv4Net>().unwrap()));
    assert!(!set.contains(&"192.168.2.0/32".parse::<Ipv4Net>().unwrap()));
    assert!(!set.contains(&"172.16.0.1/32".parse::<Ipv4Net>().unwrap()));
}

#[test]
fn ipv4_merge_adjacent() {
    // Two adjacent /25s should cover every host in the combined /24.
    let set = make_v4_set(&["192.168.1.0/25", "192.168.1.128/25"]);
    assert!(set.contains(&"192.168.1.0/32".parse::<Ipv4Net>().unwrap()));
    assert!(set.contains(&"192.168.1.127/32".parse::<Ipv4Net>().unwrap()));
    assert!(set.contains(&"192.168.1.128/32".parse::<Ipv4Net>().unwrap()));
    assert!(set.contains(&"192.168.1.255/32".parse::<Ipv4Net>().unwrap()));
}

#[test]
fn ipv4_boundary() {
    let set = make_v4_set(&["1.0.1.0/24"]);
    assert!(!set.contains(&"1.0.0.255/32".parse::<Ipv4Net>().unwrap()));
    assert!(set.contains(&"1.0.1.0/32".parse::<Ipv4Net>().unwrap()));
    assert!(set.contains(&"1.0.1.255/32".parse::<Ipv4Net>().unwrap()));
    assert!(!set.contains(&"1.0.2.0/32".parse::<Ipv4Net>().unwrap()));
}

#[test]
fn ipv4_gap_between_ranges() {
    let set = make_v4_set(&["1.0.1.0/24", "1.0.3.0/24"]);
    assert!(set.contains(&"1.0.1.1/32".parse::<Ipv4Net>().unwrap()));
    assert!(!set.contains(&"1.0.2.1/32".parse::<Ipv4Net>().unwrap())); // gap
    assert!(set.contains(&"1.0.3.1/32".parse::<Ipv4Net>().unwrap()));
}

#[test]
fn ipv6_contains_basic() {
    let set = make_v6_set(&["2001:250::/35"]);
    assert!(set.contains(&"2001:250::/128".parse::<Ipv6Net>().unwrap()));
    assert!(!set.contains(&"2001:300::/128".parse::<Ipv6Net>().unwrap()));
}

#[tokio::test]
async fn embedded_cn_ipv4_parses() {
    let geo = embedded_country_data("CN").await;
    let set = parse_ipv4_cidrs(&geo.ipv4).expect("CN IPv4 CIDRs should build");
    assert!(
        set.iter().count() > 1000,
        "Expected >1000 CN IPv4 ranges, got {}",
        set.iter().count()
    );
    // Known Chinese IP block (Tencent cloud, APNIC CN allocation).
    assert!(set.contains(&"1.0.1.1/32".parse::<Ipv4Net>().unwrap()));
    // Known non-Chinese IP.
    assert!(!set.contains(&"8.8.8.8/32".parse::<Ipv4Net>().unwrap()));
}

#[tokio::test]
async fn embedded_cn_ipv6_parses() {
    let geo = embedded_country_data("CN").await;
    let set = parse_ipv6_cidrs(&geo.ipv6).expect("CN IPv6 CIDRs should build");
    assert!(
        set.iter().count() > 100,
        "Expected >100 CN IPv6 ranges, got {}",
        set.iter().count()
    );
}

#[tokio::test]
async fn embedded_ru_ipv4_parses() {
    let geo = embedded_country_data("RU").await;
    let set = parse_ipv4_cidrs(&geo.ipv4).expect("RU IPv4 CIDRs should build");
    assert!(
        set.iter().count() > 1000,
        "Expected >1000 RU IPv4 ranges, got {}",
        set.iter().count()
    );
    // Known Russian IP block (Yandex, RIPE RU allocation).
    assert!(set.contains(&"5.255.255.77/32".parse::<Ipv4Net>().unwrap()));
    // Known non-Russian IP.
    assert!(!set.contains(&"8.8.8.8/32".parse::<Ipv4Net>().unwrap()));
}

#[tokio::test]
async fn embedded_ru_ipv6_parses() {
    let geo = embedded_country_data("RU").await;
    let set = parse_ipv6_cidrs(&geo.ipv6).expect("RU IPv6 CIDRs should build");
    assert!(
        set.iter().count() > 100,
        "Expected >100 RU IPv6 ranges, got {}",
        set.iter().count()
    );
}

#[test]
fn ipv6_boundary() {
    let set = make_v6_set(&["2001:db8::/32"]);
    assert!(
        !set.contains(
            &"2001:db7:ffff:ffff:ffff:ffff:ffff:ffff/128"
                .parse::<Ipv6Net>()
                .unwrap()
        )
    );
    assert!(set.contains(&"2001:db8::/128".parse::<Ipv6Net>().unwrap()));
    assert!(set.contains(&"2001:db8::1/128".parse::<Ipv6Net>().unwrap()));
    assert!(
        set.contains(
            &"2001:db8:ffff:ffff:ffff:ffff:ffff:ffff/128"
                .parse::<Ipv6Net>()
                .unwrap()
        )
    );
    assert!(!set.contains(&"2001:db9::/128".parse::<Ipv6Net>().unwrap()));
}

#[test]
fn ipv6_merge_adjacent() {
    // Two adjacent /33s should cover every host in the combined /32.
    let set = make_v6_set(&["2001:db8::/33", "2001:db8:8000::/33"]);
    assert!(set.contains(&"2001:db8::1/128".parse::<Ipv6Net>().unwrap()));
    assert!(
        set.contains(
            &"2001:db8:7fff:ffff:ffff:ffff:ffff:ffff/128"
                .parse::<Ipv6Net>()
                .unwrap()
        )
    );
    assert!(set.contains(&"2001:db8:8000::1/128".parse::<Ipv6Net>().unwrap()));
    assert!(
        set.contains(
            &"2001:db8:ffff:ffff:ffff:ffff:ffff:ffff/128"
                .parse::<Ipv6Net>()
                .unwrap()
        )
    );
}

#[test]
fn ipv6_gap_between_ranges() {
    let set = make_v6_set(&["2001:db8:1::/48", "2001:db8:3::/48"]);
    assert!(set.contains(&"2001:db8:1::1/128".parse::<Ipv6Net>().unwrap()));
    assert!(!set.contains(&"2001:db8:2::1/128".parse::<Ipv6Net>().unwrap())); // gap
    assert!(set.contains(&"2001:db8:3::1/128".parse::<Ipv6Net>().unwrap()));
}

#[test]
fn ipv6_multiple_prefixes() {
    let set = make_v6_set(&["2001:250::/35", "240e::/16", "2400::/12"]);
    assert!(set.contains(&"2001:250::1/128".parse::<Ipv6Net>().unwrap()));
    assert!(set.contains(&"240e::1/128".parse::<Ipv6Net>().unwrap()));
    assert!(set.contains(&"2400::1/128".parse::<Ipv6Net>().unwrap()));
    assert!(!set.contains(&"2002::1/128".parse::<Ipv6Net>().unwrap()));
    assert!(!set.contains(&"::1/128".parse::<Ipv6Net>().unwrap()));
}

#[test]
fn ipv6_empty_set() {
    let set = make_v6_set(&[]);
    assert!(!set.contains(&"2001:db8::1/128".parse::<Ipv6Net>().unwrap()));
    assert!(!set.contains(&"::1/128".parse::<Ipv6Net>().unwrap()));
}

#[test]
fn decide_route_no_tunnel() {
    // With no tunnel active, non-excluded traffic must be rejected rather than
    // leaked over the default interface (kill-switch behaviour).
    let db = GeoIpDatabase {
        excluded_countries: HashMap::new(),
    };
    assert_eq!(
        decide_route_for_addrs(&sa("1.0.1.1"), &InterfaceAddresses::default(), &db),
        RoutingDecision::Reject,
    );
}

#[test]
fn decide_route_no_tunnel_excluded_still_direct() {
    // Excluded destinations continue to route directly even when no tunnel is
    // active: they were always meant to bypass the VPN, so this leaks nothing new.
    let mut countries = HashMap::new();
    let set = CountryIpSet {
        v4: make_v4_set(&["1.0.1.0/24"]),
        v6: IpRange::new(),
    };
    countries.insert("CN".to_string(), set);
    let db = GeoIpDatabase {
        excluded_countries: countries,
    };
    assert_eq!(
        decide_route_for_addrs(&sa("1.0.1.1"), &InterfaceAddresses::default(), &db),
        RoutingDecision::DefaultInterface,
    );
}

#[test]
fn decide_route_excluded_country() {
    let mut countries = HashMap::new();
    let set = CountryIpSet {
        v4: make_v4_set(&["1.0.1.0/24"]),
        v6: IpRange::new(),
    };
    countries.insert("CN".to_string(), set);
    let db = GeoIpDatabase {
        excluded_countries: countries,
    };

    let tunnel_addrs = InterfaceAddresses {
        v4_addr: Some("10.0.0.1".parse().unwrap()),
        v6_addr: None,
    };

    // Chinese IP → bypass tunnel.
    assert_eq!(
        decide_route_for_addrs(&sa("1.0.1.1"), &tunnel_addrs, &db),
        RoutingDecision::DefaultInterface,
    );

    // Non-Chinese IP → use tunnel.
    assert_eq!(
        decide_route_for_addrs(&sa("8.8.8.8"), &tunnel_addrs, &db),
        RoutingDecision::VpnTunnelInterface,
    );
}

#[test]
fn decide_route_no_tunnel_ipv6() {
    // With no tunnel addresses at all, non-excluded traffic must be rejected.
    let db = GeoIpDatabase {
        excluded_countries: HashMap::new(),
    };
    assert_eq!(
        decide_route_for_addrs(&sa("2001:db8::1"), &InterfaceAddresses::default(), &db),
        RoutingDecision::Reject,
    );
}

#[test]
fn decide_route_ipv6_no_v6_tunnel() {
    // Tunnel has only an IPv4 address — non-excluded IPv6 destinations cannot be
    // carried by the tunnel and must be rejected, not leaked over the default
    // interface.
    let db = GeoIpDatabase {
        excluded_countries: HashMap::new(),
    };
    let tunnel_addrs = InterfaceAddresses {
        v4_addr: Some("10.0.0.1".parse().unwrap()),
        v6_addr: None,
    };
    assert_eq!(
        decide_route_for_addrs(&sa("2606:4700::1"), &tunnel_addrs, &db),
        RoutingDecision::Reject,
    );
    // IPv4 destinations can still use the tunnel.
    assert_eq!(
        decide_route_for_addrs(&sa("1.1.1.1"), &tunnel_addrs, &db),
        RoutingDecision::VpnTunnelInterface,
    );
}

#[test]
fn decide_route_excluded_country_ipv6() {
    // CN IPv6 prefix should route via default interface; non-CN IPv6 via tunnel.
    let mut countries = HashMap::new();
    let set = CountryIpSet {
        v4: IpRange::new(),
        v6: make_v6_set(&["2001:250::/35", "240e::/16"]),
    };
    countries.insert("CN".to_string(), set);
    let db = GeoIpDatabase {
        excluded_countries: countries,
    };

    let tunnel_addrs = InterfaceAddresses {
        v4_addr: Some("10.0.0.1".parse().unwrap()),
        v6_addr: Some("fc00::1".parse().unwrap()),
    };

    // Chinese IPv6 → bypass tunnel.
    assert_eq!(
        decide_route_for_addrs(&sa("2001:250::1"), &tunnel_addrs, &db),
        RoutingDecision::DefaultInterface,
    );
    assert_eq!(
        decide_route_for_addrs(&sa("240e::1"), &tunnel_addrs, &db),
        RoutingDecision::DefaultInterface,
    );

    // Non-Chinese IPv6 → use tunnel.
    assert_eq!(
        decide_route_for_addrs(&sa("2606:4700::1"), &tunnel_addrs, &db),
        RoutingDecision::VpnTunnelInterface,
    );
    assert_eq!(
        decide_route_for_addrs(&sa("2001:4860:4860::8888"), &tunnel_addrs, &db),
        RoutingDecision::VpnTunnelInterface,
    );
}

#[test]
fn decide_route_dual_stack_tunnel() {
    // With both v4 and v6 tunnel addresses, each family is independently routed.
    let mut countries = HashMap::new();
    let set = CountryIpSet {
        v4: make_v4_set(&["1.0.1.0/24"]),
        v6: make_v6_set(&["2001:250::/35"]),
    };
    countries.insert("CN".to_string(), set);
    let db = GeoIpDatabase {
        excluded_countries: countries,
    };

    let tunnel_addrs = InterfaceAddresses {
        v4_addr: Some("10.0.0.1".parse().unwrap()),
        v6_addr: Some("fc00::1".parse().unwrap()),
    };

    // CN IPv4 → default interface.
    assert_eq!(
        decide_route_for_addrs(&sa("1.0.1.1"), &tunnel_addrs, &db),
        RoutingDecision::DefaultInterface,
    );
    // CN IPv6 → default interface.
    assert_eq!(
        decide_route_for_addrs(&sa("2001:250::1"), &tunnel_addrs, &db),
        RoutingDecision::DefaultInterface,
    );
    // Non-CN IPv4 → tunnel.
    assert_eq!(
        decide_route_for_addrs(&sa("8.8.8.8"), &tunnel_addrs, &db),
        RoutingDecision::VpnTunnelInterface,
    );
    // Non-CN IPv6 → tunnel.
    assert_eq!(
        decide_route_for_addrs(&sa("2606:4700::1"), &tunnel_addrs, &db),
        RoutingDecision::VpnTunnelInterface,
    );
}

#[test]
fn decide_route_multiple_excluded_countries() {
    // Both CN and RU excluded at once — each country's ranges should be routed
    // independently, and traffic belonging to neither should still hit the tunnel.
    let mut countries = HashMap::new();
    countries.insert(
        "CN".to_string(),
        CountryIpSet {
            v4: make_v4_set(&["1.0.1.0/24"]),
            v6: IpRange::new(),
        },
    );
    countries.insert(
        "RU".to_string(),
        CountryIpSet {
            v4: make_v4_set(&["5.255.192.0/18"]),
            v6: IpRange::new(),
        },
    );
    let db = GeoIpDatabase {
        excluded_countries: countries,
    };

    let tunnel_addrs = InterfaceAddresses {
        v4_addr: Some("10.0.0.1".parse().unwrap()),
        v6_addr: None,
    };

    // Chinese IP → bypass tunnel.
    assert_eq!(
        decide_route_for_addrs(&sa("1.0.1.1"), &tunnel_addrs, &db),
        RoutingDecision::DefaultInterface,
    );
    // Russian IP → bypass tunnel.
    assert_eq!(
        decide_route_for_addrs(&sa("5.255.255.77"), &tunnel_addrs, &db),
        RoutingDecision::DefaultInterface,
    );
    // Neither CN nor RU → use tunnel.
    assert_eq!(
        decide_route_for_addrs(&sa("8.8.8.8"), &tunnel_addrs, &db),
        RoutingDecision::VpnTunnelInterface,
    );
}

#[tokio::test]
async fn embedded_cn_ipv6_known_addresses() {
    let geo = embedded_country_data("CN").await;
    let set = parse_ipv6_cidrs(&geo.ipv6).expect("CN IPv6 CIDRs should build");

    // 2001:250::/35 — CERNET (China Education and Research Network)
    assert!(
        set.contains(&"2001:250::1/128".parse::<Ipv6Net>().unwrap()),
        "CERNET prefix should be in CN IPv6 set"
    );
    // 240e::/16 — China Telecom
    assert!(
        set.contains(&"240e::1/128".parse::<Ipv6Net>().unwrap()),
        "China Telecom prefix should be in CN IPv6 set"
    );
    // 2400:3200::/32 — Alibaba Cloud
    assert!(
        set.contains(&"2400:3200::1/128".parse::<Ipv6Net>().unwrap()),
        "Alibaba Cloud prefix should be in CN IPv6 set"
    );
    // Non-Chinese allocations should not appear.
    assert!(
        !set.contains(&"2606:4700::1/128".parse::<Ipv6Net>().unwrap()),
        "Cloudflare address should not be in CN IPv6 set"
    );
    assert!(
        !set.contains(&"2001:4860:4860::8888/128".parse::<Ipv6Net>().unwrap()),
        "Google DNS should not be in CN IPv6 set"
    );
}

#[tokio::test]
async fn embedded_ru_ipv6_known_addresses() {
    let geo = embedded_country_data("RU").await;
    let set = parse_ipv6_cidrs(&geo.ipv6).expect("RU IPv6 CIDRs should build");

    // 2a02:6b8::/29 — Yandex
    assert!(
        set.contains(&"2a02:6b8::1/128".parse::<Ipv6Net>().unwrap()),
        "Yandex prefix should be in RU IPv6 set"
    );
    // 2001:640::/32 — RUNNet (Russian Institute for Public Networks)
    assert!(
        set.contains(&"2001:640::1/128".parse::<Ipv6Net>().unwrap()),
        "RUNNet prefix should be in RU IPv6 set"
    );
    // 2a00:1148::/29 — Mail.ru Group (VK)
    assert!(
        set.contains(&"2a00:1148::1/128".parse::<Ipv6Net>().unwrap()),
        "Mail.ru Group prefix should be in RU IPv6 set"
    );
    // Non-Russian allocations should not appear.
    assert!(
        !set.contains(&"2606:4700::1/128".parse::<Ipv6Net>().unwrap()),
        "Cloudflare address should not be in RU IPv6 set"
    );
    assert!(
        !set.contains(&"2001:4860:4860::8888/128".parse::<Ipv6Net>().unwrap()),
        "Google DNS should not be in RU IPv6 set"
    );
}

fn make_domain_set(domain: &[&str]) -> DomainSet {
    let text = domain.join("\n");
    DomainSet::from_text(&text).unwrap()
}

#[test]
fn domain_exact_match() {
    let set = make_domain_set(&["ipip.net", "baidu.com"]);
    assert!(set.is_excluded("ipip.net"));
    assert!(set.is_excluded("baidu.com"));
}

#[test]
fn domain_subdomain_match() {
    let set = make_domain_set(&["ipip.net"]);
    assert!(set.is_excluded("myip.ipip.net"));
    assert!(set.is_excluded("sub.myip.ipip.net"));
}

#[test]
fn domain_no_partial_match() {
    let set = make_domain_set(&["ipip.net"]);
    assert!(!set.is_excluded("notipip.net"));
}

#[test]
fn domain_fqdn_trailing_dot() {
    let set = make_domain_set(&["ipip.net"]);
    assert!(set.is_excluded("myip.ipip.net."));
}

#[test]
fn domain_non_cn() {
    let set = make_domain_set(&["ipip.net", "baidu.com"]);
    assert!(!set.is_excluded("google.com"));
    assert!(!set.is_excluded("cloudflare.com"));
}

#[tokio::test]
async fn domain_embedded_contains_ipip() {
    let set = DomainSet::load(&["CN".into()], std::path::Path::new("/nonexistent"))
        .await
        .unwrap();
    assert!(set.is_excluded("ipip.net"));
    assert!(set.is_excluded("myip.ipip.net"));
    assert!(!set.is_excluded("google.com"));
    assert!(!set.is_excluded("cloudflare.com"));
}

#[tokio::test]
async fn domain_embedded_contains_yandex() {
    let set = DomainSet::load(&["RU".into()], std::path::Path::new("/nonexistent"))
        .await
        .unwrap();
    assert!(set.is_excluded("yandex.ru"));
    assert!(set.is_excluded("mail.ru"));
    assert!(set.is_excluded("sub.yandex.ru"));
    assert!(!set.is_excluded("google.com"));
    assert!(!set.is_excluded("cloudflare.com"));
}

#[tokio::test]
async fn domain_embedded_multiple_countries() {
    // Loading CN and RU together should exclude domains from both lists.
    let set = DomainSet::load(
        &["CN".into(), "RU".into()],
        std::path::Path::new("/nonexistent"),
    )
    .await
    .unwrap();
    assert!(set.is_excluded("baidu.com"));
    assert!(set.is_excluded("yandex.ru"));
    assert!(!set.is_excluded("google.com"));
}
