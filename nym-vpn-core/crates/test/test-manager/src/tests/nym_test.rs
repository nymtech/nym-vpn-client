// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    nym_daemon::RpcClientProvider,
    tests::{
        TestContext,
        helpers_nym::{self, resolve_hostname_with_retry},
    },
};
use anyhow::{Context, ensure};
use helpers_nym::ExpectedTunnelState;
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use std::time::Duration;
use test_macro::test_function_nym;
use test_rpc::{NymServiceClient, nym_daemon::ObservedAccountState};

/// Per-hostname budget for in-VM DNS after connect (matches tunnel reconnect checks).
const ROUNDTRIP_DNS_TIMEOUT: Duration = Duration::from_secs(30);

#[test_function_nym]
pub async fn test_account_and_tunnel_roundtrip(
    test_context: TestContext,
    rpc: NymServiceClient,
    nym_proxy_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    let mut nym_proxy_client =
        dc_and_ensure_logged_in(&rpc, nym_proxy_client, &test_context.rpc_provider, false).await?;

    // Verify account identity
    let identity = nym_proxy_client
        .get_account_identity()
        .await
        .context("get_account_identity failed")?;
    let identity = identity.context("Expected account identity to be set")?;
    ensure!(!identity.is_empty(), "Account identity should not be empty");
    log::info!(
        "Account identity: {}...",
        &identity[..5.min(identity.len())]
    );

    // Connect tunnel
    log::info!("Connecting tunnel...");
    nym_proxy_client.connect_tunnel().await?;
    let (_, mut nym_proxy_client) = helpers_nym::wait_for_tunnel_state(
        &rpc,
        nym_proxy_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    // DNS resolution while connected (runs inside VM via tarpc). Bounded so a
    // stalled resolver cannot wedge the suite until outer SSH keepalives kill CI.
    let hostnames_to_test = ["nym.com", "google.com"];
    for host in &hostnames_to_test {
        log::info!("Resolving {} inside VM...", host);
        let addrs = resolve_hostname_with_retry(&rpc, host, ROUNDTRIP_DNS_TIMEOUT)
            .await
            .with_context(|| format!("DNS resolution failed for {host} inside VM"))?;
        log::info!("Resolved {} to {:?}", host, addrs);
    }

    // Disconnect tunnel
    log::info!("Disconnecting tunnel...");
    nym_proxy_client.disconnect_tunnel().await?;
    let (_, mut nym_proxy_client) = helpers_nym::wait_for_tunnel_state(
        &rpc,
        nym_proxy_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Disconnected,
    )
    .await?;

    // Verify devices
    let devices = nym_proxy_client
        .get_active_devices()
        .await
        .context("get_active_devices failed")?;
    ensure!(!devices.is_empty(), "Expected at least one active device");
    for device in &devices {
        ensure!(
            !device.device_identity_key.is_empty(),
            "Device identity key should not be empty"
        );
        log::info!(
            "Device: {}... created={}, status={:?}",
            &device.device_identity_key[..10.min(device.device_identity_key.len())],
            device.created_on_utc,
            device.status,
        );
    }

    // Verify usage
    let usages = nym_proxy_client
        .get_account_usage()
        .await
        .context("get_account_usage failed")?;
    ensure!(!usages.is_empty(), "Expected at least one usage entry");
    for (i, usage) in usages.iter().enumerate() {
        log::info!(
            "Usage [{}/{}]: valid_until={}, used={}GB, allowance={}GB",
            i + 1,
            usages.len(),
            usage.valid_until_utc,
            usage.bandwidth_used_gb,
            usage.bandwidth_allowance_gb,
        );
    }

    Ok(())
}

/// Make sure the daemon is installed and logged in and restore settings to the defaults.
pub async fn dc_and_ensure_logged_in(
    runner: &NymServiceClient,
    mut nym_proxy_client: NymProxyClient,
    provider: &RpcClientProvider,
    forget_account: bool,
) -> anyhow::Result<NymProxyClient> {
    log::debug!("🔄 Resetting daemon settings before test...");
    nym_proxy_client = helpers_nym::disconnect_and_wait(runner, nym_proxy_client, provider)
        .await
        .context("Failed to disconnect")?;

    if forget_account {
        log::debug!("🔄 Resetting device identity & ticketbooks...");
        nym_proxy_client.forget_account().await?;
        helpers_nym::wait_for_account_state(runner, ObservedAccountState::LoggedOut).await?;
    }

    nym_proxy_client = helpers_nym::login_idempotent(runner, nym_proxy_client, provider)
        .await
        .context("Failed to ensure logged in")?;

    if let Err(err) = nym_proxy_client.set_allow_lan(true).await {
        log::warn!("Failed to enable allow_lan for diagnostics: {err}");
    }

    log::debug!("🔄 Daemon successfully prepared 🔄");

    Ok(nym_proxy_client)
}
