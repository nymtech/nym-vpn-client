// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::tests::{
    TestContext,
    helpers_nym::{ExpectedTunnelState, resolve_hostname_with_retry, wait_for_tunnel_state},
    nym_test::dc_and_ensure_logged_in,
};
use anyhow::{Context, bail, ensure};
use nym_vpn_lib_types::{ExitPoint, GatewayType, ListGatewaysOptions};
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use std::{net::SocketAddr, time::Duration};
use test_macro::test_function_nym;
use test_rpc::{
    NymServiceClient,
    nym_daemon::{ObservedTunnelState, ObservedTunnelType},
};

/// Read nameservers from /etc/resolv.conf inside the guest VM via tarpc.
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
    let mut nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    let mixnet_gateways = nym_client
        .list_gateways(ListGatewaysOptions {
            gw_type: GatewayType::MixnetEntry,
            user_agent: None,
        })
        .await
        .context("list_gateways(MixnetEntry) failed")?;
    log::info!("Mixnet entry gateways: {}", mixnet_gateways.len());
    ensure!(
        !mixnet_gateways.is_empty(),
        "Expected at least one MixnetEntry gateway"
    );

    let wg_gateways = nym_client
        .list_gateways(ListGatewaysOptions {
            gw_type: GatewayType::Wg,
            user_agent: None,
        })
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
    let mut nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    // Get account summary
    let summary = nym_client
        .get_account_summary()
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

    // Get usage
    let usages = nym_client
        .get_account_usage()
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
    let mut nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    // Enable two-hop (WireGuard mode)
    nym_client.set_enable_two_hop(true).await?;

    // Connect
    log::info!("Connecting WireGuard tunnel...");
    nym_client.connect_tunnel().await?;
    let (state, mut nym_client) = wait_for_tunnel_state(
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

    // Disconnect
    log::info!("Disconnecting...");
    nym_client.disconnect_tunnel().await?;
    wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Disconnected,
    )
    .await?;

    Ok(())
}

#[test_function_nym(priority = 11)]
pub async fn test_mixnet_connect_disconnect(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let mut nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    // Disable two-hop (Mixnet mode)
    nym_client.set_enable_two_hop(false).await?;

    // Connect with longer timeout for mixnet
    log::info!("Connecting Mixnet tunnel (this may take longer)...");
    nym_client.connect_tunnel().await?;
    let (state, mut nym_client) = wait_for_tunnel_state(
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

    // Disconnect
    log::info!("Disconnecting...");
    nym_client.disconnect_tunnel().await?;
    wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Disconnected,
    )
    .await?;

    Ok(())
}

#[test_function_nym(priority = 15)]
pub async fn test_dns_leak(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let mut nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    // pre-VPN DNS servers from guest VM's resolv.conf
    let pre_vpn_nameservers = get_vm_nameservers(&rpc).await?;
    log::info!("Pre-VPN nameservers (in VM): {:?}", pre_vpn_nameservers);
    ensure!(
        !pre_vpn_nameservers.is_empty(),
        "VM should have at least one nameserver"
    );

    // connect
    nym_client.set_enable_two_hop(true).await?;
    log::info!("Connecting tunnel for DNS leak test...");
    nym_client.connect_tunnel().await?;
    let (_, mut nym_client) = wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    // check resolv.conf after connecting
    let post_vpn_nameservers = get_vm_nameservers(&rpc).await?;
    log::info!("Post-VPN nameservers (in VM): {:?}", post_vpn_nameservers);

    let leaked_nameservers: Vec<_> = post_vpn_nameservers
        .iter()
        .filter(|ns| pre_vpn_nameservers.contains(ns))
        .collect();
    ensure!(
        leaked_nameservers.is_empty(),
        "DNS LEAK: post-VPN resolv.conf still contains pre-VPN nameservers: {:?}",
        leaked_nameservers,
    );

    // DNS resolution works inside VM through the tunnel
    let addrs = rpc
        .resolve_hostname("nym.com".to_string())
        .await
        .context("DNS resolution failed inside VM while VPN is connected")?;
    log::info!("Resolved nym.com inside VM: {:?}", addrs);
    ensure!(
        !addrs.is_empty(),
        "DNS resolution inside VM returned no addresses while connected"
    );

    // verify TCP connectivity to confirm DNS gave a real address
    let dest = SocketAddr::new(addrs[0].ip(), 443);
    rpc.send_tcp(None, "0.0.0.0:0".parse().unwrap(), dest)
        .await
        .context("TCP connectivity check failed inside VM while VPN is connected")?;
    log::info!("TCP connectivity to {} verified inside VM", dest);

    // triggers DNS lookups to bashws-controlled subdomains, then queries their
    // API to see which resolver IPs actually made the queries. If any match our
    // pre-VPN nameservers, could indicate a DNS leak outside the tunnel
    let bash_ws_result = check_dns_leak_bash_ws(&rpc, &pre_vpn_nameservers).await;
    match bash_ws_result {
        Ok(()) => log::info!("bash.ws DNS leak check passed"),
        Err(e) => {
            // If bash.ws detected a real leak, fail hard.
            // If bash.ws itself is unreachable/broken, log a warning but don't
            // fail the test since above checks alraedy passed
            let msg = format!("{e:#}");
            if msg.contains("DNS LEAK") {
                bail!("{e}");
            }
            log::warn!("bash.ws check could not complete (service may be down): {e}");
        }
    }

    log::info!("Disconnecting...");
    nym_client.disconnect_tunnel().await?;
    wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Disconnected,
    )
    .await?;

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
    let mut nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    const TARGET_COUNTRY: &str = "CH";

    nym_client.set_enable_two_hop(true).await?;
    nym_client
        .set_exit_point(ExitPoint::Country {
            two_letter_iso_country_code: TARGET_COUNTRY.to_string(),
        })
        .await
        .context("set_exit_point failed")?;

    log::info!("Connecting with exit country {}...", TARGET_COUNTRY);
    nym_client.connect_tunnel().await?;
    let (_, mut nym_client) = wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    // verify DNS works inside VM before making HTTP request
    let addrs = rpc
        .resolve_hostname("ipinfo.io".to_string())
        .await
        .context("DNS resolution of ipinfo.io failed inside VM")?;
    ensure!(
        !addrs.is_empty(),
        "DNS resolution of ipinfo.io returned no addresses"
    );

    // check exit country via ipinfo.io (requires curl in VM image)
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
    nym_client.disconnect_tunnel().await?;
    wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Disconnected,
    )
    .await?;

    Ok(())
}

#[test_function_nym(priority = 25)]
pub async fn test_reconnect_tunnel(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let mut nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    // connect with wg
    nym_client.set_enable_two_hop(true).await?;
    log::info!("Connecting initial tunnel...");
    nym_client.connect_tunnel().await?;
    let (_, mut nym_client) = wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    log::info!("Reconnecting tunnel...");
    nym_client
        .reconnect_tunnel()
        .await
        .context("reconnect_tunnel() failed")?;
    let (_, mut nym_client) = wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    // verify connectivity
    // retry DNS because the data plane may not be ready
    // immediately after the tunnel state transitions to Connected
    let addrs = resolve_hostname_with_retry(&rpc, "nym.com", Duration::from_secs(30)).await?;
    log::info!("DNS resolution after reconnect (in VM): {:?}", addrs);

    log::info!("Disconnecting...");
    nym_client.disconnect_tunnel().await?;
    wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Disconnected,
    )
    .await?;

    Ok(())
}
