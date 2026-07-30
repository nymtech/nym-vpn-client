//! User reliability journeys: LAN policy, offline detection, daemon restart, sticky settings.
//!
//! Priorities sit after core tunnel tests and before censorship blocklist cases.

use crate::{
    logging::SKIP_PREFIX,
    nym_daemon::RpcClientProvider,
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
use nym_vpn_lib_types::ExitPoint;
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use test_macro::test_function_nym;
use test_rpc::{
    NymServiceClient,
    nym_daemon::{ObservedTunnelState, ObservedTunnelType},
};

const PUBLIC_TCP_PROBE: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 443);
const PROBE_BIND: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
const OFFLINE_WAIT: Duration = Duration::from_secs(45);
const DAEMON_READY_WAIT: Duration = Duration::from_secs(60);

fn merge_body_and_cleanup(
    body: Result<(), anyhow::Error>,
    cleanup: Result<(), anyhow::Error>,
) -> Result<(), anyhow::Error> {
    match (body, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(cleanup_err)) => Err(cleanup_err),
        (Err(body_err), Ok(())) => Err(body_err),
        (Err(body_err), Err(cleanup_err)) => Err(body_err.context(format!(
            "cleanup also failed (guest may be left degraded): {cleanup_err:#}"
        ))),
    }
}

async fn disconnect_cleanup(
    rpc: &NymServiceClient,
    nym_client: NymProxyClient,
    provider: &RpcClientProvider,
) -> Result<(), anyhow::Error> {
    helpers_nym::disconnect_and_wait(rpc, nym_client, provider)
        .await
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!(e))
}

async fn connect_and_wait_connected(
    rpc: &NymServiceClient,
    provider: &RpcClientProvider,
    nym_client: NymProxyClient,
) -> anyhow::Result<(ObservedTunnelState, NymProxyClient)> {
    let nym_client = helpers_nym::connect_tunnel_with_recovery(provider, nym_client).await?;
    wait_for_tunnel_state(rpc, nym_client, provider, ExpectedTunnelState::Connected)
        .await
        .map_err(Into::into)
}

/// LAN / link-local ranges that `allow_lan=false` is expected to fence off.
fn is_lan_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_private() || v4.is_link_local(),
        IpAddr::V6(v6) => {
            let segments = v6.segments();
            // Unique local fc00::/7 and link-local fe80::/10
            (segments[0] & 0xfe00) == 0xfc00 || (segments[0] & 0xffc0) == 0xfe80
        }
    }
}

async fn get_vm_lan_nameservers(rpc: &NymServiceClient) -> anyhow::Result<Vec<IpAddr>> {
    let output = rpc
        .exec("resolvectl", ["dns"])
        .await
        .context("resolvectl dns failed")?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut nameservers = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        // Prefer non-tun links for "prior LAN resolvers".
        if line.contains("(tun") {
            continue;
        }
        if let Some((_, servers)) = line.split_once(':') {
            for server in servers.split_whitespace() {
                if let Ok(ip) = server.parse::<IpAddr>()
                    && is_lan_ip(ip)
                {
                    nameservers.push(ip);
                }
            }
        }
    }
    Ok(nameservers)
}

async fn tcp_reachable(rpc: &NymServiceClient, dest: SocketAddr) -> bool {
    rpc.send_tcp(None, PROBE_BIND, dest).await.is_ok()
}

async fn assert_tunnel_dns_and_tcp(rpc: &NymServiceClient) -> anyhow::Result<()> {
    let addrs = resolve_hostname_with_retry(rpc, "nym.com", ROUNDTRIP_DNS_TIMEOUT)
        .await
        .context("in-tunnel DNS failed")?;
    let dest = SocketAddr::new(addrs[0].ip(), 443);
    rpc.send_tcp(None, PROBE_BIND, dest)
        .await
        .context("in-tunnel TCP failed")?;
    Ok(())
}

