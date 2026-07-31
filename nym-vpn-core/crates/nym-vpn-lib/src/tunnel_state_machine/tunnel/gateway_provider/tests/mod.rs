// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only
#![allow(unused)]

use std::{collections::HashMap, time::Duration};

use nym_gateway_directory::{
    Asn, AsnKind, BlacklistedGateways, Config, Gateway, GatewayType, Location, Performance,
    ScoreValue,
};
use nym_sdk::UserAgent;
use nym_vpn_api_client::response::NodeFamily;
use nym_vpn_lib_types::{
    EntryPoint, ExitPoint, GatewayIndependence, TentativeGateways, TunnelType,
};
use nym_vpn_store::keys::wireguard::WireguardKeysDb;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::tunnel_state_machine::tunnel::gateway_provider::{
    error::GatewayProviderError, gateway_cache::tests::MockGatewayCache,
    geo_ip::tests::MockGeoIpClient, selector::select_gateways,
};

use super::*;

/// Minimal gateway data persisted in `mainnet_gateways.json`.
/// Only the fields needed for the synthetic-family test scenarios are kept;
/// everything else (performance, probe data, etc.) is filled with defaults
/// at test runtime by [`record_to_gateway`].
#[derive(Serialize, Deserialize)]
struct GatewayRecord {
    id: String,
    name: String,
    cc: Option<String>,
    asn: Option<String>,
    route: Option<String>,
}

fn gateway_to_record(gw: &Gateway) -> GatewayRecord {
    GatewayRecord {
        id: gw.identity.to_base58_string(),
        name: gw.name.trim().to_string(),
        cc: gw
            .location
            .as_ref()
            .map(|l| l.two_letter_iso_country_code.clone()),
        asn: gw
            .location
            .as_ref()
            .and_then(|l| l.asn.as_ref())
            .map(|a| a.asn.clone()),
        route: gw
            .location
            .as_ref()
            .and_then(|l| l.asn.as_ref())
            .map(|a| a.route.to_string()),
    }
}

fn record_to_gateway(r: &GatewayRecord) -> Gateway {
    let perf = Performance {
        last_updated_utc: Default::default(),
        score: ScoreValue::High,
        mixnet_score: ScoreValue::High,
        load: ScoreValue::Low,
        uptime_percentage_last_24_hours: Default::default(),
    };
    let location = r.cc.as_deref().map(|cc| Location {
        two_letter_iso_country_code: cc.to_string(),
        asn: match (&r.asn, &r.route) {
            (Some(asn), Some(route)) => Some(Asn {
                asn: asn.clone(),
                name: String::new(),
                route: route.parse().expect("invalid route in snapshot"),
                kind: AsnKind::Other,
            }),
            _ => None,
        },
        ..Default::default()
    });
    if let Some(loc) = location {
        Gateway::builder()
            .identity(r.id.parse().expect("invalid gateway id in snapshot"))
            .name(r.name.clone())
            .performance(perf)
            .location(loc)
            .build()
    } else {
        Gateway::builder()
            .identity(r.id.parse().expect("invalid gateway id in snapshot"))
            .name(r.name.clone())
            .performance(perf)
            .build()
    }
}

// ─── shared test constants and gateway-builder helpers ─────────────────────

const GW_A: &str = "5XjrYTRu5j5npwtMenq5vanVtDBREu3iX5yymS8qxnu9";
const GW_B: &str = "5ZWdDN9pQ18vYkYYs5ZERh4P4JLtMiijscZ6FvwSfVxR";
const GW_C: &str = "4X7zJBWts5VmB3bFbK7RUo37w2U2AqLKn7Z3fDaWHoy4";

/// Parse a gateway's raw identity into `nym_vpn_lib_types::NodeIdentity` so it
/// can be used in `EntryPoint::Gateway` / `ExitPoint::Gateway` settings.
fn gw_identity(gw: &Gateway) -> nym_vpn_lib_types::NodeIdentity {
    gw.identity
        .to_base58_string()
        .parse()
        .expect("gateway identity must be a valid base58 ed25519 key")
}

