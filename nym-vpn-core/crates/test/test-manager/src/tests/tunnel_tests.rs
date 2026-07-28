// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::tests::{
    TestContext,
    helpers_nym::{self, ExpectedTunnelState, resolve_hostname_with_retry, wait_for_tunnel_state},
    nym_test::dc_and_ensure_logged_in,
};
use anyhow::{Context, bail, ensure};
use nym_vpn_lib_types::{ExitPoint, GatewayType, ListGatewaysOptions};
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use test_macro::test_function_nym;
use test_rpc::{
    NymServiceClient,
    nym_daemon::{ObservedTunnelState, ObservedTunnelType},
};

/// Parse `resolvectl dns` link lines, e.g. `Link 5 (tun1): 127.111.152.46`.
fn parse_resolvectl_dns(output: &str) -> Vec<(String, Vec<String>)> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (link, servers) = line.strip_prefix("Link ")?.split_once(':')?;
            let name = link.split_once('(')?.1.strip_suffix(')')?.to_string();
            Some((
                name,
                servers
                    .split_whitespace()
                    .map(|server| server.to_string())
                    .collect(),
            ))
        })
        .collect()
}

/// Nameservers systemd-resolved has configured on the tunnel interfaces.
/// Returns `None` when resolved is not managing DNS on this VM.
async fn get_tunnel_link_nameservers(
    rpc: &NymServiceClient,
) -> Result<Option<Vec<String>>, anyhow::Error> {
    let Ok(output) = rpc.exec("resolvectl", ["dns"]).await else {
        return Ok(None);
    };
    if !output.success() {
        return Ok(None);
    }

    let links = parse_resolvectl_dns(&String::from_utf8_lossy(&output.stdout));
    Ok(Some(
        links
            .into_iter()
            .filter(|(name, _)| name.starts_with("tun"))
            .flat_map(|(_, servers)| servers)
            .collect(),
    ))
}

const DNS_PORT: u16 = 53;
const RESOLVER_PROBE_BIND_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0);

fn resolver_probe_addr(nameserver: &str) -> Option<SocketAddr> {
    nameserver
        .parse::<IpAddr>()
        .ok()
        .map(|ip| SocketAddr::new(ip, DNS_PORT))
}

async fn reachable_resolvers(rpc: &NymServiceClient, nameservers: &[String]) -> Vec<SocketAddr> {
    let mut reachable = Vec::new();
    for nameserver in nameservers {
        let Some(dest) = resolver_probe_addr(nameserver) else {
            log::warn!("Skipping unparseable nameserver {nameserver}");
            continue;
        };
        match rpc.send_tcp(None, RESOLVER_PROBE_BIND_ADDR, dest).await {
            Ok(()) => reachable.push(dest),
            Err(error) => log::info!("Resolver {dest} not reachable on TCP: {error}"),
        }
    }
    reachable
}

#[cfg(test)]
mod resolvectl_tests {
    use super::{DNS_PORT, parse_resolvectl_dns, resolver_probe_addr};

    #[test]
    fn resolver_probe_addr_covers_both_families_and_rejects_hostnames() {
        assert_eq!(
            resolver_probe_addr("172.29.1.1").map(|addr| (addr.to_string(), addr.port())),
            Some(("172.29.1.1:53".to_string(), DNS_PORT))
        );
        assert_eq!(
            resolver_probe_addr("2620:fe::fe").map(|addr| addr.to_string()),
            Some("[2620:fe::fe]:53".to_string())
        );
        assert_eq!(resolver_probe_addr("dns.example.com"), None);
        assert_eq!(resolver_probe_addr(""), None);
    }

    #[test]
    fn parses_link_nameservers_and_ignores_global_and_empty_links() {
        let output = "Global:\nLink 2 (eth0): 172.29.1.1\nLink 4 (tun0):\nLink 5 (tun1): 127.111.152.46 10.1.0.1\n";
        let links = parse_resolvectl_dns(output);

        assert_eq!(
            links,
            vec![
                ("eth0".to_string(), vec!["172.29.1.1".to_string()]),
                ("tun0".to_string(), vec![]),
                (
                    "tun1".to_string(),
                    vec!["127.111.152.46".to_string(), "10.1.0.1".to_string()]
                ),
            ]
        );
    }

    #[test]
    fn returns_nothing_for_output_without_link_lines() {
        assert!(parse_resolvectl_dns("Global: 1.1.1.1\n").is_empty());
        assert!(parse_resolvectl_dns("").is_empty());
    }
}

