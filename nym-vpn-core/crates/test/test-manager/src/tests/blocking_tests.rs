//! Censorship scenario tests
//!
//! Priorities are intentionally high so these run after the core suite. Block rules can
//! poison guest networking / serial RPC if cleanup is skipped on failure.

use crate::tests::{
    TestContext,
    helpers_nym::{self, ROUNDTRIP_DNS_TIMEOUT, resolve_hostname_with_retry},
    nym_test::dc_and_ensure_logged_in,
};
use anyhow::Context;
use helpers_nym::ExpectedTunnelState;
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};
use test_macro::test_function_nym;
use test_rpc::NymServiceClient;

/// Prefer the body error; always surface cleanup failure (chained) so leftover
/// iptables/SNI rules are not silent when both paths fail.
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

/// After delayed-block setup fails, always attempt DNS unblock and chain that
/// failure into the returned error when unblock also fails.
fn merge_delayed_setup_failure(
    delayed_err: anyhow::Error,
    dns_unblock: Result<(), anyhow::Error>,
) -> anyhow::Error {
    match dns_unblock {
        Ok(()) => delayed_err,
        Err(cleanup_err) => delayed_err.context(format!(
            "DNS unblock after delayed block setup failure also failed (rules may remain): {cleanup_err:#}"
        )),
    }
}

/// get socket addresses for default dns nameservers used by the vpn client
fn get_default_nameserver_sockaddrs() -> Vec<String> {
    let dns_nameservers = [
        "1.1.1.1",
        "1.0.0.1",
        "2606:4700:4700::1111",
        "2606:4700:4700::1001",
        "9.9.9.9",
        "149.112.112.112",
        "2620:00fe::00fe",
        "2620:00fe::00fe:0009",
    ];
    let dns_proto_ports = [443, 853];

    dns_nameservers
        .iter()
        .flat_map(|&addr| {
            let formatted_addr = if addr.contains(':') {
                format!("[{}]", addr) // IPv6 needs brackets
            } else {
                addr.to_string() // IPv4
            };
            dns_proto_ports
                .iter()
                .map(move |&port| format!("{}:{}", formatted_addr, port))
        })
        .collect()
}

/// The in-tunnel resolver forwards to the default nameservers, so a test that blocks
/// them by IP cannot also assert that hostname resolution works.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TunnelVerification {
    ResolveHostnames,
    ReachIpOnly,
}

/// Google Public DNS answers on 443 and is deliberately absent from
/// [`get_default_nameserver_sockaddrs`], so it stays reachable while those are blocked.
const IP_PROBE_SOCKET_ADDRS: [SocketAddr; 2] = [
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 443),
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 4, 4)), 443),
];

const IP_PROBE_BIND_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);

fn verification_for_blocked_addrs(blocked_socket_addrs: &[String]) -> TunnelVerification {
    let default_nameservers = get_default_nameserver_sockaddrs();
    if blocked_socket_addrs
        .iter()
        .any(|addr| default_nameservers.contains(addr))
    {
        TunnelVerification::ReachIpOnly
    } else {
        TunnelVerification::ResolveHostnames
    }
}

/// Verify that the VPN tunnel carries traffic, by hostname or by IP depending on
/// whether the test blocked the resolvers the tunnel itself depends on.
async fn verify_tunnel_connectivity(
    rpc: &NymServiceClient,
    verification: TunnelVerification,
) -> anyhow::Result<()> {
    match verification {
        TunnelVerification::ResolveHostnames => {
            for host in ["nym.com", "google.com"] {
                log::info!("Resolving {} inside VM via VPN tunnel...", host);
                let addrs = resolve_hostname_with_retry(rpc, host, ROUNDTRIP_DNS_TIMEOUT)
                    .await
                    .context(format!("DNS resolution failed for {host} inside VM"))?;
                log::info!("Resolved {} to {:?}", host, addrs);
            }
            Ok(())
        }
        TunnelVerification::ReachIpOnly => {
            let mut last_error = None;
            for dest in IP_PROBE_SOCKET_ADDRS {
                log::info!("Connecting to {} inside VM via VPN tunnel...", dest);
                match rpc.send_tcp(None, IP_PROBE_BIND_ADDR, dest).await {
                    Ok(()) => {
                        log::info!("TCP connectivity to {} verified inside VM", dest);
                        return Ok(());
                    }
                    Err(error) => {
                        log::warn!("TCP connectivity to {dest} failed inside VM: {error}");
                        last_error =
                            Some(anyhow::Error::new(error).context(format!("probe {dest}")));
                    }
                }
            }
            Err(last_error
                .unwrap_or_else(|| anyhow::anyhow!("no probe addresses configured"))
                .context("TCP connectivity failed inside VM for every probe address"))
        }
    }
}

