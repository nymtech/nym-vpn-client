// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only


use crate::nym_daemon::RpcClientProvider;
use crate::tests::config_nym::TEST_CONFIG_NYM;
use crate::tests::{TestContext, helpers_nym};
use anyhow::{Context, anyhow, bail, ensure};
use nym_vpn_lib_types::{AccountControllerErrorStateReason, AccountControllerState, TunnelState};
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use std::time::Duration;
use test_macro::test_function_nym;
use test_rpc::NymServiceClient;
use tokio::time::Instant;

/// Poll `is_account_stored()` until it returns `true`, or bail after `timeout`.
pub async fn wait_for_account_stored(
    nym_proxy_client: &mut NymProxyClient,
    timeout: Duration,
) -> anyhow::Result<()> {
    let started = Instant::now();
    loop {
        if nym_proxy_client.is_account_stored().await? {
            log::debug!("Account stored confirmed");
            return Ok(());
        }
        if Instant::now() > started + timeout {
            bail!("Account was not stored after {}s", timeout.as_secs());
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

#[test_function_nym]
pub async fn test_account_and_tunnel_roundtrip(
    _: TestContext,
    rpc: NymServiceClient,
    mut nym_proxy_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    log::info!("test_account_and_tunnel_roundtrip: account store, tunnel connect/disconnect, device & usage check");
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
    log::info!("Account identity: {}...", &identity[..5.min(identity.len())]);

    // Connect tunnel
    log::info!("Connecting tunnel...");
    nym_proxy_client.connect_tunnel_friendly().await?;
    wait_for_tunnel_state(&mut nym_proxy_client, ExpectedTunnelState::Connected).await?;

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
    wait_for_tunnel_state(&mut nym_proxy_client, ExpectedTunnelState::Disconnected).await?;

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

    log::info!("test_account_and_tunnel_roundtrip: PASSED");
    Ok(())
}

#[derive(Debug, PartialEq)]
pub enum ExpectedTunnelState {
    Connected,
    Disconnected,
    Connecting,
    Disconnecting,
    Offline,
    Error(String),
}

impl From<TunnelState> for ExpectedTunnelState {
    fn from(value: TunnelState) -> Self {
        match value {
            TunnelState::Connected { .. } => ExpectedTunnelState::Connected,
            TunnelState::Disconnected { .. } => ExpectedTunnelState::Disconnected,
            TunnelState::Connecting { .. } => ExpectedTunnelState::Connecting,
            TunnelState::Disconnecting { .. } => ExpectedTunnelState::Disconnecting,
            TunnelState::Offline { .. } => ExpectedTunnelState::Offline,
            TunnelState::Error(reason) => ExpectedTunnelState::Error(reason.to_string()),
        }
    }
}

pub async fn wait_for_tunnel_state(
    nym_proxy_client: &mut NymProxyClient,
    expected_state: ExpectedTunnelState,
) -> anyhow::Result<()> {
    wait_for_tunnel_state_with_timeout(nym_proxy_client, expected_state, Duration::from_secs(60))
        .await
}

pub async fn wait_for_tunnel_state_with_timeout(
    nym_proxy_client: &mut NymProxyClient,
    expected_state: ExpectedTunnelState,
    timeout: Duration,
) -> anyhow::Result<()> {
    let started = Instant::now();

    loop {
        let current_state: ExpectedTunnelState = nym_proxy_client.get_tunnel_state().await?.into();
        if current_state == expected_state {
            log::debug!("✅ Tunnel state {current_state:?} reached!");
            return Ok(());
        } else if Instant::now() > started + timeout {
            bail!(
                "Couldn't reach {expected_state:?} state in {}s",
                timeout.as_secs()
            );
        }
        log::debug!("Tunnel state: {current_state:?} (expecting {expected_state:?})");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

pub async fn wait_for_account_state(
    nym_proxy_client: &mut NymProxyClient,
    expected_state: AccountControllerState,
) -> anyhow::Result<()> {
    let timeout = Duration::from_secs(60);
    let started = Instant::now();

    loop {
        let current_state = nym_proxy_client.get_account_state().await?;
        if current_state == expected_state {
            log::debug!("Account state {current_state} reached!");
            return Ok(());
        } else if Instant::now() > started + timeout {
            bail!(
                "Couldn't reach {expected_state:?} in {}s",
                timeout.as_secs()
            );
        }
        log::debug!("Account state: {current_state:?} (expecting {expected_state:?})");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

/// Make sure the daemon is installed and logged in and restore settings to the defaults.
pub async fn prepare_daemon_nym(
    nym_proxy_client: &mut NymProxyClient,
    ensure_logged_in: bool,
) -> anyhow::Result<()> {
    // Check if daemon should be restarted
    // let mut nym_client = ensure_daemon_version_nym(rpc, rpc_provider)
    //     .await
    //     .context("Failed to restart daemon")?;

    log::debug!("🔄 Resetting daemon settings before test...");
    helpers_nym::disconnect_and_wait(nym_proxy_client)
        .await
        .context("Failed to disconnect daemon after test")?;
    log::debug!("🔄 Resetting device identity...");
    nym_proxy_client
        .reset_device_identity(None)
        .await
        .context("Failed to reset settings")?;
    log::debug!("🔄 Ensuring account is logged in to nym-vpnd...");

    log::debug!("🔄 Daemon successfully reset 🔄");
    if ensure_logged_in {
        helpers_nym::ensure_logged_in(nym_proxy_client).await?;
    }

    Ok(())
}

/// Reset the daemons environment.
///
/// Will and restart or reinstall it if necessary.
// async fn ensure_daemon_version_nym(
//     rpc: &NymServiceClient,
//     rpc_provider: &RpcClientProvider,
// ) -> anyhow::Result<NymProxyClient> {
//     let app_package_filename = &TEST_CONFIG.app_package_filename;
//
//     let must_reinstall_app =
//         match correct_daemon_version_is_running(rpc_provider.new_client_nym().await).await {
//             Ok(correct_version) => !correct_version,
//             // Failing to reach the daemon is a sign that it is not installed
//             Err(mullvad_management_interface::Error::Rpc(..)) => {
//                 log::debug!("Daemon is not running, attempting to start it");
//
//                 let failed_starting_daemon = rpc.enable_nymvpn_daemon().await.is_err()
//                     || rpc.start_nymvpn_daemon().await.is_err();
//                 if failed_starting_daemon {
//                     log::warn!("Failed to start the daemon service");
//                 }
//                 failed_starting_daemon
//             }
//             Err(e) => panic!("Failed to get app version: {e}"),
//         };
//
//     if must_reinstall_app {
//         // NOTE: Reinstalling the app resets the daemon environment
//         helpers_nym::install_app(rpc, app_package_filename, rpc_provider)
//             .await
//             .with_context(|| format!("Failed to install app '{app_package_filename}'"))
//     } else {
//         ensure_daemon_environment_nym(rpc)
//             .await
//             .context("Failed to reset daemon environment")?;
//
//         Ok(rpc_provider.new_client_nym().await)
//     }
// }

async fn ensure_daemon_environment_nym(rpc: &NymServiceClient) -> Result<(), anyhow::Error> {
    let current_env = rpc
        .get_daemon_environment_overrides()
        .await
        .context("Failed to get daemon env variables")?;
    let default_env = helpers_nym::get_app_env()
        .await
        .context("Failed to get daemon default env variables")?;
    if current_env != default_env {
        log::debug!(
            "Restarting daemon due changed environment variables. Values since last test {current_env:?}"
        );
        rpc.set_daemon_environment(default_env)
            .await
            .context("Failed to restart daemon")?;
    };
    Ok(())
}

// TODO dz Nym doesn't have a version check API
// async fn correct_daemon_version_is_running(
//     mut nym_client: NymProxyClient,
// ) -> Result<bool, mullvad_management_interface::Error> {
//     let app_package_filename = &TEST_CONFIG.app_package_filename;
//     let expected_version = get_version_from_path(std::path::Path::new(app_package_filename))
//         .unwrap_or_else(|_| panic!("Invalid app version: {app_package_filename}"));

//     let version = nym_client.get_current_version().await?;
//     Ok(version == expected_version)
// }
