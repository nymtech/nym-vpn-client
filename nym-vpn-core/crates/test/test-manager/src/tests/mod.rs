// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod blocking_tests;
mod bridge_tests;
pub mod config_nym;
mod envs;
pub(crate) mod helpers_nym;
pub mod nym_test;
mod test_metadata;
mod tunnel_tests;

use futures::future::BoxFuture;
use itertools::Itertools;
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use std::{ops::Not, time::Duration};
pub use test_metadata::TestMetadata;

use crate::{nym_daemon::RpcClientProvider, tests::config_nym::TEST_CONFIG_NYM};
use test_rpc::{NymServiceClient, meta::Os};

/// Timeout for disconnect waits (and other non-connect tunnel waits that pass an explicit duration).
const WAIT_FOR_TUNNEL_STATE_TIMEOUT: Duration = Duration::from_secs(40);

/// Timeout for `wait_for_tunnel_state` (e.g. Connected after cold zk-nym).
pub(crate) const WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Clone)]
pub struct TestContext {
    pub rpc_provider: RpcClientProvider,
}

pub type TestWrapperFunctionNym = fn(
    TestContext,
    NymServiceClient,
    Option<NymProxyClient>,
) -> BoxFuture<'static, anyhow::Result<()>>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("RPC call failed")]
    Rpc(#[from] test_rpc::Error),

    #[error("The daemon returned an error: {0}")]
    Daemon(String),

    #[error("Nym gRPC client ran into an error: {0}")]
    NymManagementInterface(#[from] nym_vpn_proto::rpc_client::Error),

    #[error("An error occurred: {0}")]
    Other(#[from] anyhow::Error),
}

#[derive(Clone)]
/// An abbreviated version of [`TestMetadata`]
pub struct TestDescription {
    pub name: &'static str,
    pub targets: &'static [Os],
    pub priority: Option<i32>,
}

pub fn should_run_on_os(targets: &[Os], os: Os) -> bool {
    targets.is_empty() || targets.contains(&os)
}

/// Get a list of all tests, sorted by priority.
pub fn get_test_descriptions() -> Vec<TestDescription> {
    let tests: Vec<_> = inventory::iter::<TestMetadata>()
        .map(|test| TestDescription {
            priority: test.priority,
            name: test.name,
            targets: test.targets,
        })
        .sorted_by_key(|test| test.priority)
        .collect_vec();

    // TODO: test app upgrade from a released version to this
    // Since `test_upgrade_app` is not registered with inventory, we need to add it manually
    // let test_upgrade_app = TestDescription {
    //     priority: None,
    //     name: "test_upgrade_app",
    //     targets: &[],
    // };
    // [vec![test_upgrade_app], tests].concat()

    tests
}

/// Return all tests with names matching the input argument. Filters out tests that are skipped for
/// the target platform and `test_upgrade_app`, which is run separately.
pub fn get_filtered_tests(
    specified_tests: &[String],
    skipped_tests: &[String],
) -> Result<Vec<TestMetadata>, anyhow::Error> {
    let mut tests: Vec<_> = inventory::iter::<TestMetadata>().cloned().collect();
    tests.sort_by_key(|test| test.priority.unwrap_or(0));

    // Filter out empty strings that may come from shell expansion
    let specified_tests: Vec<_> = specified_tests.iter().filter(|s| !s.is_empty()).collect();

    let mut tests = if specified_tests.is_empty() {
        // Keep all tests
        tests
    } else {
        specified_tests
            .iter()
            .map(|f| {
                tests
                    .iter()
                    .find(|t| t.name.eq_ignore_ascii_case(f))
                    .cloned()
                    .ok_or(anyhow::anyhow!("Test '{f}' not found"))
            })
            .collect::<Result<_, anyhow::Error>>()?
    };

    tests.retain(|test| {
        skipped_tests
            .iter()
            .any(|skip| skip.eq_ignore_ascii_case(test.name))
            .not()
    });

    tests.retain(|test| should_run_on_os(test.targets, TEST_CONFIG_NYM.os));

    Ok(tests)
}

// TODO dz adjust for nym
// /// Make sure the daemon is installed and logged in and restore settings to the defaults.
// pub async fn prepare_daemon(
//     rpc: &ServiceClient,
//     rpc_provider: &RpcClientProvider,
// ) -> anyhow::Result<MullvadProxyClient> {
//     // Check if daemon should be restarted
//     let mut mullvad_client = ensure_daemon_version(rpc, rpc_provider)
//         .await
//         .context("Failed to restart daemon")?;

//     log::debug!("Resetting daemon settings before test");
//     helpers::disconnect_and_wait(&mut mullvad_client)
//         .await
//         .context("Failed to disconnect daemon after test")?;
//     mullvad_client
//         .reset_settings()
//         .await
//         .context("Failed to reset settings")?;
//     helpers::ensure_logged_in(&mut mullvad_client).await?;

//     Ok(mullvad_client)
// }

//    /// Reset the daemons environment.
//    ///
//    /// Will and restart or reinstall it if necessary.
// async fn ensure_daemon_version(
//     rpc: &ServiceClient,
//     rpc_provider: &RpcClientProvider,
// ) -> anyhow::Result<MullvadProxyClient> {
//     let app_package_filename = &TEST_CONFIG.app_package_filename;

//     let must_reinstall_app =
//         match correct_daemon_version_is_running(rpc_provider.new_client().await).await {
//             Ok(correct_version) => !correct_version,
//             // Failing to reach the daemon is a sign that it is not installed
//             Err(mullvad_management_interface::Error::Rpc(..)) => {
//                 log::debug!("Daemon is not running, attempting to start it");

//                 let failed_starting_daemon = rpc.enable_mullvad_daemon().await.is_err()
//                     || rpc.start_mullvad_daemon().await.is_err();
//                 if failed_starting_daemon {
//                     log::warn!("Failed to start the daemon service");
//                 }
//                 failed_starting_daemon
//             }
//             Err(e) => panic!("Failed to get app version: {e}"),
//         };

//     if must_reinstall_app {
//         // NOTE: Reinstalling the app resets the daemon environment
//         install_app(rpc, app_package_filename, rpc_provider)
//             .await
//             .with_context(|| format!("Failed to install app '{app_package_filename}'"))
//     } else {
//         ensure_daemon_environment(rpc)
//             .await
//             .context("Failed to reset daemon environment")?;

//         Ok(rpc_provider.new_client().await)
//     }
// }

// TODO dz adjust these for nym
// /// Conditionally restart the running daemon
// ///
// /// If the daemon was started with non-standard environment variables, subsequent tests may break
// /// due to assuming a default configuration. In that case, reset the environment variables and
// /// restart.
// pub async fn ensure_daemon_environment(rpc: &ServiceClient) -> Result<(), anyhow::Error> {
//     let current_env = rpc
//         .get_daemon_environment()
//         .await
//         .context("Failed to get daemon env variables")?;
//     let default_env = get_app_env()
//         .await
//         .context("Failed to get daemon default env variables")?;
//     if current_env != default_env {
//         log::debug!(
//             "Restarting daemon due changed environment variables. Values since last test {current_env:?}"
//         );
//         rpc.set_daemon_environment(default_env)
//             .await
//             .context("Failed to restart daemon")?;
//     };
//     Ok(())
// }
//
// /// Checks if daemon is installed with the version specified by `TEST_CONFIG.app_package_filename`
// async fn correct_daemon_version_is_running(
//     mut mullvad_client: MullvadProxyClient,
// ) -> Result<bool, mullvad_management_interface::Error> {
//     let app_package_filename = &TEST_CONFIG.app_package_filename;
//     let expected_version = get_version_from_path(std::path::Path::new(app_package_filename))
//         .unwrap_or_else(|_| panic!("Invalid app version: {app_package_filename}"));
//     let version = mullvad_client.get_current_version().await?;
//     Ok(version == expected_version)
// }