async fn with_socket_blocks<F, Fut>(
    rpc: &NymServiceClient,
    socket_addrs: &[String],
    body: F,
) -> anyhow::Result<()>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = anyhow::Result<()>>,
{
    block_socket_addrs(rpc, socket_addrs).await?;
    let body_result = body().await;
    let cleanup = unblock_socket_addrs(rpc, socket_addrs)
        .await
        .context("Failed to unblock socket addresses after test");
    merge_body_and_cleanup(body_result, cleanup)
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
    block_server_name_indicators(rpc, domains).await?;
    let body_result = body().await;
    let cleanup = unblock_server_name_indicators(rpc, domains)
        .await
        .context("Failed to unblock SNI domains after test");
    merge_body_and_cleanup(body_result, cleanup)
}

async fn with_delayed_and_dns_blocks<F, Fut>(
    rpc: &NymServiceClient,
    dns_addrs: &[String],
    delayed_addrs: &[&str],
    body: F,
) -> anyhow::Result<()>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = anyhow::Result<()>>,
{
    block_socket_addrs(rpc, dns_addrs).await?;
    let delayed_block = block_socket_addrs_delayed(rpc, delayed_addrs).await;
    if let Err(e) = delayed_block {
        let dns_unblock = unblock_socket_addrs(rpc, dns_addrs)
            .await
            .context("Failed to unblock DNS after delayed block setup failure");
        return Err(merge_delayed_setup_failure(e, dns_unblock));
    }

    let body_result = body().await;

    let delayed_cleanup = unblock_socket_addrs_delayed(rpc, delayed_addrs)
        .await
        .context("Failed to unblock delayed socket addresses after test");
    let dns_cleanup = unblock_socket_addrs(rpc, dns_addrs)
        .await
        .context("Failed to unblock DNS socket addresses after test");
    let cleanup = match (delayed_cleanup, dns_cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), Ok(())) | (Ok(()), Err(e)) => Err(e),
        (Err(e), Err(dns_err)) => {
            log::warn!("DNS unblock after delayed unblock failure also failed: {dns_err:#}");
            Err(e)
        }
    };
    merge_body_and_cleanup(body_result, cleanup)
}

