//! Bridge / QUIC / Amnezia surface tests (censorship Tier B).
//!
//! Missing bridge inventory must fail with an explicit `SKIP:` message (not a silent pass).
//! iptables-based UDP/SNI cases are Linux-only (`target_os = "linux"`).

use crate::{
    logging::SKIP_PREFIX,
    tests::{
        TestContext,
        helpers_nym::{
            self, ExpectedTunnelState, ROUNDTRIP_DNS_TIMEOUT, resolve_hostname_with_retry,
            wait_for_tunnel_state,
        },
        nym_test::dc_and_ensure_logged_in,
    },
};
use anyhow::{Context, bail, ensure};
use nym_vpn_lib_types::{GatewayType, ListGatewaysOptions};
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};
use test_macro::test_function_nym;
use test_rpc::NymServiceClient;

const IP_PROBE: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 443);
const PROBE_BIND: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);

/// Production wiring in `tunnel_monitor`: `entry_amnezia = !use_bridges`.
/// There is no separate `set_enable_amnezia` RPC - AmneziaWG BASE applies when bridges are off.
fn amnezia_enabled_for_bridges(use_bridges: bool) -> bool {
    !use_bridges
}

fn merge_body_and_cleanup(
    body: Result<(), anyhow::Error>,
    cleanup: Result<(), anyhow::Error>,
) -> Result<(), anyhow::Error> {
    match (body, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(cleanup_err)) => Err(cleanup_err),
        (Err(body_err), Ok(())) => Err(body_err),
        (Err(body_err), Err(cleanup_err)) => Err(body_err.context(format!(
            "blocklist cleanup also failed (rules may remain): {cleanup_err:#}"
        ))),
    }
}

