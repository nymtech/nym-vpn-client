// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// TODO dz add a test with the following steps

// get_account_identity
// get_account_state
//
// store_account
// is_account_stored
//
// get_account_identity
// get_account_state
// get_account_usage
//
// set_network
//
// list_gateways
//
// connect_tunnel
// get_tunnel_state
//
// <do something: ping or download a file>
//
// disconnect_tunnel
// get_tunnel_state
//
// get_active_devices
// get_account_usage

// TODO dz test annotated with `test_function_nym` should come here
// test function will have access to RPC context & Nym client

use crate::nym_daemon::RpcClientProvider;
use crate::tests::config_nym::TEST_CONFIG_NYM;
use crate::tests::{TestContext, helpers_nym};
use anyhow::{Context, bail};
use nym_vpn_lib_types::{AccountControllerState, TunnelState};
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use std::time::Duration;
use test_macro::test_function_nym;
use test_rpc::NymServiceClient;
use tokio::time::Instant;

#[test_function_nym]
pub async fn test_happy_nym(
    _: TestContext,
    rpc: NymServiceClient,
    mut nym_proxy_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    log::info!("🏗️ 🏗️ 🏗️ Starting a sample Nym test");
    // prepare_daemon_nym(&rpc, rpc_provider).await?;

    let daemon_version = rpc
        .nymvpn_daemon_version()
        .await
        .inspect_err(|err| log::error!("Failed to get daemon version {err}"))?;

    log::debug!("Nym daemon version: {daemon_version}");

    rpc.start_nymvpn_daemon()
        .await
        .inspect_err(|err| log::error!("Failed to start / restart nymvpn daemon {err}"))?;

    let status = rpc.nymvpn_daemon_get_status().await?;
    log::debug!("nym-vpnd status: {status:?}");

    log::debug!("Trying to stop nym-vpnd");
    rpc.stop_nymvpn_daemon().await?;
    let status = rpc.nymvpn_daemon_get_status().await?;
    log::debug!("nym-vpnd status: {status:?}");

    log::debug!("Trying to start nym-vpnd again...");
    rpc.start_nymvpn_daemon().await?;
    loop {
        let status = rpc.nymvpn_daemon_get_status().await?;
        log::debug!("nym-vpnd status: {status:?}");
        if status.is_running() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    log::info!("🚀 🚀 🚀 Successfully completed a sample Nym test");

    Ok(())
}

// async fn ip() -> Result<(), Box<dyn std::error::Error>> {
//     let resp = reqwest::get("https://ipinfo.io").await?.text().await?;
//     println!("{}", resp);
//     Ok(())
// }

#[test_function_nym]
pub async fn basic_functionality(
    _: TestContext,
    rpc: NymServiceClient,
    mut nym_proxy_client: NymProxyClient,
) -> Result<(), anyhow::Error> {
    log::info!(" 🏗 Basic functionality test");
    // prepare_daemon_nym(&rpc, rpc_provider).await?;

    let is_stored = nym_proxy_client.is_account_stored().await?;
    let account_state = nym_proxy_client.get_account_state().await?;
    let account_identity = nym_proxy_client.get_account_identity().await?;
    log::debug!("nym-vpnd has a registered account: {is_stored}");
    log::debug!("Account state: {account_state:?}");
    log::debug!("Account identity: {account_identity:?}");

    log::debug!("Registering a mnemonic...");
    if let Some(err) = nym_proxy_client
        .store_account_friendly(&TEST_CONFIG_NYM.mnemonic)
        .await?
        .error
    {
        log::error!("{}", err);
    }

    let timeout = tokio::time::sleep(Duration::from_secs(60 * 2)).await;
    loop {
        let is_stored = nym_proxy_client.is_account_stored().await?;
        log::debug!("nym-vpnd has a registered account: {is_stored}");
        if is_stored {
            break;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    wait_for_account_state(
        &mut nym_proxy_client,
        AccountControllerState::ReadyToConnect,
    )
    .await?;
    let account_identity = nym_proxy_client.get_account_identity().await?;
    if let Some(identity) = account_identity {
        log::debug!("Account identity: {}...", &identity[..5]);
    }

    log::info!("🚀 Connecting tunnel...");
    nym_proxy_client.connect_tunnel_friendly().await?;
    wait_for_tunnel_state(&mut nym_proxy_client, ExpectedTunnelState::Connected).await?;

    let hostnames_to_test = vec![("nym.com", 443), ("google.com", 443)];
    for host in hostnames_to_test {
        log::debug!("🌍 Trying to resolve {}", host.0);
        let result = helpers_nym::resolve_hostname_with_retries(host).await;
        log::debug!("Result: {:?}", result);
    }

    log::info!("🔌 Disconnecting tunnel...");
    nym_proxy_client.disconnect_tunnel().await?;
    wait_for_tunnel_state(&mut nym_proxy_client, ExpectedTunnelState::Disconnected).await?;

    let devices = nym_proxy_client.get_active_devices().await?;
    log::debug!("Listing active devices:");
    for device in devices {
        log::debug!(
            "{}...: created on: {}, status: {:?}",
            &device.device_identity_key[..10],
            device.created_on_utc,
            device.status
        );
    }

    let usages = nym_proxy_client.get_account_usage().await?;
    log::debug!("Usage details:");
    for usage in usages {
        log::debug!("Created on: {}", usage.created_on_utc);
        log::debug!("Subscription valid until: {}", usage.valid_until_utc);
        log::debug!("Bandwidth used: {}GB", usage.bandwidth_used_gb);
        log::debug!("Allowance: {}GB", usage.bandwidth_allowance_gb);
    }

    log::info!("🏁 🏁 🏁 Passed!");

    Ok(())
}

#[derive(Debug, PartialEq)]
enum ExpectedTunnelState {
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

async fn wait_for_tunnel_state(
    nym_proxy_client: &mut NymProxyClient,
    expected_state: ExpectedTunnelState,
) -> anyhow::Result<()> {
    let timeout = Duration::from_secs(60);
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

async fn wait_for_account_state(
    nym_proxy_client: &mut NymProxyClient,
    expected_state: AccountControllerState,
) -> anyhow::Result<()> {
    let timeout = Duration::from_secs(60);
    let started = Instant::now();

    loop {
        let current_state = nym_proxy_client.get_account_state().await?;
        if current_state == expected_state {
            log::debug!("✅ Account state {current_state} reached!");
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
    rpc: &NymServiceClient,
    rpc_provider: &RpcClientProvider,
) -> anyhow::Result<NymProxyClient> {
    // Check if daemon should be restarted
    // let mut nym_client = ensure_daemon_version_nym(rpc, rpc_provider)
    //     .await
    //     .context("Failed to restart daemon")?;

    let mut nym_client = rpc_provider.new_client_nym().await;
    // log::debug!("Resetting daemon settings before test...");
    // helpers_nym::disconnect_and_wait(&mut nym_client)
    //     .await
    //     .context("Failed to disconnect daemon after test")?;
    log::debug!("Resetting device identity...");
    nym_client
        .reset_device_identity(None)
        .await
        .context("Failed to reset settings")?;
    log::debug!("Ensuring account is logged in to nym-vpnd...");
    helpers_nym::ensure_logged_in(&mut nym_client).await?;

    log::debug!("Preparing daemon success ! ! !");
    Ok(nym_client)
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