async fn connect_verify_disconnect(
    test_context: &TestContext,
    rpc: &NymServiceClient,
    nym_client: NymProxyClient,
    verification: TunnelVerification,
) -> anyhow::Result<()> {
    let nym_client =
        dc_and_ensure_logged_in(rpc, nym_client, &test_context.rpc_provider, false).await?;

    let nym_client =
        helpers_nym::set_enable_two_hop_with_recovery(&test_context.rpc_provider, nym_client, true)
            .await?;

    log::info!("Connecting tunnel...");
    let nym_client =
        helpers_nym::connect_tunnel_with_recovery(&test_context.rpc_provider, nym_client).await?;
    let (_, nym_client) = helpers_nym::wait_for_tunnel_state(
        rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    verify_tunnel_connectivity(rpc, verification).await?;

    log::info!("Disconnecting tunnel...");
    helpers_nym::disconnect_and_wait(rpc, nym_client, &test_context.rpc_provider).await?;

    Ok(())
}

/// Ensure connection with default DNS Nameservers blocklisted
#[test_function_nym(priority = 100)]
pub async fn test_tunnel_blocklisted_dns_nameservers_by_ip(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let dns_nameservers = get_default_nameserver_sockaddrs();
    let verification = verification_for_blocked_addrs(&dns_nameservers);
    log::debug!("Blocking DNS nameservers: {:?}", dns_nameservers);
    with_socket_blocks(&rpc, &dns_nameservers, || async {
        connect_verify_disconnect(&test_context, &rpc, nym_client, verification).await
    })
    .await
}

/// Test Connection with default VPN API host blocklisted
#[test_function_nym(priority = 101)]
pub async fn test_tunnel_blocklisted_vpn_api(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let vpn_api_hosts = ["nymvpn.com:443"];
    log::debug!("Adding blocking rule for the VPN API: {:?}", vpn_api_hosts);
    with_sni_blocks(&rpc, &vpn_api_hosts, || async {
        connect_verify_disconnect(
            &test_context,
            &rpc,
            nym_client,
            TunnelVerification::ResolveHostnames,
        )
        .await
    })
    .await
}

/// Test Connection with default NYM API host blocklisted
#[test_function_nym(priority = 102)]
pub async fn test_tunnel_blocklisted_nym_api(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let nym_api_hosts = ["validator.nymtech.net:443"];
    log::debug!("Adding blocking rule for the Nym API: {:?}", nym_api_hosts);
    with_sni_blocks(&rpc, &nym_api_hosts, || async {
        connect_verify_disconnect(
            &test_context,
            &rpc,
            nym_client,
            TunnelVerification::ResolveHostnames,
        )
        .await
    })
    .await
}

/// Test connection establishment with conditions seen in Russian ISP in Feb 2026.
///
/// Connections to the API that only transfered a small amount of data would be allowed, but
/// connections that exceeded around 3kB transfer would have traffic dropped. This resulted in a
/// Read error within the client. See NYM-349 for more details including logs and changes.
///
/// This test should result in a Read error during topology fetching that enables domain fronting
/// and then successfully connects. I block DNS to force usage of the  default fallback address for
/// the Nym API.
#[test_function_nym(priority = 103)]
pub async fn test_tunnel_delayed_blocklisted_nym_api(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let default_nym_api_socket_addr = ["212.71.233.232:443"];
    let dns_nameservers = get_default_nameserver_sockaddrs();
    let verification = verification_for_blocked_addrs(&dns_nameservers);

    log::debug!("Blocking DNS nameservers: {:?}", dns_nameservers);
    log::debug!(
        "Adding Delayed Blocking rule for Nym API: {:?}",
        default_nym_api_socket_addr
    );

    with_delayed_and_dns_blocks(
        &rpc,
        &dns_nameservers,
        &default_nym_api_socket_addr,
        || async { connect_verify_disconnect(&test_context, &rpc, nym_client, verification).await },
    )
    .await
}

async fn block_socket_addrs<T: AsRef<str> + std::fmt::Debug>(
    rpc: &NymServiceClient,
    socket_addrs: &[T],
) -> anyhow::Result<()> {
    let mut args = vec!["block"];
    args.extend(socket_addrs.iter().map(|s| s.as_ref()));

    let result = rpc
        .exec("/tmp/ip_block.sh", args)
        .await
        .context("Failed to execute ip_block.sh")?;

    if !result.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        anyhow::bail!("ip_block.sh failed to block socket addresses: {}", stderr);
    }

    log::info!("Successfully blocked socket addresses: {:?}", socket_addrs);
    Ok(())
}

async fn block_server_name_indicators<T: AsRef<str> + std::fmt::Debug>(
    rpc: &NymServiceClient,
    domains: &[T],
) -> anyhow::Result<()> {
    let mut args = vec!["block"];
    args.extend(domains.iter().map(|d| d.as_ref()));

    let result = rpc
        .exec("/tmp/sni_block.sh", args)
        .await
        .context("Failed to execute sni_block.sh")?;

    if !result.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        anyhow::bail!("sni_block.sh failed to block domains: {}", stderr);
    }

    log::info!("Successfully blocked domains: {:?}", domains);
    Ok(())
}