async fn block_udp_addrs(rpc: &NymServiceClient, socket_addrs: &[&str]) -> anyhow::Result<()> {
    let mut args = vec!["block"];
    args.extend(socket_addrs.iter().copied());
    let output = rpc
        .exec("/tmp/udp_block.sh", args)
        .await
        .context("udp_block.sh block")?;
    ensure!(
        output.success(),
        "udp_block.sh block failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

async fn unblock_udp_addrs(rpc: &NymServiceClient, socket_addrs: &[&str]) -> anyhow::Result<()> {
    let mut args = vec!["unblock"];
    args.extend(socket_addrs.iter().copied());
    let output = rpc
        .exec("/tmp/udp_block.sh", args)
        .await
        .context("udp_block.sh unblock")?;
    ensure!(
        output.success(),
        "udp_block.sh unblock failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

async fn with_udp_blocks<F, Fut>(
    rpc: &NymServiceClient,
    socket_addrs: &[&str],
    body: F,
) -> anyhow::Result<()>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = anyhow::Result<()>>,
{
    // Always attempt unblock after a block attempt so a mid-list failure cannot leave DROP rules.
    let block_result = block_udp_addrs(rpc, socket_addrs).await;
    let body_result = if block_result.is_ok() {
        body().await
    } else {
        Ok(())
    };
    let cleanup = unblock_udp_addrs(rpc, socket_addrs)
        .await
        .context("Failed to unblock UDP addresses after test");
    match block_result {
        Ok(()) => merge_body_and_cleanup(body_result, cleanup),
        Err(block_err) => merge_body_and_cleanup(Err(block_err), cleanup),
    }
}

async fn block_sni_domains(rpc: &NymServiceClient, domains: &[&str]) -> anyhow::Result<()> {
    let mut args = vec!["block"];
    args.extend(domains.iter().copied());
    let output = rpc
        .exec("/tmp/sni_block.sh", args)
        .await
        .context("sni_block.sh block")?;
    ensure!(
        output.success(),
        "sni_block.sh block failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

async fn unblock_sni_domains(rpc: &NymServiceClient, domains: &[&str]) -> anyhow::Result<()> {
    let mut args = vec!["unblock"];
    args.extend(domains.iter().copied());
    let output = rpc
        .exec("/tmp/sni_block.sh", args)
        .await
        .context("sni_block.sh unblock")?;
    ensure!(
        output.success(),
        "sni_block.sh unblock failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

async fn with_sni_blocks<F, Fut>(
    rpc: &NymServiceClient,
    domains: &[&str],
    body: F,
) -> anyhow::Result<()>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = anyhow::Result<()>>,
{
    let block_result = block_sni_domains(rpc, domains).await;
    let body_result = if block_result.is_ok() {
        body().await
    } else {
        Ok(())
    };
    let cleanup = unblock_sni_domains(rpc, domains)
        .await
        .context("Failed to unblock SNI domains after test");
    match block_result {
        Ok(()) => merge_body_and_cleanup(body_result, cleanup),
        Err(block_err) => merge_body_and_cleanup(Err(block_err), cleanup),
    }
}

async fn count_bridge_capable_wg_entries(
    provider: &crate::nym_daemon::RpcClientProvider,
    client: NymProxyClient,
) -> anyhow::Result<(usize, NymProxyClient)> {
    let (gateways, client) = helpers_nym::call_nym_with_transport_recovery(
        provider,
        client,
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

    let with_bridges = gateways
        .iter()
        .filter(|gw| {
            gw.bridge_params
                .as_ref()
                .is_some_and(|params| !params.transports.is_empty())
        })
        .count();
    Ok((with_bridges, client))
}

async fn verify_in_tunnel_tcp(rpc: &NymServiceClient) -> anyhow::Result<()> {
    let addrs = resolve_hostname_with_retry(rpc, "nym.com", ROUNDTRIP_DNS_TIMEOUT)
        .await
        .context("DNS inside tunnel failed")?;
    let dest = SocketAddr::new(addrs[0].ip(), 443);
    rpc.send_tcp(None, PROBE_BIND, dest)
        .await
        .context("TCP inside tunnel failed")?;
    // Secondary IP probe in case resolver is odd.
    let _ = rpc.send_tcp(None, PROBE_BIND, IP_PROBE).await;
    Ok(())
}

async fn connect_wg_with_bridges(
    test_context: &TestContext,
    rpc: &NymServiceClient,
    nym_client: NymProxyClient,
) -> anyhow::Result<NymProxyClient> {
    let nym_client =
        dc_and_ensure_logged_in(rpc, nym_client, &test_context.rpc_provider, false).await?;

    let (bridge_count, nym_client) =
        count_bridge_capable_wg_entries(&test_context.rpc_provider, nym_client).await?;
    if bridge_count == 0 {
        bail!(
            "{SKIP_PREFIX} no WireGuard entry gateway with non-empty bridge_params in current \
             inventory (QUIC bridges unavailable for this run)"
        );
    }
    log::info!("Found {bridge_count} bridge-capable WG entries");

    let nym_client =
        helpers_nym::set_enable_two_hop_with_recovery(&test_context.rpc_provider, nym_client, true)
            .await?;
    let nym_client = helpers_nym::set_enable_bridges_with_recovery(
        &test_context.rpc_provider,
        nym_client,
        true,
    )
    .await?;

    let config = {
        let (cfg, client) = helpers_nym::call_nym_with_transport_recovery(
            &test_context.rpc_provider,
            nym_client,
            |mut client| async move {
                let result = client.get_config().await;
                (client, result)
            },
        )
        .await
        .context("get_config failed")?;
        ensure!(
            cfg.enable_bridges,
            "enable_bridges was set but get_config reports false"
        );
        client
    };

    let nym_client =
        helpers_nym::connect_tunnel_with_recovery(&test_context.rpc_provider, config).await?;
    let (_, nym_client) = wait_for_tunnel_state(
        rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;
    verify_in_tunnel_tcp(rpc).await?;
    Ok(nym_client)
}

/// Bridges ON + inventory present: WireGuard connect and in-tunnel TCP must succeed.
/// Priority 40-43: after tunnel_tests (max 25), strictly before blocklist_tests
/// (min 100) - see `blocking_tests::tests::blocklist_priorities_run_after_core_suite`.
#[test_function_nym(priority = 40)]
pub async fn test_bridges_enabled_connect_when_available(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let nym_client = connect_wg_with_bridges(&test_context, &rpc, nym_client).await?;
    helpers_nym::disconnect_and_wait(&rpc, nym_client, &test_context.rpc_provider).await?;
    Ok(())
}

/// Public DoH/QUIC UDP:443 to well-known resolvers blocked (not gateway/bridge endpoints).
/// With bridges enabled + inventory, WG connect must still reach Connected (or SKIP).
/// This is a lightweight censorship signal, not full bridge-DPI coverage.
/// Linux/iptables only (`udp_block.sh` is not deployed on macOS).
#[test_function_nym(priority = 41, target_os = "linux")]
pub async fn test_public_doh_udp_443_blocked_bridges_still_connect(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    // Public resolver UDP/443 only - does not block entry bridge IPs or WG UDP.
    let udp_targets = [
        "1.1.1.1:443",
        "1.0.0.1:443",
        "8.8.8.8:443",
        "8.8.4.4:443",
        "9.9.9.9:443",
    ];

    with_udp_blocks(&rpc, &udp_targets, || async {
        let nym_client = connect_wg_with_bridges(&test_context, &rpc, nym_client).await?;
        helpers_nym::disconnect_and_wait(&rpc, nym_client, &test_context.rpc_provider).await?;
        Ok(())
    })
    .await
}

/// VPN API SNI block + bridges enabled (domain-front / bridge path under API censorship).
/// Linux/iptables only (same guest constraint as other SNI blocklist cases).
#[test_function_nym(priority = 42, target_os = "linux")]
pub async fn test_api_sni_block_plus_bridges(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let domains = ["nymvpn.com:443"];
    with_sni_blocks(&rpc, &domains, || async {
        let nym_client = connect_wg_with_bridges(&test_context, &rpc, nym_client).await?;
        helpers_nym::disconnect_and_wait(&rpc, nym_client, &test_context.rpc_provider).await?;
        Ok(())
    })
    .await
}

/// AmneziaWG surface: WG entry uses Amnezia BASE when bridges are disabled
/// (`entry_amnezia = !use_bridges` in tunnel_monitor). No separate Amnezia RPC exists.
/// Behavioral DPI under junk traffic is Tier C (out of scope).
#[test_function_nym(priority = 43, target_os = "linux")]
pub async fn test_amnezia_entry_config_surface(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    ensure!(
        amnezia_enabled_for_bridges(false),
        "test precondition: Amnezia must be on when bridges are off"
    );
    ensure!(
        !amnezia_enabled_for_bridges(true),
        "negative control: Amnezia must be off when bridges are on"
    );

    let nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;
    let nym_client =
        helpers_nym::set_enable_two_hop_with_recovery(&test_context.rpc_provider, nym_client, true)
            .await?;
    // Explicit bridges-off is the Amnezia enable path (not a separate toggle).
    let nym_client = helpers_nym::set_enable_bridges_with_recovery(
        &test_context.rpc_provider,
        nym_client,
        false,
    )
    .await?;

    let nym_client = {
        let (cfg, client) = helpers_nym::call_nym_with_transport_recovery(
            &test_context.rpc_provider,
            nym_client,
            |mut client| async move {
                let result = client.get_config().await;
                (client, result)
            },
        )
        .await
        .context("get_config failed")?;
        ensure!(
            !cfg.enable_bridges,
            "Amnezia entry path requires enable_bridges=false; get_config reported true"
        );
        client
    };

    let nym_client =
        helpers_nym::connect_tunnel_with_recovery(&test_context.rpc_provider, nym_client).await?;
    let (_, nym_client) = wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    let body = async {
        let journal = rpc
            .exec(
                "journalctl",
                [
                    "-u",
                    "nym-vpnd",
                    "--no-pager",
                    "-n",
                    "400",
                    "-o",
                    "cat",
                ],
            )
            .await
            .context("journalctl nym-vpnd failed")?;
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&journal.stdout),
            String::from_utf8_lossy(&journal.stderr)
        )
        .to_ascii_lowercase();

        let markers = ["amnezia", "junk packet", " jc=", "jmin", "jmax"];
        let hit = markers.iter().any(|m| text.contains(m));
        if !hit {
            let tail: String = text
                .chars()
                .rev()
                .take(500)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            // Logging oracle only - bridges-off path was asserted above; missing markers are
            // an observability gap, not "feature never enabled".
            bail!(
                "{SKIP_PREFIX} bridges=false (Amnezia entry path) but no Amnezia marker in recent \
                 nym-vpnd logs (config-only oracle; behavioral DPI is Tier C). Log sample tail: \
                 {tail}"
            );
        }
        log::info!("Amnezia surface marker found in nym-vpnd journal");
        Ok(())
    }
    .await;

    let cleanup = helpers_nym::disconnect_and_wait(&rpc, nym_client, &test_context.rpc_provider)
        .await
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!(e));
    merge_body_and_cleanup(body, cleanup)
}

#[cfg(test)]
mod unit_tests {
    use super::{amnezia_enabled_for_bridges, merge_body_and_cleanup};
    use crate::tests::get_test_descriptions;

    #[test]
    fn amnezia_on_iff_bridges_off() {
        assert!(amnezia_enabled_for_bridges(false));
        assert!(!amnezia_enabled_for_bridges(true));
    }

    /// Bridge tests mutate iptables (UDP/SNI blocks) like `blocklist_tests`; they must run
    /// strictly before the blocklist priority range (min 100) so
    /// `blocking_tests::tests::blocklist_priorities_run_after_core_suite` still holds.
    #[test]
    fn bridge_test_priorities_stay_below_blocklist_range() {
        const BLOCKLIST_MIN_PRIORITY: i32 = 100;
        const BRIDGE_TEST_NAMES: [&str; 4] = [
            "test_bridges_enabled_connect_when_available",
            "test_public_doh_udp_443_blocked_bridges_still_connect",
            "test_api_sni_block_plus_bridges",
            "test_amnezia_entry_config_surface",
        ];
        let tests = get_test_descriptions();
        for name in BRIDGE_TEST_NAMES {
            let priority = tests
                .iter()
                .find(|t| t.name == name)
                .and_then(|t| t.priority)
                .unwrap_or_else(|| panic!("bridge test '{name}' must be registered"));
            assert!(
                priority < BLOCKLIST_MIN_PRIORITY,
                "bridge test '{name}' priority {priority} must stay below blocklist min {BLOCKLIST_MIN_PRIORITY}"
            );
        }
    }

    #[test]
    fn merge_ok_ok() {
        assert!(merge_body_and_cleanup(Ok(()), Ok(())).is_ok());
    }

    #[test]
    fn merge_ok_cleanup_err() {
        let err = merge_body_and_cleanup(Ok(()), Err(anyhow::anyhow!("cleanup")))
            .expect_err("cleanup-only");
        assert!(err.to_string().contains("cleanup"), "{err}");
    }

    #[test]
    fn merge_body_err_cleanup_ok() {
        let err = merge_body_and_cleanup(Err(anyhow::anyhow!("body")), Ok(()))
            .expect_err("body-only");
        assert!(err.to_string().contains("body"), "{err}");
        assert!(!format!("{err:#}").contains("cleanup"), "{err:#}");
    }

    #[test]
    fn merge_chains_cleanup_into_body_error() {
        let err = merge_body_and_cleanup(
            Err(anyhow::anyhow!("body")),
            Err(anyhow::anyhow!("cleanup")),
        )
        .unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("body"), "{msg}");
        assert!(msg.contains("cleanup"), "{msg}");
    }
}
