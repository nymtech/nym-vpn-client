// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::tests::{
    TestContext, account_nym::forget_current_device, config_nym::TEST_CONFIG_NYM, helpers_nym,
};
use anyhow::{Context, bail, ensure};
use helpers_nym::ExpectedTunnelState;
use nym_vpn_lib_types::AccountControllerState;
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use std::time::Duration;
use test_macro::test_function_nym;
use test_rpc::NymServiceClient;

/// Poll `is_account_stored()` until it returns `true`, or bail after `timeout`.
pub async fn wait_for_account_stored(
    nym_proxy_client: &mut NymProxyClient,
    timeout: Duration,
) -> anyhow::Result<()> {
    tokio::time::timeout(timeout, async {
        loop {
            match nym_proxy_client.is_account_stored().await {
                Ok(true) => {
                    log::debug!("Account stored confirmed");
                    return Ok(());
                }
                Ok(false) => continue,
                Err(e) => bail!("Failed to check if account was stored: {e}"),
            }
        }
    })
    .await
    .map_err(anyhow::Error::msg)?
}

#[test_function_nym]
pub async fn test_account_and_tunnel_roundtrip(
    _: TestContext,
    rpc: NymServiceClient,
    mut nym_proxy_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    prepare_daemon_nym(&mut nym_proxy_client, false).await?;

    // Store account
    log::info!("Storing account...");
    if let Some(err) = nym_proxy_client
        .store_account_friendly(&TEST_CONFIG_NYM.mnemonic)
        .await?
        .error
    {
        bail!("store_account_friendly returned error: {err}");
    }

    // Wait for account to be stored (bounded)
    wait_for_account_stored(&mut nym_proxy_client, Duration::from_secs(60)).await?;
    wait_for_account_state(
        &mut nym_proxy_client,
        AccountControllerState::ReadyToConnect,
    )
    .await?;

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
    nym_proxy_client.connect_tunnel_friendly().await?;
    helpers_nym::wait_for_tunnel_state(&mut nym_proxy_client, ExpectedTunnelState::Connected)
        .await?;

    // DNS resolution while connected (runs inside VM via tarpc)
    let hostnames_to_test = ["nym.com", "google.com"];
    for host in &hostnames_to_test {
        log::info!("Resolving {} inside VM...", host);
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

    // Disconnect tunnel
    log::info!("Disconnecting tunnel...");
    nym_proxy_client.disconnect_tunnel().await?;
    helpers_nym::wait_for_tunnel_state(&mut nym_proxy_client, ExpectedTunnelState::Disconnected)
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

pub async fn wait_for_account_state(
    nym_proxy_client: &mut NymProxyClient,
    expected_state: AccountControllerState,
) -> anyhow::Result<()> {
    let timeout = Duration::from_secs(60);

    tokio::time::timeout(timeout, async {
        loop {
            match nym_proxy_client.get_account_state().await {
                Ok(current_state) => {
                    if current_state.eq(&expected_state) {
                        log::debug!("Account state {current_state} reached!");
                        return Ok(());
                    } else {
                        log::trace!(
                            "Account state: {current_state:?} (expecting {expected_state:?})"
                        );
                    }
                }
                Err(e) => bail!("Failed to get account state: {e}"),
            }
        }
    })
    .await
    .map_err(|_| {
        anyhow::anyhow!(
            "Account state listener timed out after {}s",
            timeout.as_secs()
        )
    })?
}

/// Make sure the daemon is installed and logged in and restore settings to the defaults.
pub async fn prepare_daemon_nym(
    nym_proxy_client: &mut NymProxyClient,
    ensure_logged_in: bool,
) -> anyhow::Result<()> {
    log::debug!("🔄 Resetting daemon settings before test...");
    helpers_nym::disconnect_and_wait(nym_proxy_client)
        .await
        .context("Failed to disconnect daemon after test")?;

    if ensure_logged_in {
        log::debug!("🔄 Ensuring account is logged in to nym-vpnd...");
        helpers_nym::ensure_logged_in(nym_proxy_client).await?;
    } else {
        log::debug!("🔄 Resetting device identity...");
        forget_current_device(nym_proxy_client).await?;
    }
    log::debug!("🔄 Daemon successfully reset 🔄");

    Ok(())
}