/// Gateway with all three independence-relevant fields populated:
/// node family, ASN, and subnet (route). `asn` is a plain AS number string
/// (e.g. `"AS100"`) and `route` is a CIDR string (e.g. `"10.0.0.0/24"`).
fn gw_full(id: &str, family_id: u32, asn: &str, route: &str) -> Gateway {
    Gateway::builder()
        .identity(id.parse().unwrap())
        .performance(Performance {
            last_updated_utc: Default::default(),
            score: ScoreValue::High,
            mixnet_score: ScoreValue::High,
            load: ScoreValue::Low,
            uptime_percentage_last_24_hours: Default::default(),
        })
        .family_data(Some(NodeFamily {
            id: family_id,
            name: format!("Family {family_id}"),
            description: String::new(),
            family_stake: 0,
            members: 2,
        }))
        .location(Location {
            two_letter_iso_country_code: String::new(),
            asn: Some(Asn {
                asn: asn.to_string(),
                name: String::new(),
                route: route.parse().expect("invalid test CIDR route"),
                kind: AsnKind::Other,
            }),
            ..Default::default()
        })
        .build()
}

// ─── mainnet client helpers (needed only for the live-data test) ────────────

fn user_agent() -> UserAgent {
    UserAgent {
        application: "test".to_string(),
        version: "0.0.1".to_string(),
        platform: "test".to_string(),
        git_commit: "test".to_string(),
    }
}

fn new_mainnet() -> Config {
    let mainnet_network_defaults = nym_sdk::NymNetworkDetails::default();
    let default_nyxd_url = mainnet_network_defaults
        .endpoints
        .first()
        .expect("rust sdk mainnet default incorrectly configured")
        .nyxd_url();
    let default_api_urls = mainnet_network_defaults.nym_api_urls();
    let default_nym_vpn_api_urls = mainnet_network_defaults.nym_vpn_api_urls();
    Config::new(
        default_nyxd_url,
        default_api_urls,
        default_nym_vpn_api_urls,
        None,
    )
    .unwrap()
}

fn mainnet_gateway_client() -> GatewayClient {
    let config = new_mainnet();
    GatewayClient::new(config, user_agent()).unwrap()
}

pub fn default_tunnel_settings() -> TunnelSettings {
    TunnelSettings {
        enable_ipv6: false,
        tunnel_type: TunnelType::Wireguard,
        allow_lan: false,
        enable_ad_blocking: false,
        residential_exit: false,
        mixnet_tunnel_options: Default::default(),
        wireguard_tunnel_options: Default::default(),
        gateway_performance_options: Default::default(),
        mixnet_client_config: None,
        entry_point: Box::new(EntryPoint::Random),
        exit_point: Box::new(ExitPoint::Random),
        dns: Default::default(),
        split_tunnel: Default::default(),
        gateway_selection_algorithm_config: Default::default(),
        geo_exclusion_settings: Default::default(),
        gateway_independence: GatewayIndependence {
            different_asn: false,
            different_node_family: false,
            different_subnet: false,
            ..Default::default()
        },
    }
}

pub fn gateway_id_to_gateway(id: &str) -> Gateway {
    Gateway::builder()
        .identity(id.parse().unwrap())
        .performance(Performance {
            last_updated_utc: Default::default(),
            score: ScoreValue::High,
            mixnet_score: ScoreValue::High,
            load: ScoreValue::Low,
            uptime_percentage_last_24_hours: Default::default(),
        })
        .build()
}