async fn block_socket_addrs_delayed<T: AsRef<str> + std::fmt::Debug>(
    rpc: &NymServiceClient,
    socket_addrs: &[T],
) -> anyhow::Result<()> {
    let mut args = vec!["block"];
    args.extend(socket_addrs.iter().map(|s| s.as_ref()));

    let result = rpc
        .exec("/tmp/delayed_ip_block.sh", args)
        .await
        .context("Failed to execute delayed_ip_block.sh")?;

    if !result.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        anyhow::bail!(
            "delayed_ip_block.sh failed to block socket addresses: {}",
            stderr
        );
    }

    log::info!(
        "Successfully blocked socket addresses (delayed): {:?}",
        socket_addrs
    );
    Ok(())
}

async fn unblock_socket_addrs<T: AsRef<str> + std::fmt::Debug>(
    rpc: &NymServiceClient,
    socket_addrs: &[T],
) -> anyhow::Result<()> {
    let mut args = vec!["unblock"];
    args.extend(socket_addrs.iter().map(|s| s.as_ref()));

    let result = rpc
        .exec("/tmp/ip_block.sh", args)
        .await
        .context("Failed to execute ip_block.sh")?;

    if !result.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        anyhow::bail!("ip_block.sh failed to unblock socket addresses: {}", stderr);
    }

    log::info!(
        "Successfully unblocked socket addresses: {:?}",
        socket_addrs
    );
    Ok(())
}

async fn unblock_server_name_indicators<T: AsRef<str> + std::fmt::Debug>(
    rpc: &NymServiceClient,
    domains: &[T],
) -> anyhow::Result<()> {
    let mut args = vec!["unblock"];
    args.extend(domains.iter().map(|d| d.as_ref()));

    let result = rpc
        .exec("/tmp/sni_block.sh", args)
        .await
        .context("Failed to execute sni_block.sh")?;

    if !result.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        anyhow::bail!("sni_block.sh failed to unblock domains: {}", stderr);
    }

    log::info!("Successfully unblocked domains: {:?}", domains);
    Ok(())
}

