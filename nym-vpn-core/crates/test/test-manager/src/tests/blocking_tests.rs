//! Censorship scenario tests

use crate::tests::{
    TestContext,
    helpers_nym::{self},
    nym_test::dc_and_ensure_logged_in,
};
use anyhow::{Context, ensure};
use helpers_nym::ExpectedTunnelState;
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use test_macro::test_function_nym;
use test_rpc::NymServiceClient;

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

/// Ensure connection with default DNS Nameservers blocklisted
#[test_function_nym]
pub async fn test_tunnel_blocklisted_dns_nameservers_by_ip(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    // Common DNS nameservers to block
    let dns_nameservers = get_default_nameserver_sockaddrs();

    // Block DNS nameservers
    log::debug!("Blocking DNS nameservers: {:?}", dns_nameservers);
    block_socket_addrs(&rpc, &dns_nameservers).await?;

    // Ensure proper login state
    let mut nym_client = dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    // connect with wg
    nym_client.set_enable_two_hop(true).await?;

    // Connect tunnel
    log::info!("Connecting tunnel...");
    nym_client.connect_tunnel().await?;
    let (_, mut nym_client) = helpers_nym::wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    // Verify tunnel connectivity
    verify_tunnel_connectivity(&rpc).await?;

    // Disconnect tunnel
    log::info!("Disconnecting tunnel...");
    nym_client.disconnect_tunnel().await?;
    helpers_nym::wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Disconnected,
    )
    .await?;

    // Clean up: unblock DNS nameservers
    log::debug!("Unblocking DNS nameservers...");
    unblock_socket_addrs(&rpc, &dns_nameservers).await?;

    Ok(())
}

/// Test Connection with default VPN API host blocklisted
#[test_function_nym]
pub async fn test_tunnel_blocklisted_vpn_api(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    // TODO: Determine the actual VPN API endpoints to block
    let vpn_api_hosts = ["nymvpn.com:443"];

    // Block VPN API hosts
    log::debug!("Adding blocking rule for the VPN API: {:?}", vpn_api_hosts);
    block_server_name_indicators(&rpc, &vpn_api_hosts).await?;

    // Ensure we're logged out first
    let mut nym_client = dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    // connect with wg
    nym_client.set_enable_two_hop(true).await?;

    // Attempt to connect - this should either fail or bypass the blocking
    log::info!("Attempting to connect tunnel with VPN API blocked...");
    nym_client.connect_tunnel().await?;
    let (_, mut nym_client) = helpers_nym::wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    // Verify tunnel connectivity
    verify_tunnel_connectivity(&rpc).await?;

    // Disconnect tunnel
    log::info!("Disconnecting tunnel...");
    nym_client.disconnect_tunnel().await?;
    helpers_nym::wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Disconnected,
    )
    .await?;

    // Clean up: unblock the VPN API hosts
    log::debug!("Removing SNI based blocking rule for the VPN API ...");
    unblock_server_name_indicators(&rpc, &vpn_api_hosts).await?;

    Ok(())
}

/// Test Connection with default NYM API host blocklisted
#[test_function_nym]
pub async fn test_tunnel_blocklisted_nym_api(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    // TODO: Determine the actual Nym API endpoints to block
    let nym_api_hosts = ["validator.nymtech.net:443"];

    // Block Nym API hosts
    log::debug!("Adding blocking rule for the Nym API: {:?}", nym_api_hosts);
    block_server_name_indicators(&rpc, &nym_api_hosts).await?;

    // Ensure we're logged out first
    let mut nym_client = dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    // connect with wg
    nym_client.set_enable_two_hop(true).await?;

    // Attempt to connect - this should either fail or bypass the blocking
    log::info!("Attempting to connect tunnel with Nym API blocked...");
    nym_client.connect_tunnel().await?;
    let (_, mut nym_client) = helpers_nym::wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    // Verify tunnel connectivity
    verify_tunnel_connectivity(&rpc).await?;

    // Disconnect tunnel
    log::info!("Disconnecting tunnel...");
    nym_client.disconnect_tunnel().await?;
    helpers_nym::wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Disconnected,
    )
    .await?;

    // Clean up: unblock the Nym API hosts
    log::debug!("Removing SNI based blocking rule for the Nym API ...");
    unblock_server_name_indicators(&rpc, &nym_api_hosts).await?;

    Ok(())
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
#[test_function_nym]
pub async fn test_tunnel_delayed_blocklisted_nym_api(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let default_nym_api_socket_addr = ["212.71.233.232:443"];

    // Common DNS nameservers to block
    let dns_nameservers = get_default_nameserver_sockaddrs();

    // Block DNS nameservers
    log::debug!("Blocking DNS nameservers: {:?}", dns_nameservers);
    block_socket_addrs(&rpc, &dns_nameservers).await?;

    // Block DNS nameservers
    log::debug!(
        "Adding Delayed Blocking rule for Nym API: {:?}",
        default_nym_api_socket_addr
    );
    block_socket_addrs_delayed(&rpc, &default_nym_api_socket_addr).await?;

    // Ensure we're logged out first
    let mut nym_client = dc_and_ensure_logged_in(&rpc, nym_client, &test_context.rpc_provider, false).await?;

    // connect with wg
    nym_client.set_enable_two_hop(true).await?;

    // Attempt to connect - this should either fail or bypass the blocking
    log::info!("Attempting to connect tunnel with Nym API blocked...");
    nym_client.connect_tunnel().await?;
    let (_, mut nym_client) = helpers_nym::wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    // Verify tunnel connectivity
    verify_tunnel_connectivity(&rpc).await?;

    // Disconnect tunnel
    log::info!("Disconnecting tunnel...");
    nym_client.disconnect_tunnel().await?;
    helpers_nym::wait_for_tunnel_state(
        &rpc,
        nym_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Disconnected,
    )
    .await?;

    // Block DNS nameservers
    log::debug!("Removing Delayed Blocking rule for Nym API...");
    unblock_socket_addrs_delayed(&rpc, &default_nym_api_socket_addr).await?;

    // Clean up: unblock DNS nameservers
    log::debug!("Unblocking DNS nameservers...");
    unblock_socket_addrs(&rpc, &dns_nameservers).await?;

    Ok(())
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