async fn get_vm_nameservers(rpc: &NymServiceClient) -> Result<Vec<String>, anyhow::Error> {
    let resolv_output = rpc
        .exec("cat", ["/etc/resolv.conf"])
        .await
        .context("Failed to read /etc/resolv.conf in VM")?;
    let resolv_str = String::from_utf8_lossy(&resolv_output.stdout);
    Ok(resolv_str
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with("nameserver") {
                line.split_whitespace().nth(1).map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect())
}

#[test_function_nym(priority = 1)]
pub async fn test_daemon_info(
    _test_context: TestContext,
    _rpc: NymServiceClient,
    mut nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let info = nym_client.get_info().await.context("get_info() failed")?;
    log::info!("Daemon version: {}", info.version);
    log::info!("Build timestamp: {:?}", info.build_timestamp);
    log::info!("Nym network: {:?}", info.nym_network);
    ensure!(
        !info.version.is_empty(),
        "Daemon version should not be empty"
    );
    ensure!(
        !info.nym_network.network_name.is_empty(),
        "Daemon should report a non-empty nym network name"
    );

    let feature_flags = nym_client
        .get_feature_flags()
        .await
        .context("get_feature_flags() failed")?;
    log::info!("Feature flags: {:?}", feature_flags);

    let system_messages = nym_client
        .get_system_messages()
        .await
        .context("get_system_messages() failed")?;
    log::info!("System messages count: {}", system_messages.len());

    Ok(())
}

#[test_function_nym(priority = 3)]
pub async fn test_list_gateways(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    let (mixnet_gateways, nym_client) = helpers_nym::call_nym_with_transport_recovery(
        &test_context.rpc_provider,
        nym_client,
        |mut client| async move {
            let result = client
                .list_gateways(ListGatewaysOptions {
                    gw_type: GatewayType::MixnetEntry,
                    user_agent: None,
                })
                .await;
            (client, result)
        },
    )
    .await
    .context("list_gateways(MixnetEntry) failed")?;
    log::info!("Mixnet entry gateways: {}", mixnet_gateways.len());
    ensure!(
        !mixnet_gateways.is_empty(),
        "Expected at least one MixnetEntry gateway"
    );

    let (wg_gateways, _nym_client) = helpers_nym::call_nym_with_transport_recovery(
        &test_context.rpc_provider,
        nym_client,
        |mut client| async move {
            let result = client
                .list_gateways(ListGatewaysOptions {
                    gw_type: GatewayType::Wg,
                    user_agent: None,
                })
                .await;
            (client, result)
        },
    )
    .await
    .context("list_gateways(Wg) failed")?;
    log::info!("WireGuard gateways: {}", wg_gateways.len());
    ensure!(!wg_gateways.is_empty(), "Expected at least one Wg gateway");

    Ok(())
}

#[test_function_nym(priority = 7)]
pub async fn test_account_summary_and_usage(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    let (summary, nym_client) = helpers_nym::call_nym_with_transport_recovery(
        &test_context.rpc_provider,
        nym_client,
        |mut client| async move {
            let result = client.get_account_summary().await;
            (client, result)
        },
    )
    .await
    .context("get_account_summary() failed")?;
    log::info!("Account summary: {:?}", summary);
    let summary = summary.context("Expected account summary to be present")?;
    ensure!(
        summary.traffic_limit_gb > 0,
        "Expected positive traffic limit, got {}",
        summary.traffic_limit_gb
    );
    ensure!(
        summary.traffic_used_gb <= summary.traffic_limit_gb,
        "Traffic used ({} GB) exceeds limit ({} GB)",
        summary.traffic_used_gb,
        summary.traffic_limit_gb
    );

    let (usages, _nym_client) = helpers_nym::call_nym_with_transport_recovery(
        &test_context.rpc_provider,
        nym_client,
        |mut client| async move {
            let result = client.get_account_usage().await;
            (client, result)
        },
    )
    .await
    .context("get_account_usage() failed")?;
    log::info!("Usage entries: {}", usages.len());
    ensure!(!usages.is_empty(), "Expected at least one usage entry");
    for (i, usage) in usages.iter().enumerate() {
        log::info!(
            "  [{}/{}] valid_until={}, allowance={}GB, used={}GB",
            i + 1,
            usages.len(),
            usage.valid_until_utc,
            usage.bandwidth_allowance_gb,
            usage.bandwidth_used_gb,
        );
        ensure!(
            !usage.valid_until_utc.is_empty(),
            "Usage entry {} has empty valid_until_utc",
            i + 1
        );
        ensure!(
            usage.bandwidth_allowance_gb > 0.0,
            "Usage entry {} has non-positive bandwidth allowance: {}",
            i + 1,
            usage.bandwidth_allowance_gb
        );
        ensure!(
            usage.bandwidth_used_gb >= 0.0,
            "Usage entry {} has negative bandwidth used: {}",
            i + 1,
            usage.bandwidth_used_gb
        );
    }

    Ok(())
}

#[test_function_nym(priority = 10)]
pub async fn test_wireguard_connect_disconnect(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    let nym_client =
        helpers_nym::set_enable_two_hop_with_recovery(&test_context.rpc_provider, nym_client, true)
            .await?;

    log::info!("Connecting WireGuard tunnel...");
    let nym_client =
        helpers_nym::connect_tunnel_with_recovery(&test_context.rpc_provider, nym_client).await?;
    let (state, nym_client) = wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;
    match state {
        ObservedTunnelState::Connected {
            tunnel_type: ObservedTunnelType::Wireguard,
        } => log::info!("Tunnel type: Wireguard"),
        other => bail!("Expected Connected Wireguard, got {other:?}"),
    }

    log::info!("Disconnecting...");
    helpers_nym::disconnect_and_wait(&rpc, nym_client, &test_context.rpc_provider).await?;

    Ok(())
}

#[test_function_nym(priority = 11)]
pub async fn test_mixnet_connect_disconnect(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    let nym_client = helpers_nym::set_enable_two_hop_with_recovery(
        &test_context.rpc_provider,
        nym_client,
        false,
    )
    .await?;

    log::info!("Connecting Mixnet tunnel (this may take longer)...");
    let nym_client =
        helpers_nym::connect_tunnel_with_recovery(&test_context.rpc_provider, nym_client).await?;
    let (state, nym_client) = wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;
    match state {
        ObservedTunnelState::Connected {
            tunnel_type: ObservedTunnelType::Mixnet,
        } => log::info!("Tunnel type: Mixnet"),
        other => bail!("Expected Connected Mixnet, got {other:?}"),
    }

    log::info!("Disconnecting...");
    helpers_nym::disconnect_and_wait(&rpc, nym_client, &test_context.rpc_provider).await?;

    Ok(())
}

#[test_function_nym(priority = 15)]
pub async fn test_dns_leak(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    let pre_vpn_nameservers = get_vm_nameservers(&rpc).await?;
    log::info!("Pre-VPN nameservers (in VM): {:?}", pre_vpn_nameservers);
    ensure!(
        !pre_vpn_nameservers.is_empty(),
        "VM should have at least one nameserver"
    );

    let reachable_before = reachable_resolvers(&rpc, &pre_vpn_nameservers).await;
    log::info!("Pre-VPN resolvers reachable on TCP: {:?}", reachable_before);

    let nym_client =
        helpers_nym::set_enable_two_hop_with_recovery(&test_context.rpc_provider, nym_client, true)
            .await?;
    log::info!("Connecting tunnel for DNS leak test...");
    let nym_client =
        helpers_nym::connect_tunnel_with_recovery(&test_context.rpc_provider, nym_client).await?;
    let (_, nym_client) = wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    // Under systemd-resolved the daemon sets DNS per link and never rewrites
    // /etc/resolv.conf, so that file still lists the DHCP nameserver by design.
    match get_tunnel_link_nameservers(&rpc).await? {
        Some(tunnel_nameservers) => {
            log::info!("Tunnel link nameservers (in VM): {:?}", tunnel_nameservers);
            ensure!(
                !tunnel_nameservers.is_empty(),
                "tunnel interface has no nameservers configured while connected"
            );
            let leaked: Vec<_> = tunnel_nameservers
                .iter()
                .filter(|ns| pre_vpn_nameservers.contains(ns))
                .collect();
            ensure!(
                leaked.is_empty(),
                "DNS LEAK: tunnel interface still resolves via pre-VPN nameservers: {:?}",
                leaked,
            );
        }
        None => {
            let post_vpn_nameservers = get_vm_nameservers(&rpc).await?;
            log::info!("Post-VPN nameservers (in VM): {:?}", post_vpn_nameservers);
            let leaked: Vec<_> = post_vpn_nameservers
                .iter()
                .filter(|ns| pre_vpn_nameservers.contains(ns))
                .collect();
            ensure!(
                leaked.is_empty(),
                "DNS LEAK: post-VPN resolv.conf still contains pre-VPN nameservers: {:?}",
                leaked,
            );
        }
    }

    if reachable_before.is_empty() {
        log::warn!(
            "No pre-VPN resolver answered on TCP/53, so reachability while connected proves nothing"
        );
    } else {
        let still_reachable = reachable_resolvers(
            &rpc,
            &reachable_before
                .iter()
                .map(|addr| addr.ip().to_string())
                .collect::<Vec<_>>(),
        )
        .await;
        ensure!(
            still_reachable.is_empty(),
            "DNS LEAK: pre-VPN resolvers still reachable while connected: {:?}. \
             The harness enables allow_lan, which may be exposing the LAN resolver",
            still_reachable,
        );
        log::info!("Pre-VPN resolvers are unreachable while connected");
    }

    let addrs = rpc
        .resolve_hostname("nym.com".to_string())
        .await
        .context("DNS resolution failed inside VM while VPN is connected")?;
    log::info!("Resolved nym.com inside VM: {:?}", addrs);
    ensure!(
        !addrs.is_empty(),
        "DNS resolution inside VM returned no addresses while connected"
    );

    let dest = SocketAddr::new(addrs[0].ip(), 443);
    rpc.send_tcp(None, "0.0.0.0:0".parse().unwrap(), dest)
        .await
        .context("TCP connectivity check failed inside VM while VPN is connected")?;
    log::info!("TCP connectivity to {} verified inside VM", dest);

    let bash_ws_result = check_dns_leak_bash_ws(&rpc, &pre_vpn_nameservers).await;
    match bash_ws_result {
        Ok(()) => log::info!("bash.ws DNS leak check passed"),
        Err(e) => {
            let msg = format!("{e:#}");
            if msg.contains("DNS LEAK") {
                bail!("{e}");
            }
            log::warn!("bash.ws check could not complete (service may be down): {e}");
        }
    }

    log::info!("Disconnecting...");
    helpers_nym::disconnect_and_wait(&rpc, nym_client, &test_context.rpc_provider).await?;

    Ok(())
}

/// end to end DNS leak check using bash.ws
///
/// Triggers DNS lookups to bash.ws-controlled subdomains via `ping` from inside
/// the VM to see which resolver IPs were observed: as recommended by bash-ws here
/// https://github.com/macvk/dnsleaktest/blob/3391229768585520b6d0cc6b8939da444efc7273/dnsleaktest.sh
async fn check_dns_leak_bash_ws(
    rpc: &NymServiceClient,
    pre_vpn_nameservers: &[String],
) -> Result<(), anyhow::Error> {
    // obtain a new test ID from bash.ws
    let id_output = rpc
        .exec("curl", ["-s", "--max-time", "10", "https://bash.ws/id"])
        .await
        .context("Failed to reach bash.ws (curl)")?;
    let test_id = String::from_utf8_lossy(&id_output.stdout)
        .trim()
        .to_string();
    log::info!("bash.ws test ID: {}", test_id);
    ensure!(
        !test_id.is_empty() && test_id.chars().all(|c| c.is_alphanumeric() || c == '-'),
        "Failed to obtain valid bash.ws test ID, got: '{}'",
        test_id
    );

    // use ping because the ICMP packet doesn't matter: the DNS lookup to
    // resolve the hostname will reach bash.ws's authoritative nameserver,
    // which records which resolver IP made the query
    const PING_COUNT: i32 = 10;
    for i in 1..=PING_COUNT {
        let hostname = format!("{i}.{test_id}.bash.ws");
        let _ = rpc.exec("ping", ["-c", "1", hostname.as_str()]).await;
    }
    log::info!("bash.ws DNS probes sent ({} pings)", PING_COUNT);

    // let bash.ws breathe to collect results
    tokio::time::sleep(Duration::from_secs(5)).await;

    // fetch results as JSON
    let result_url = format!("https://bash.ws/dnsleak/test/{test_id}?json");
    let result_output = rpc
        .exec("curl", ["-s", "--max-time", "10", result_url.as_str()])
        .await
        .context("Failed to fetch bash.ws results")?;
    let result_str = String::from_utf8_lossy(&result_output.stdout);
    log::info!("bash.ws raw results: {}", result_str);

    let results: Vec<serde_json::Value> =
        serde_json::from_str(&result_str).context("Failed to parse bash.ws results as JSON")?;

    // log public IP and conclusion from bash.ws
    for entry in &results {
        let entry_type = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let ip = entry.get("ip").and_then(|v| v.as_str()).unwrap_or("");
        let country = entry.get("country_name").and_then(|v| v.as_str());
        let asn = entry.get("asn").and_then(|v| v.as_str());
        match entry_type {
            "ip" => log::info!("bash.ws: your IP: {ip}"),
            "dns" => log::info!(
                "bash.ws: DNS resolver: {ip}{}",
                match (country, asn) {
                    (Some(c), Some(a)) if !c.is_empty() && !a.is_empty() => format!(" [{c} {a}]"),
                    (Some(c), _) if !c.is_empty() => format!(" [{c}]"),
                    _ => String::new(),
                }
            ),
            "conclusion" => log::info!("bash.ws conclusion: {ip}"),
            other => log::debug!("bash.ws: unknown entry type '{other}': {ip}"),
        }
    }

    // only check "dns" type entries against pre-VPN nameservers.
    // "ip" entries are the user's public IP, not DNS resolvers.
    let dns_entries: Vec<_> = results
        .iter()
        .filter(|e| e.get("type").and_then(|v| v.as_str()) == Some("dns"))
        .collect();
    ensure!(
        !dns_entries.is_empty(),
        "bash.ws returned no DNS resolver entries — probes may not have reached their servers"
    );

    for entry in &dns_entries {
        if let Some(ip) = entry.get("ip").and_then(|v| v.as_str())
            && pre_vpn_nameservers.contains(&ip.to_string())
        {
            bail!(
                "DNS LEAK DETECTED: bash.ws observed resolver {} which matches a pre-VPN nameserver",
                ip
            );
        }
    }
    log::info!(
        "No DNS leak detected via bash.ws ({} DNS resolvers checked)",
        dns_entries.len()
    );
    Ok(())
}

#[test_function_nym(priority = 20)]
pub async fn test_country_exit_node(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    const TARGET_COUNTRY: &str = "CH";

    let nym_client =
        helpers_nym::set_enable_two_hop_with_recovery(&test_context.rpc_provider, nym_client, true)
            .await?;
    let (_, nym_client) = helpers_nym::call_nym_with_transport_recovery(
        &test_context.rpc_provider,
        nym_client,
        |mut client| async move {
            let result = client
                .set_exit_point(ExitPoint::Country {
                    two_letter_iso_country_code: TARGET_COUNTRY.to_string(),
                })
                .await;
            (client, result)
        },
    )
    .await
    .context("set_exit_point failed")?;

    log::info!("Connecting with exit country {}...", TARGET_COUNTRY);
    let nym_client =
        helpers_nym::connect_tunnel_with_recovery(&test_context.rpc_provider, nym_client).await?;
    let (_, nym_client) = wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    let addrs = rpc
        .resolve_hostname("ipinfo.io".to_string())
        .await
        .context("DNS resolution of ipinfo.io failed inside VM")?;
    ensure!(
        !addrs.is_empty(),
        "DNS resolution of ipinfo.io returned no addresses"
    );

    let ip_output = rpc
        .exec("curl", ["-s", "--max-time", "15", "https://ipinfo.io/json"])
        .await
        .context("Failed to curl ipinfo.io from VM")?;
    let ip_str = String::from_utf8_lossy(&ip_output.stdout);
    log::info!("ipinfo.io response: {}", ip_str);

    let ip_info: serde_json::Value =
        serde_json::from_str(&ip_str).context("Failed to parse ipinfo.io response as JSON")?;
    let country = ip_info
        .get("country")
        .and_then(|v| v.as_str())
        .context("ipinfo.io response missing 'country' field")?;
    log::info!("Detected exit country: {}", country);
    ensure!(
        country == TARGET_COUNTRY,
        "Expected exit country {}, got {}",
        TARGET_COUNTRY,
        country,
    );

    log::info!("Disconnecting...");
    helpers_nym::disconnect_and_wait(&rpc, nym_client, &test_context.rpc_provider).await?;

    Ok(())
}

#[test_function_nym(priority = 25)]
pub async fn test_reconnect_tunnel(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    let nym_client =
        helpers_nym::set_enable_two_hop_with_recovery(&test_context.rpc_provider, nym_client, true)
            .await?;
    log::info!("Connecting initial tunnel...");
    let nym_client =
        helpers_nym::connect_tunnel_with_recovery(&test_context.rpc_provider, nym_client).await?;
    let (_, nym_client) = wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    log::info!("Reconnecting tunnel...");
    let (_, nym_client) = helpers_nym::call_nym_with_transport_recovery(
        &test_context.rpc_provider,
        nym_client,
        |mut client| async move {
            let result = client.reconnect_tunnel().await;
            (client, result)
        },
    )
    .await
    .context("reconnect_tunnel() failed")?;
    let (_, nym_client) = wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    let addrs = resolve_hostname_with_retry(&rpc, "nym.com", Duration::from_secs(30)).await?;
    log::info!("DNS resolution after reconnect (in VM): {:?}", addrs);

    log::info!("Disconnecting...");
    helpers_nym::disconnect_and_wait(&rpc, nym_client, &test_context.rpc_provider).await?;

    Ok(())
}