async fn unblock_socket_addrs_delayed<T: AsRef<str> + std::fmt::Debug>(
    rpc: &NymServiceClient,
    socket_addrs: &[T],
) -> anyhow::Result<()> {
    let mut args = vec!["unblock"];
    args.extend(socket_addrs.iter().map(|s| s.as_ref()));

    let result = rpc
        .exec("/tmp/delayed_ip_block.sh", args)
        .await
        .context("Failed to execute delayed_ip_block.sh")?;

    if !result.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        anyhow::bail!(
            "delayed_ip_block.sh failed to unblock socket addresses: {}",
            stderr
        );
    }

    log::info!(
        "Successfully unblocked socket addresses (delayed): {:?}",
        socket_addrs
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        IP_PROBE_SOCKET_ADDRS, TunnelVerification, get_default_nameserver_sockaddrs,
        merge_body_and_cleanup, merge_delayed_setup_failure, verification_for_blocked_addrs,
    };
    use crate::tests::get_test_descriptions;

    #[test]
    fn blocking_the_resolvers_downgrades_verification_to_ip_only() {
        assert_eq!(
            verification_for_blocked_addrs(&get_default_nameserver_sockaddrs()),
            TunnelVerification::ReachIpOnly,
            "the in-tunnel resolver forwards to these, so hostnames cannot resolve"
        );
        assert_eq!(
            verification_for_blocked_addrs(&["212.71.233.232:443".to_string()]),
            TunnelVerification::ResolveHostnames
        );
        assert_eq!(
            verification_for_blocked_addrs(&[]),
            TunnelVerification::ResolveHostnames
        );
    }

    #[test]
    fn ip_probe_addresses_are_never_blocked_by_the_dns_blocklist() {
        let blocked = get_default_nameserver_sockaddrs();
        for probe in IP_PROBE_SOCKET_ADDRS {
            assert!(
                !blocked.contains(&probe.to_string()),
                "probe {probe} is blocked by the very test that relies on it"
            );
        }
    }

    /// Must match `#[test_function_nym(priority = …)]` on the blocklist tests (after tunnel max 25).
    const BLOCKLIST_PRIORITY_DNS: i32 = 100;
    const BLOCKLIST_PRIORITY_VPN_API: i32 = 101;
    const BLOCKLIST_PRIORITY_NYM_API: i32 = 102;
    const BLOCKLIST_PRIORITY_DELAYED: i32 = 103;

    #[test]
    fn merge_chains_cleanup_error_when_body_also_fails() {
        let err = merge_body_and_cleanup(
            Err(anyhow::anyhow!("body failed")),
            Err(anyhow::anyhow!("cleanup failed")),
        )
        .expect_err("dual failure must still err");
        let rendered = format!("{err:#}");
        assert!(rendered.contains("body failed"), "{rendered}");
        assert!(
            rendered.contains("cleanup also failed") || rendered.contains("cleanup failed"),
            "{rendered}"
        );
    }

    #[test]
    fn merge_surfaces_cleanup_error_when_body_ok() {
        let err = merge_body_and_cleanup(Ok(()), Err(anyhow::anyhow!("cleanup failed")))
            .expect_err("cleanup must fail the test when body succeeded");
        assert!(err.to_string().contains("cleanup failed"));
    }

    #[test]
    fn merge_ok_when_both_ok() {
        merge_body_and_cleanup(Ok(()), Ok(())).expect("both ok");
    }

    #[test]
    fn delayed_setup_failure_keeps_delayed_error_when_dns_unblocks() {
        let err = merge_delayed_setup_failure(anyhow::anyhow!("delayed failed"), Ok(()));
        assert!(err.to_string().contains("delayed failed"));
    }

    #[test]
    fn delayed_setup_failure_chains_dns_unblock_error() {
        let err = merge_delayed_setup_failure(
            anyhow::anyhow!("delayed failed"),
            Err(anyhow::anyhow!("dns unblock failed")),
        );
        let rendered = format!("{err:#}");
        assert!(rendered.contains("delayed failed"), "{rendered}");
        assert!(
            rendered.contains("DNS unblock") || rendered.contains("dns unblock failed"),
            "{rendered}"
        );
    }

    #[test]
    fn blocklist_priorities_run_after_core_suite() {
        const _: () = assert!(BLOCKLIST_PRIORITY_DNS > 25);
        const _: () = assert!(BLOCKLIST_PRIORITY_VPN_API > BLOCKLIST_PRIORITY_DNS);
        const _: () = assert!(BLOCKLIST_PRIORITY_NYM_API > BLOCKLIST_PRIORITY_VPN_API);
        const _: () = assert!(BLOCKLIST_PRIORITY_DELAYED > BLOCKLIST_PRIORITY_NYM_API);

        let tests = get_test_descriptions();
        let max_non_blocklist = tests
            .iter()
            .filter(|t| !t.name.contains("blocklist"))
            .map(|t| t.priority.unwrap_or(0))
            .max()
            .unwrap_or(0);
        let min_blocklist = tests
            .iter()
            .filter(|t| t.name.contains("blocklist"))
            .map(|t| t.priority.unwrap_or(0))
            .min()
            .expect("blocklist tests must be registered");
        assert!(
            min_blocklist > max_non_blocklist,
            "blocklist min priority {min_blocklist} must exceed non-blocklist max {max_non_blocklist}"
        );

        let priority_of = |name: &str| {
            tests
                .iter()
                .find(|t| t.name == name)
                .and_then(|t| t.priority)
                .unwrap_or(0)
        };
        assert_eq!(
            priority_of("test_tunnel_blocklisted_dns_nameservers_by_ip"),
            BLOCKLIST_PRIORITY_DNS
        );
        assert_eq!(
            priority_of("test_tunnel_blocklisted_vpn_api"),
            BLOCKLIST_PRIORITY_VPN_API
        );
        assert_eq!(
            priority_of("test_tunnel_blocklisted_nym_api"),
            BLOCKLIST_PRIORITY_NYM_API
        );
        assert_eq!(
            priority_of("test_tunnel_delayed_blocklisted_nym_api"),
            BLOCKLIST_PRIORITY_DELAYED
        );
    }
}