#[tokio::test]
async fn error_stream() {
    let shutdown_token = CancellationToken::new();
    let gateways = Arc::new(RwLock::new(None));
    let mut tunnel_settings = default_tunnel_settings();
    tunnel_settings
        .gateway_selection_algorithm_config
        .enable_geo_location = false;
    let (mut gw_provider, handle) = GatewayProvider::new(
        MockGatewayCache::new(gateways),
        MockGeoIpClient::new(),
        tunnel_settings,
        WireguardKeysDb::Ephemeral(Default::default()),
        shutdown_token.child_token(),
    );
    // No gateways come out of the stream when there are no gateways to select from
    assert!(
        tokio::time::timeout(Duration::from_millis(100), gw_provider.next())
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    shutdown_token.cancel();
    handle.await.unwrap();
}

#[tokio::test]
async fn set_and_stream() {
    let shutdown_token = CancellationToken::new();
    let possible_gateways = [
        "2zHiExNRKiCXVKS35SNKtK4apGfZELMpA1jJ2gVevJoz",
        "38zcSsvjXsAX7C28ko2H3Lt55X4TYxfZYkPADxKXZHUj",
    ]
    .map(gateway_id_to_gateway);
    let gateways = Arc::new(RwLock::new(Some(possible_gateways.to_vec())));
    let mut tunnel_settings = default_tunnel_settings();
    tunnel_settings
        .gateway_selection_algorithm_config
        .enable_geo_location = false;
    let (mut gw_provider, handle) = GatewayProvider::new(
        MockGatewayCache::new(gateways),
        MockGeoIpClient::new(),
        tunnel_settings,
        WireguardKeysDb::Ephemeral(Default::default()),
        shutdown_token.child_token(),
    );
    gw_provider
        .set_tunnel_settings(default_tunnel_settings())
        .await
        .unwrap();
    // check we have "infinite" stream
    for _ in 0..100 {
        gw_provider.next().await.unwrap().unwrap();
    }

    shutdown_token.cancel();
    handle.await.unwrap();
}

/// Regression test for the intermittent `NoGatewaysAvailable` race.
///
/// `set_tunnel_settings` (triggered on every connect press via
/// `set_gateway_independence`) swaps in a brand-new, empty selection stream and
/// asks the algorithm to recompute. If `tentative_gateways` is queried before
/// the first fresh selection lands in that stream, it must wait for it rather
/// than immediately reporting `NoGatewaysAvailable`
#[tokio::test]
async fn tentative_gateways_waits_for_fresh_selection_after_reset() {
    let shutdown_token = CancellationToken::new();
    let possible_gateways = [
        "2zHiExNRKiCXVKS35SNKtK4apGfZELMpA1jJ2gVevJoz",
        "38zcSsvjXsAX7C28ko2H3Lt55X4TYxfZYkPADxKXZHUj",
    ]
    .map(gateway_id_to_gateway);
    let gateways = Arc::new(RwLock::new(Some(possible_gateways.to_vec())));
    let mut tunnel_settings = default_tunnel_settings();
    tunnel_settings
        .gateway_selection_algorithm_config
        .enable_geo_location = false;

    let cache = MockGatewayCache::new_with_lookup_delay(gateways, Duration::from_millis(50));
    let (gw_provider, handle) = GatewayProvider::new(
        cache,
        MockGeoIpClient::new(),
        tunnel_settings.clone(),
        WireguardKeysDb::Ephemeral(Default::default()),
        shutdown_token.child_token(),
    );

    // Reset the stream (as set_gateway_independence does on every connect press)
    // and immediately query, before the freshly computed selection is ready.
    gw_provider
        .set_tunnel_settings(tunnel_settings)
        .await
        .unwrap();
    let tentative = gw_provider.tentative_gateways().await;

    assert!(
        matches!(tentative, TentativeGateways::Selected { .. }),
        "tentative_gateways must wait for the freshly computed selection instead \
         of returning NoGatewaysAvailable; got {tentative:?}"
    );

    shutdown_token.cancel();
    handle.await.unwrap();
}

#[tokio::test]
async fn mainnet_syntethic_node_families() {
    fn lcp_len(a: &str, b: &str) -> usize {
        a.chars()
            .zip(b.chars())
            .take_while(|(ca, cb)| ca == cb)
            .map(|(c, _)| c.len_utf8())
            .sum()
    }

    fn prefix_cluster(
        mut names: Vec<String>,
        min_prefix: usize,
    ) -> (Vec<Vec<String>>, Vec<String>) {
        names.sort();
        let mut groups: Vec<Vec<String>> = Vec::new();
        for name in names {
            let joins = groups.last().is_some_and(|g| {
                !name.is_empty() && !g[0].is_empty() && lcp_len(&g[0], &name) >= min_prefix
            });
            if joins {
                groups.last_mut().unwrap().push(name);
            } else {
                groups.push(vec![name]);
            }
        }
        let (families, singletons): (Vec<_>, Vec<_>) =
            groups.into_iter().partition(|g| g.len() > 1);
        (families, singletons.into_iter().flatten().collect())
    }

    fn family_label(members: &[String]) -> String {
        let prefix_len = members.iter().skip(1).fold(members[0].len(), |acc, m| {
            acc.min(lcp_len(&members[0][..acc], m.as_str()))
        });
        if prefix_len >= 3 {
            return members[0][..prefix_len]
                .trim_end_matches(|c: char| {
                    c.is_ascii_digit() || (!c.is_alphanumeric() && c != ']')
                })
                .to_string();
        }
        let rev0: String = members[0].chars().rev().collect();
        let suffix_len = members.iter().skip(1).fold(rev0.len(), |acc, m| {
            let rev_m: String = m.chars().rev().collect();
            acc.min(lcp_len(&rev0[..acc], &rev_m))
        });
        let suffix: String = rev0[..suffix_len].chars().rev().collect();
        format!(
            "*.{}",
            suffix.trim_start_matches(|c: char| !c.is_alphanumeric())
        )
    }

    // ── Load gateway data ────────────────────────────────────────────────
    //
    // Default: load from the persisted snapshot (no network required).
    // To refresh the snapshot, uncomment the block below, run the test once,
    // then re-comment it.
    //
    // --- begin refresh block (uncomment to re-fetch from mainnet and overwrite the snapshot) ---
    // let live_gateways: Vec<Gateway> = mainnet_gateway_client()
    //     .lookup_gateways(GatewayType::Wg)
    //     .await
    //     .unwrap()
    //     .into_iter()
    //     .collect();
    // let snapshot_path = concat!(
    //     env!("CARGO_MANIFEST_DIR"),
    //     "/src/tunnel_state_machine/tunnel/gateway_provider/tests/mainnet_gateways.json"
    // );
    // std::fs::write(
    //     snapshot_path,
    //     serde_json::to_string_pretty(
    //         &live_gateways.iter().map(gateway_to_record).collect::<Vec<_>>(),
    //     )
    //     .unwrap(),
    // )
    // .unwrap();
    // let mut gateways = live_gateways;
    // --- end refresh block ---

    let records: Vec<GatewayRecord> =
        serde_json::from_str(include_str!("mainnet_gateways.json")).unwrap();
    let mut gateways: Vec<Gateway> = records.iter().map(record_to_gateway).collect();

    let names: Vec<String> = gateways.iter().map(|g| g.name.trim().to_string()).collect();

    let (mut families, singletons) = prefix_cluster(names, 5);
    let reversed: Vec<String> = singletons
        .iter()
        .map(|n| n.chars().rev().collect())
        .collect();
    let (suffix_groups, remaining_reversed) = prefix_cluster(reversed, 8);
    for mut group in suffix_groups {
        for name in &mut group {
            *name = name.chars().rev().collect();
        }
        families.push(group);
    }
    let singletons: Vec<String> = remaining_reversed
        .into_iter()
        .map(|n| n.chars().rev().collect())
        .collect();
    let (short_families, singletons) = prefix_cluster(singletons, 3);
    families.extend(short_families);

    let mut name_to_family: HashMap<String, (u32, String, usize)> = HashMap::new();
    for (id, family) in families.iter().enumerate() {
        let label = family_label(family);
        for name in family {
            name_to_family.insert(name.clone(), (id as u32, label.clone(), family.len()));
        }
    }

    for gw in &mut gateways {
        let name = gw.name.trim().to_string();
        if let Some((id, label, size)) = name_to_family.get(&name) {
            gw.family_data = Some(NodeFamily {
                id: *id,
                name: label.clone(),
                description: String::new(),
                family_stake: 0,
                members: *size,
            });
        }
    }

    // ── Print partition with ASN annotations ─────────────────────────────
    families.sort_by(|a, b| a[0].cmp(&b[0]));
    println!(
        "{} families, {} singletons\n",
        families.len(),
        singletons.len()
    );
    for members in &families {
        println!("  {} ({} members)", family_label(members), members.len());
        for name in members {
            let gw = gateways
                .iter()
                .find(|g| g.name.trim() == name.as_str())
                .unwrap();
            let asn = gw
                .location
                .as_ref()
                .and_then(|l| l.asn.as_ref())
                .map(|a| a.asn.as_str())
                .unwrap_or("no-ASN");
            let cc = gw
                .location
                .as_ref()
                .map(|l| l.two_letter_iso_country_code.as_str())
                .unwrap_or("??");
            println!("    - {name} [{cc}] ASN={asn}");
        }
    }
    println!("\nSingletons ({}):", singletons.len());
    for name in &singletons {
        println!("  - {name}");
    }

    // ── Index structures shared by all scenarios ──────────────────────────
    let mut by_family_id: HashMap<u32, Vec<Gateway>> = HashMap::new();
    for gw in &gateways {
        if let Some(nf) = &gw.family_data {
            by_family_id.entry(nf.id).or_default().push(gw.clone());
        }
    }
    let mut by_asn: HashMap<String, Vec<Gateway>> = HashMap::new();
    for gw in &gateways {
        if let Some(asn) = gw.location.as_ref().and_then(|l| l.asn.as_ref()) {
            by_asn.entry(asn.asn.clone()).or_default().push(gw.clone());
        }
    }
    let mut by_subnet: HashMap<String, Vec<Gateway>> = HashMap::new();
    for gw in &gateways {
        if let Some(route) = gw
            .location
            .as_ref()
            .and_then(|l| l.asn.as_ref())
            .map(|a| a.route.to_string())
        {
            by_subnet.entry(route).or_default().push(gw.clone());
        }
    }
    let mut by_country: HashMap<String, Vec<Gateway>> = HashMap::new();
    for gw in &gateways {
        if let Some(loc) = &gw.location {
            by_country
                .entry(loc.two_letter_iso_country_code.clone())
                .or_default()
                .push(gw.clone());
        }
    }

    // One shared full-gateway cache used by every scenario.
    let full_cache_data = Arc::new(RwLock::new(Some(gateways.clone())));

    let default_settings = || {
        let mut s = default_tunnel_settings();
        s.gateway_independence = GatewayIndependence::default();
        s
    };

    // Helper: run select_gateways against the full gateway pool with the
    // given settings (which control which entry/exit is requested).
    let full_select = |settings: TunnelSettings| {
        let cache = MockGatewayCache::new(full_cache_data.clone());
        async move {
            select_gateways(
                cache,
                &BlacklistedGateways::new(),
                &settings,
                None,
                &WireguardKeysDb::Ephemeral(Default::default()),
            )
            .await
        }
    };

    // (gw_identity is defined at module level)

    // ── Scenario A: same synthetic family → impossible under default criteria ──
    // Explicitly request two gateways that share a synthetic family ID.
    // The default independence settings (different_node_family: true) make this
    // an impossible pair, even though the full gateway pool is available.
    let same_family_pair = by_family_id.values().find(|v| v.len() >= 2).map(|v| {
        let label = v[0].family_data.as_ref().unwrap().name.clone();
        (v[0].clone(), v[1].clone(), label)
    });
    if let Some((gw_a, gw_b, family_name)) = same_family_pair {
        println!(
            "\nScenario A — same family '{}': entry={} exit={}",
            family_name,
            gw_a.name.trim(),
            gw_b.name.trim()
        );
        let mut settings = default_settings();
        *settings.entry_point = EntryPoint::Gateway {
            identity: gw_identity(&gw_a),
        };
        *settings.exit_point = ExitPoint::Gateway {
            identity: gw_identity(&gw_b),
        };
        let result = full_select(settings).await;
        assert!(
            matches!(
                result,
                Err(GatewayProviderError::NeedsRelaxedIndependenceCriteria)
            ),
            "same-family pair under default criteria should need relaxed criteria; got {result:?}"
        );
    } else {
        println!("Scenario A skipped: no synthetic family with ≥2 members found");
    }

    // ── Scenario B: same ASN → impossible under default criteria ─────────
    // Explicitly request two gateways that share the same AS number.
    // The default independence settings (different_asn: true) reject this pair.
    let same_asn_pair = by_asn
        .iter()
        .find(|(_, v)| v.len() >= 2)
        .map(|(asn, v)| (asn.clone(), v[0].clone(), v[1].clone()));
    if let Some((asn_str, gw_a, gw_b)) = same_asn_pair {
        println!(
            "\nScenario B — same ASN {}: entry={} exit={}",
            asn_str,
            gw_a.name.trim(),
            gw_b.name.trim()
        );
        let mut settings = default_settings();
        *settings.entry_point = EntryPoint::Gateway {
            identity: gw_identity(&gw_a),
        };
        *settings.exit_point = ExitPoint::Gateway {
            identity: gw_identity(&gw_b),
        };
        let result = full_select(settings).await;
        assert!(
            matches!(
                result,
                Err(GatewayProviderError::NeedsRelaxedIndependenceCriteria)
            ),
            "same-ASN pair under default criteria should need relaxed criteria; got {result:?}"
        );
    } else {
        println!("Scenario B skipped: no two gateways share an ASN in mainnet data");
    }

    // ── Scenario C: overlapping subnet → impossible under default criteria ─
    // Explicitly request two gateways that announce the same IP prefix.
    // The default independence settings (different_subnet: true) reject this pair.
    let same_subnet_pair = by_subnet
        .iter()
        .find(|(_, v)| v.len() >= 2)
        .map(|(route, v)| (route.clone(), v[0].clone(), v[1].clone()));
    if let Some((route_str, gw_a, gw_b)) = same_subnet_pair {
        println!(
            "\nScenario C — same subnet {}: entry={} exit={}",
            route_str,
            gw_a.name.trim(),
            gw_b.name.trim()
        );
        let mut settings = default_settings();
        *settings.entry_point = EntryPoint::Gateway {
            identity: gw_identity(&gw_a),
        };
        *settings.exit_point = ExitPoint::Gateway {
            identity: gw_identity(&gw_b),
        };
        let result = full_select(settings).await;
        assert!(
            matches!(
                result,
                Err(GatewayProviderError::NeedsRelaxedIndependenceCriteria)
            ),
            "same-subnet pair under default criteria should need relaxed criteria; got {result:?}"
        );
    } else {
        println!("Scenario C skipped: no two gateways share a subnet route in mainnet data");
    }

    // ── Scenario D: random selection succeeds across the entire mainnet pool ─
    // With no explicit entry/exit the selector picks freely; the full mainnet
    // is diverse enough to always find a pair that satisfies all default criteria.
    println!("\nScenario D — random entry+exit on entire mainnet pool");
    let result = full_select(default_settings()).await;
    assert!(
        result.is_ok(),
        "random selection on full pool under default criteria should succeed; got {result:?}"
    );
    let selected = result.unwrap();
    let entry_family = selected.entry_gateway().family_data.as_ref().map(|f| f.id);
    let exit_family = selected.exit_gateway().family_data.as_ref().map(|f| f.id);
    if let (Some(ef), Some(xf)) = (entry_family, exit_family) {
        assert_ne!(
            ef, xf,
            "selected entry and exit must not share a synthetic family"
        );
    }
    let entry_asn = selected
        .entry_gateway()
        .location
        .as_ref()
        .and_then(|l| l.asn.as_ref())
        .map(|a| a.asn.clone());
    let exit_asn = selected
        .exit_gateway()
        .location
        .as_ref()
        .and_then(|l| l.asn.as_ref())
        .map(|a| a.asn.clone());
    if let (Some(ea), Some(xa)) = (entry_asn, exit_asn) {
        assert_ne!(ea, xa, "selected entry and exit must not share an ASN");
    }
    println!(
        "  entry: {} exit: {}",
        selected.entry_gateway().name.trim(),
        selected.exit_gateway().name.trim()
    );

    // ── Scenario E: same-country entry+exit with single-ASN country ──────
    // When both entry and exit are country-constrained to a country whose
    // gateways all share one AS, the default criteria (different_asn: true)
    // can never be satisfied regardless of how many gateways are in the pool.
    // We check every such country, not just the first one.
    let single_asn_countries: Vec<(String, Vec<Gateway>)> = by_country
        .iter()
        .filter(|(_, gws)| {
            gws.len() >= 2
                && gws
                    .iter()
                    .all(|gw| gw.location.as_ref().and_then(|l| l.asn.as_ref()).is_some())
                && {
                    let first_asn = gws[0]
                        .location
                        .as_ref()
                        .and_then(|l| l.asn.as_ref())
                        .map(|a| &a.asn);
                    gws.iter().all(|gw| {
                        gw.location
                            .as_ref()
                            .and_then(|l| l.asn.as_ref())
                            .map(|a| &a.asn)
                            == first_asn
                    })
                }
        })
        .map(|(cc, gws)| (cc.clone(), gws.clone()))
        .collect();
    if single_asn_countries.is_empty() {
        println!("Scenario E skipped: no country with ≥2 gateways sharing one ASN found");
    }
    for (cc, country_gws) in single_asn_countries {
        let asn_label = country_gws[0]
            .location
            .as_ref()
            .and_then(|l| l.asn.as_ref())
            .map(|a| a.asn.as_str())
            .unwrap_or("?");
        println!(
            "\nScenario E — entry+exit both in country {} ({} gateways, all ASN {})",
            cc,
            country_gws.len(),
            asn_label
        );
        let mut settings = default_settings();
        *settings.entry_point = EntryPoint::Country {
            two_letter_iso_country_code: cc.clone(),
        };
        *settings.exit_point = ExitPoint::Country {
            two_letter_iso_country_code: cc.clone(),
        };
        let result = full_select(settings).await;
        assert!(
            matches!(
                result,
                Err(GatewayProviderError::NeedsRelaxedIndependenceCriteria)
            ),
            "country {cc}: same single-ASN country for entry+exit under default criteria should need relaxed criteria; got {result:?}"
        );
    }

    // ── Scenario F: same-country entry+exit with single-family country ────
    // When both entry and exit are country-constrained to a country whose
    // gateways all belong to the same synthetic node family, the default
    // criteria (different_node_family: true) can never be satisfied.
    // We check every such country, not just the first one.
    let single_family_countries: Vec<(String, Vec<Gateway>)> = by_country
        .iter()
        .filter(|(_, gws)| {
            gws.len() >= 2 && gws.iter().all(|gw| gw.family_data.is_some()) && {
                let first_family = gws[0].family_data.as_ref().map(|f| f.id);
                gws.iter()
                    .all(|gw| gw.family_data.as_ref().map(|f| f.id) == first_family)
            }
        })
        .map(|(cc, gws)| (cc.clone(), gws.clone()))
        .collect();
    if single_family_countries.is_empty() {
        println!(
            "Scenario F skipped: no country with ≥2 gateways all in the same synthetic family"
        );
    }
    for (cc, country_gws) in single_family_countries {
        let family_label = country_gws[0]
            .family_data
            .as_ref()
            .map(|f| f.name.as_str())
            .unwrap_or("?");
        println!(
            "\nScenario F — entry+exit both in country {} ({} gateways, all family '{}')",
            cc,
            country_gws.len(),
            family_label
        );
        let mut settings = default_settings();
        *settings.entry_point = EntryPoint::Country {
            two_letter_iso_country_code: cc.clone(),
        };
        *settings.exit_point = ExitPoint::Country {
            two_letter_iso_country_code: cc.clone(),
        };
        let result = full_select(settings).await;
        assert!(
            matches!(
                result,
                Err(GatewayProviderError::NeedsRelaxedIndependenceCriteria)
            ),
            "country {cc}: same single-family country for entry+exit under default criteria should need relaxed criteria; got {result:?}"
        );
    }
}

// ─── targeted independence unit tests ──────────────────────────────────────

/// When every gateway in the pool belongs to the same synthetic family (but
/// has distinct ASNs and subnets so only the family criterion fires), no
/// independent pair can be formed under the default criteria and the selector
/// must signal `NeedsRelaxedIndependenceCriteria`.
#[tokio::test]
async fn all_gateways_same_family_blocks_selection() {
    let gateways = vec![
        gw_full(GW_A, 1, "AS100", "10.0.0.0/24"),
        gw_full(GW_B, 1, "AS200", "10.1.0.0/24"),
        gw_full(GW_C, 1, "AS300", "10.2.0.0/24"),
    ];
    let cache = MockGatewayCache::new(Arc::new(RwLock::new(Some(gateways))));

    let mut settings = default_tunnel_settings();
    settings.gateway_independence = GatewayIndependence::default();

    let result = select_gateways(
        cache,
        &BlacklistedGateways::new(),
        &settings,
        None,
        &WireguardKeysDb::Ephemeral(Default::default()),
    )
    .await;

    assert!(
        matches!(
            result,
            Err(GatewayProviderError::NeedsRelaxedIndependenceCriteria)
        ),
        "all-same-family pool should require relaxed criteria; got {result:?}"
    );
}

/// When the pool is designed so that exactly one unordered pair satisfies all
/// three default independence criteria simultaneously, random selection must
/// always land on that pair.
///
/// Gateway design (default criteria: different_family AND different_asn AND different_subnet):
///
///   GW_A — family 1, AS100, 10.0.0.0/24
///   GW_B — family 2, AS100, 10.0.0.0/24  ← same ASN + subnet as GW_A
///   GW_C — family 1, AS200, 10.1.0.0/24  ← same family as GW_A
///
/// Pair analysis:
///   (A, B) — AS100 == AS100  → fails ASN criterion
///   (A, C) — family 1 == 1  → fails family criterion
///   (B, C) — family 2 ≠ 1, AS100 ≠ AS200, subnets differ → only valid pair
///
/// Regardless of which of B or C the selector picks as entry, it must always
/// produce the {GW_B, GW_C} pair.
#[tokio::test]
async fn single_valid_pair_is_always_chosen() {
    let gw_b = gw_full(GW_B, 2, "AS100", "10.0.0.0/24");
    let gw_c = gw_full(GW_C, 1, "AS200", "10.1.0.0/24");

    let cache = MockGatewayCache::new(Arc::new(RwLock::new(Some(vec![
        gw_full(GW_A, 1, "AS100", "10.0.0.0/24"),
        gw_b,
        gw_c,
    ]))));

    let mut settings = default_tunnel_settings();
    settings.gateway_independence = GatewayIndependence::default();

    let selected = select_gateways(
        cache,
        &BlacklistedGateways::new(),
        &settings,
        None,
        &WireguardKeysDb::Ephemeral(Default::default()),
    )
    .await
    .expect("GW_B + GW_C is a valid independent pair — selection must succeed");

    let entry_id = selected.entry_gateway().identity.to_base58_string();
    let exit_id = selected.exit_gateway().identity.to_base58_string();
    assert!(
        (entry_id == GW_B && exit_id == GW_C) || (entry_id == GW_C && exit_id == GW_B),
        "only the GW_B + GW_C pair satisfies all independence criteria (in either order); \
         got entry={entry_id} exit={exit_id}"
    );
}

/// An empty gateway pool yields a hard error (no gateways available), not a
/// `NeedsRelaxedIndependenceCriteria` hint — relaxing criteria cannot create
/// gateways out of thin air.
#[tokio::test]
async fn empty_gateway_pool_returns_error() {
    let cache = MockGatewayCache::new(Arc::new(RwLock::new(Some(vec![]))));

    let result = select_gateways(
        cache,
        &BlacklistedGateways::new(),
        &default_tunnel_settings(),
        None,
        &WireguardKeysDb::Ephemeral(Default::default()),
    )
    .await;

    assert!(
        result.is_err(),
        "empty pool must return an error; got {result:?}"
    );
    assert!(
        !matches!(
            result,
            Err(GatewayProviderError::NeedsRelaxedIndependenceCriteria)
        ),
        "empty-pool error must not be NeedsRelaxedIndependenceCriteria \
         (relaxing criteria cannot help an empty pool)"
    );
}