async fn detect_default_iface(rpc: &NymServiceClient) -> anyhow::Result<String> {
    let output = rpc
        .exec("ip", ["-o", "route", "show", "default"])
        .await
        .context("ip route show default")?;
    ensure!(
        output.success(),
        "ip route show default failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let line = String::from_utf8_lossy(&output.stdout);
    let iface = line
        .split_whitespace()
        .skip_while(|tok| *tok != "dev")
        .nth(1)
        .context("could not parse default route interface")?;
    Ok(iface.to_string())
}

async fn wait_for_observed_offline(
    rpc: &NymServiceClient,
    timeout: Duration,
) -> anyhow::Result<()> {
    let deadline = tokio::time::Instant::now() + timeout;
    let mut last = None;
    loop {
        if tokio::time::Instant::now() >= deadline {
            bail!("timed out waiting for Offline tunnel state; last={last:?}");
        }
        match rpc.get_observed_tunnel_state().await {
            Ok(ObservedTunnelState::Offline) => return Ok(()),
            Ok(state) => last = Some(format!("{state:?}")),
            Err(err) => last = Some(format!("observe error: {err}")),
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn bring_iface_down(rpc: &NymServiceClient, iface: &str) -> anyhow::Result<()> {
    let output = rpc
        .exec("ip", ["link", "set", "dev", iface, "down"])
        .await
        .context("ip link set down")?;
    ensure!(
        output.success(),
        "failed to bring {iface} down: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

async fn bring_iface_up(rpc: &NymServiceClient, iface: &str) -> anyhow::Result<()> {
    let up = rpc
        .exec("ip", ["link", "set", "dev", iface, "up"])
        .await
        .context("ip link set up")?;
    ensure!(
        up.success(),
        "failed to bring {iface} up: {}",
        String::from_utf8_lossy(&up.stderr)
    );
    let _ = rpc.exec("dhclient", ["-1", "-v", iface]).await;
    let _ = rpc.exec("networkctl", ["renew", iface]).await;
    Ok(())
}

async fn wait_daemon_rpc_ready(
    provider: &crate::nym_daemon::RpcClientProvider,
    timeout: Duration,
) -> anyhow::Result<NymProxyClient> {
    let deadline = tokio::time::Instant::now() + timeout;
    let mut last_err = None;
    while tokio::time::Instant::now() < deadline {
        match provider.recover_client_nym().await {
            Ok(client) => match helpers_nym::ensure_daemon_rpc_responsive(provider, client).await {
                Ok(client) => return Ok(client),
                Err(err) => last_err = Some(err.to_string()),
            },
            Err(err) => last_err = Some(err.to_string()),
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    bail!(
        "daemon RPC not ready after {}s; last={:?}",
        timeout.as_secs(),
        last_err
    )
}

/// User disables Local network access: prior *LAN* resolvers must not answer on TCP while connected.
#[test_function_nym(priority = 30)]
pub async fn test_allow_lan_off_blocks_prior_resolver_tcp(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    let lan_nameservers = get_vm_lan_nameservers(&rpc).await?;
    if lan_nameservers.is_empty() {
        bail!(
            "{SKIP_PREFIX} VM has no LAN/private resolvectl nameservers to probe under \
             allow_lan=false (public resolvers are not a LAN-policy oracle)"
        );
    }

    let nym_client =
        helpers_nym::set_allow_lan_with_recovery(&test_context.rpc_provider, nym_client, false)
            .await?;
    let nym_client =
        helpers_nym::set_enable_two_hop_with_recovery(&test_context.rpc_provider, nym_client, true)
            .await?;
    let (_, nym_client) =
        connect_and_wait_connected(&rpc, &test_context.rpc_provider, nym_client).await?;

    let body = async {
        let mut still_open = Vec::new();
        for ip in &lan_nameservers {
            let dest = SocketAddr::new(*ip, 53);
            if tcp_reachable(&rpc, dest).await {
                still_open.push(dest);
            }
        }
        ensure!(
            still_open.is_empty(),
            "allow_lan=false but LAN resolvers still reachable on TCP/53: {still_open:?}"
        );
        assert_tunnel_dns_and_tcp(&rpc).await?;
        Ok(())
    }
    .await;

    let cleanup = disconnect_cleanup(&rpc, nym_client, &test_context.rpc_provider).await;
    merge_body_and_cleanup(body, cleanup)
}

/// Taking the default uplink down must surface ObservedTunnelState::Offline (daemon detection).
/// Does not assert clearnet TCP failure — that is "no route", not FirewallPolicy::Blocked.
#[test_function_nym(priority = 31)]
pub async fn test_uplink_down_reports_offline(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;
    let _ = helpers_nym::set_allow_lan_with_recovery(&test_context.rpc_provider, nym_client, false)
        .await?;

    ensure!(
        tcp_reachable(&rpc, PUBLIC_TCP_PROBE).await,
        "precondition: public TCP to {PUBLIC_TCP_PROBE} should work while online"
    );

    let iface = detect_default_iface(&rpc).await?;
    log::info!("Bringing uplink {iface} down to force Offline");
    bring_iface_down(&rpc, &iface).await?;
    let body = async {
        wait_for_observed_offline(&rpc, OFFLINE_WAIT).await?;
        Ok(())
    }
    .await;
    let cleanup = bring_iface_up(&rpc, &iface).await;
    merge_body_and_cleanup(body, cleanup)?;

    wait_daemon_rpc_ready(&test_context.rpc_provider, Duration::from_secs(30))
        .await
        .context("daemon RPC not ready after uplink restore")?;

    ensure!(
        tcp_reachable(&rpc, PUBLIC_TCP_PROBE).await,
        "public TCP should work again after uplink restore"
    );

    Ok(())
}

/// Daemon crash/restart: recover RPC, reconnect, DNS/TCP still works.
#[test_function_nym(priority = 32)]
pub async fn test_daemon_restart_then_reconnect(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;
    let nym_client =
        helpers_nym::set_enable_two_hop_with_recovery(&test_context.rpc_provider, nym_client, true)
            .await?;
    let (_, nym_client) =
        connect_and_wait_connected(&rpc, &test_context.rpc_provider, nym_client).await?;

    drop(nym_client);
    log::info!("Restarting nym-vpnd via systemd...");
    rpc.restart_nymvpn_daemon()
        .await
        .context("restart_nymvpn_daemon failed")?;

    let mut nym_client = wait_daemon_rpc_ready(&test_context.rpc_provider, DAEMON_READY_WAIT)
        .await
        .context("daemon did not become ready after restart")?;

    nym_client = helpers_nym::login_idempotent(&rpc, nym_client, &test_context.rpc_provider)
        .await
        .context("re-login after daemon restart failed")?;
    nym_client =
        helpers_nym::finish_prep_with_allow_lan(&test_context.rpc_provider, nym_client).await?;
    nym_client =
        helpers_nym::set_enable_two_hop_with_recovery(&test_context.rpc_provider, nym_client, true)
            .await?;
    let (_, nym_client) =
        connect_and_wait_connected(&rpc, &test_context.rpc_provider, nym_client).await?;

    let body = assert_tunnel_dns_and_tcp(&rpc).await;
    let cleanup = disconnect_cleanup(&rpc, nym_client, &test_context.rpc_provider).await;
    merge_body_and_cleanup(body, cleanup)
}

/// Fast ↔ Mixnet toggle must select the matching observed tunnel type across reconnects.
#[test_function_nym(priority = 33)]
pub async fn test_two_hop_toggle_survives_reconnect(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let mut nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    for (two_hop, expected) in [
        (true, ObservedTunnelType::Wireguard),
        (false, ObservedTunnelType::Mixnet),
        (true, ObservedTunnelType::Wireguard),
    ] {
        nym_client = helpers_nym::set_enable_two_hop_with_recovery(
            &test_context.rpc_provider,
            nym_client,
            two_hop,
        )
        .await?;
        let (state, client) =
            connect_and_wait_connected(&rpc, &test_context.rpc_provider, nym_client).await?;
        nym_client = client;
        match state {
            ObservedTunnelState::Connected { tunnel_type } if tunnel_type == expected => {
                log::info!("Got expected tunnel type {expected:?} (two_hop={two_hop})");
            }
            other => bail!("two_hop={two_hop}: expected Connected {expected:?}, got {other:?}"),
        }
        nym_client =
            helpers_nym::disconnect_and_wait(&rpc, nym_client, &test_context.rpc_provider).await?;
    }

    Ok(())
}

/// Country exit selection must stick across reconnect_tunnel.
#[test_function_nym(priority = 34)]
pub async fn test_exit_country_persists_across_reconnect(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    const TARGET_COUNTRY: &str = "CH";

    let nym_client =
        dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;
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

    let (_, nym_client) =
        connect_and_wait_connected(&rpc, &test_context.rpc_provider, nym_client).await?;

    let (_, nym_client) = helpers_nym::call_nym_with_transport_recovery(
        &test_context.rpc_provider,
        nym_client,
        |mut client| async move {
            let result = client.reconnect_tunnel().await;
            (client, result)
        },
    )
    .await
    .context("reconnect_tunnel failed")?;
    let nym_client = {
        let (_, client) = wait_for_tunnel_state(
            &rpc,
            nym_client,
            &test_context.rpc_provider,
            ExpectedTunnelState::Connected,
        )
        .await?;
        client
    };

    let body = async {
        let _ = resolve_hostname_with_retry(&rpc, "ipinfo.io", ROUNDTRIP_DNS_TIMEOUT).await?;
        let ip_output = rpc
            .exec("curl", ["-s", "--max-time", "15", "https://ipinfo.io/json"])
            .await
            .context("curl ipinfo.io failed")?;
        let ip_info: serde_json::Value =
            serde_json::from_slice(&ip_output.stdout).context("parse ipinfo.io JSON")?;
        let country = ip_info
            .get("country")
            .and_then(|v| v.as_str())
            .context("ipinfo.io missing country")?;
        ensure!(
            country == TARGET_COUNTRY,
            "after reconnect expected country {TARGET_COUNTRY}, got {country}"
        );
        Ok(())
    }
    .await;

    let cleanup = disconnect_cleanup(&rpc, nym_client, &test_context.rpc_provider).await;
    merge_body_and_cleanup(body, cleanup)
}

#[cfg(test)]
mod unit_tests {
    use super::{is_lan_ip, merge_body_and_cleanup};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn merge_prefers_body_error_but_surfaces_cleanup() {
        let err = merge_body_and_cleanup(
            Err(anyhow::anyhow!("body failed")),
            Err(anyhow::anyhow!("cleanup failed")),
        )
        .expect_err("combined failure");
        let msg = format!("{err:#}");
        assert!(msg.contains("body failed"), "{msg}");
        assert!(msg.contains("cleanup failed"), "{msg}");
    }

    #[test]
    fn merge_cleanup_only_surfaces() {
        let err = merge_body_and_cleanup(Ok(()), Err(anyhow::anyhow!("cleanup failed")))
            .expect_err("cleanup-only");
        assert!(err.to_string().contains("cleanup failed"));
    }

    #[test]
    fn lan_ip_classifier_accepts_private_rejects_public() {
        assert!(is_lan_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
        assert!(is_lan_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(is_lan_ip(IpAddr::V4(Ipv4Addr::new(172, 29, 1, 1))));
        assert!(!is_lan_ip(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))));
        assert!(!is_lan_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
        assert!(is_lan_ip(IpAddr::V6(Ipv6Addr::new(
            0xfd00, 0, 0, 0, 0, 0, 0, 1
        ))));
        assert!(!is_lan_ip(IpAddr::V6(Ipv6Addr::new(
            0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111
        ))));
    }
}
