//! Censorship scenario tests
//!
//! Priorities are intentionally high so these run after the core suite. Block rules can
//! poison guest networking / serial RPC if cleanup is skipped on failure.

use crate::tests::{
    TestContext,
    helpers_nym::{self},
    nym_test::dc_and_ensure_logged_in,
};
use anyhow::{Context, ensure};
use helpers_nym::ExpectedTunnelState;
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use std::future::Future;
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

/// Verify that the VPN tunnel is working by performing DNS resolution tests
async fn verify_tunnel_connectivity(rpc: &NymServiceClient) -> anyhow::Result<()> {
    let hostnames_to_test = ["nym.com", "google.com"];
    for host in &hostnames_to_test {
        log::info!("Resolving {} inside VM via VPN tunnel...", host);
        let addrs = rpc
            .resolve_hostname(host.to_string())
            .await
            .context(format!("DNS resolution failed for {} inside VM", host))?;
        log::info!("Resolved {} to {:?}", host, addrs);
        ensure!(
            !addrs.is_empty(),
            "DNS resolution returned no addresses for {} inside VM",
            host
        );
    }
    Ok(())
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
) -> anyhow::Result<()> {
    let mut nym_client =
        dc_and_ensure_logged_in(rpc, nym_client, &test_context.rpc_provider, false).await?;

    nym_client.set_enable_two_hop(true).await?;

    log::info!("Connecting tunnel...");
    nym_client.connect_tunnel().await?;
    let (_, mut nym_client) = helpers_nym::wait_for_tunnel_state(
        rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    verify_tunnel_connectivity(rpc).await?;

    log::info!("Disconnecting tunnel...");
    nym_client.disconnect_tunnel().await?;
    helpers_nym::wait_for_tunnel_state(
        rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Disconnected,
    )
    .await?;

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
    log::debug!("Blocking DNS nameservers: {:?}", dns_nameservers);
    with_socket_blocks(&rpc, &dns_nameservers, || async {
        connect_verify_disconnect(&test_context, &rpc, nym_client).await
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
        connect_verify_disconnect(&test_context, &rpc, nym_client).await
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
        connect_verify_disconnect(&test_context, &rpc, nym_client).await
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

    log::debug!("Blocking DNS nameservers: {:?}", dns_nameservers);
    log::debug!(
        "Adding Delayed Blocking rule for Nym API: {:?}",
        default_nym_api_socket_addr
    );

    with_delayed_and_dns_blocks(
        &rpc,
        &dns_nameservers,
        &default_nym_api_socket_addr,
        || async { connect_verify_disconnect(&test_context, &rpc, nym_client).await },
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
    use super::{merge_body_and_cleanup, merge_delayed_setup_failure};
    use crate::tests::get_test_descriptions;

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
        assert!(BLOCKLIST_PRIORITY_DNS > 25);
        assert!(BLOCKLIST_PRIORITY_VPN_API > BLOCKLIST_PRIORITY_DNS);
        assert!(BLOCKLIST_PRIORITY_NYM_API > BLOCKLIST_PRIORITY_VPN_API);
        assert!(BLOCKLIST_PRIORITY_DELAYED > BLOCKLIST_PRIORITY_NYM_API);

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
