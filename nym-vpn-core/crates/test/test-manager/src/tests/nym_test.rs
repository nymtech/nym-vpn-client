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
    let nym_proxy_client =
        dc_and_ensure_logged_in(&rpc, nym_proxy_client, &test_context.rpc_provider, false).await?;

    let (identity, nym_proxy_client) = helpers_nym::call_nym_with_transport_recovery(
        &test_context.rpc_provider,
        nym_proxy_client,
        |mut client| async move {
            let result = client.get_account_identity().await;
            (client, result)
        },
    )
    .await
    .context("get_account_identity failed")?;
    let identity = identity.context("Expected account identity to be set")?;
    ensure!(!identity.is_empty(), "Account identity should not be empty");
    log::info!(
        "Account identity: {}...",
        &identity[..5.min(identity.len())]
    );

    log::info!("Connecting tunnel...");
    let nym_proxy_client =
        helpers_nym::connect_tunnel_with_recovery(&test_context.rpc_provider, nym_proxy_client)
            .await?;
    let (_, nym_proxy_client) = helpers_nym::wait_for_tunnel_state(
        &rpc,
        nym_proxy_client,
        &test_context.rpc_provider,
        ExpectedTunnelState::Connected,
    )
    .await?;

    let hostnames_to_test = ["nym.com", "google.com"];
    for host in &hostnames_to_test {
        log::info!("Resolving {} inside VM...", host);
        let addrs = resolve_hostname_with_retry(&rpc, host, ROUNDTRIP_DNS_TIMEOUT)
            .await
            .with_context(|| format!("DNS resolution failed for {host} inside VM"))?;
        log::info!("Resolved {} to {:?}", host, addrs);
    }

    log::info!("Disconnecting tunnel...");
    let nym_proxy_client =
        helpers_nym::disconnect_and_wait(&rpc, nym_proxy_client, &test_context.rpc_provider)
            .await
            .context("Failed to disconnect after Connected")?;

    let (devices, nym_proxy_client) = helpers_nym::call_nym_with_transport_recovery(
        &test_context.rpc_provider,
        nym_proxy_client,
        |mut client| async move {
            let result = client.get_active_devices().await;
            (client, result)
        },
    )
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

    let (usages, _nym_proxy_client) = helpers_nym::call_nym_with_transport_recovery(
        &test_context.rpc_provider,
        nym_proxy_client,
        |mut client| async move {
            let result = client.get_account_usage().await;
            (client, result)
        },
    )
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

pub async fn dc_and_ensure_logged_in(
    runner: &NymServiceClient,
    mut nym_proxy_client: NymProxyClient,
    provider: &RpcClientProvider,
    forget_account: bool,
) -> anyhow::Result<NymProxyClient> {
    log::debug!("Resetting daemon settings before test...");
    nym_proxy_client = helpers_nym::disconnect_and_wait(runner, nym_proxy_client, provider)
        .await
        .context("Failed to disconnect")?;

    nym_proxy_client =
        helpers_nym::replace_client_after_disconnect_prep(provider, nym_proxy_client)
            .await
            .context("Failed to recover DaemonRpc after disconnect prep")?;

    if forget_account {
        log::debug!("Resetting device identity & ticketbooks...");
        let (_, client) = helpers_nym::call_nym_with_transport_recovery(
            provider,
            nym_proxy_client,
            |mut client| async move {
                let result = client.forget_account().await;
                (client, result)
            },
        )
        .await
        .context("forget_account failed")?;
        nym_proxy_client = client;
        helpers_nym::wait_for_account_state(runner, ObservedAccountState::LoggedOut).await?;
    }

    nym_proxy_client = helpers_nym::login_idempotent(runner, nym_proxy_client, provider)
        .await
        .context("Failed to ensure logged in")?;

    nym_proxy_client = helpers_nym::finish_prep_with_allow_lan(provider, nym_proxy_client)
        .await
        .context("DaemonRpc unresponsive after login prep")?;

    log::debug!("Daemon successfully prepared");

    Ok(nym_proxy_client)
}
