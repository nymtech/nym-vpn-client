// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::common::{TestBench, account_summary::*, endpoints, nyxd_endpoints};

use nym_vpn_api_client::response::NymVpnDeviceStatus;
use nym_vpn_lib_types::{AccountControllerErrorStateReason, AccountControllerState};

/// How to use these tests :
///
/// 1. Create a TestBench object with TestBench::new.
/// This will spawn an AccountController that will be stopped on drop. It has a mock API and offline monitor and everything
/// 2. Use TestBench::register_mocks to give behaviors to the mock VPN API.
/// Mock are in common::endpoints and some response helpers can be find in the common crate as well
/// 3. Run your tests by giving commands to the TestBench (e.g. store_mock_account, go_offline)
///
/// 4. Use TestBench::assert_state to test the AC state. This takes care of yielding back to the tokio and wait a certain time for the state we're looking for

#[tokio::test]
async fn offline_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = TestBench::new().await?;
    let credential_proxy = test_bench.credential_proxy.clone();

    // Adding behavior to the VPN API
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_ready_to_connect()),
        endpoints::register_account_200(mock_api_device(NymVpnDeviceStatus::Active)),
        endpoints::zknym_available_200(credential_proxy.clone()),
        endpoints::zknym_post(credential_proxy.clone()),
        endpoints::zknym_id(credential_proxy.clone()),
        endpoints::partial_verification_key_200(credential_proxy.clone()),
        endpoints::confirm_zk_nym_download_by_id_200(credential_proxy.clone()),
        endpoints::account_update_device_200(mock_api_device(NymVpnDeviceStatus::DeleteMe)),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    // Simulating offline mode
    test_bench.go_offline()?;
    test_bench
        .assert_state(AccountControllerState::Offline)
        .await;

    test_bench.go_online()?;
    test_bench
        .assert_state(AccountControllerState::LoggedOut)
        .await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::Syncing)
        .await;
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;

    test_bench.go_offline()?;
    test_bench
        .assert_state(AccountControllerState::Offline)
        .await;

    test_bench.go_online()?;
    test_bench
        .assert_state(AccountControllerState::Syncing)
        .await;
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;

    test_bench.forget_account().await?;
    test_bench
        .assert_state(AccountControllerState::LoggedOut)
        .await;
    Ok(())
}

#[tokio::test]
async fn api_error_reponse_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = TestBench::new().await?;

    // Adding behaviors to the VPN API
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_403(unrelated_error()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    // Commands
    test_bench.store_mock_account().await?;

    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::ApiFailure {
                context: "SYNCING_NETWORK_STATE".into(),
                details: "API returned an error: 55cbd0ee-4ff5-4f3d-930e-6f6a95ce849f".into(),
            },
        ))
        .await;
    Ok(())
}

#[tokio::test]
async fn unregistered_account_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = TestBench::new().await?;

    // Adding behavior to the VPN API
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_403(unregistered_account()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;

    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::AccountStatusNotActive {
                status: "unregistered".into(),
            },
        ))
        .await;
    Ok(())
}

#[tokio::test]
async fn desynced_device_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = TestBench::new().await?;

    // Adding behavior to the VPN API
    let mocks = vec![
        endpoints::desynced_health(),
        endpoints::account_summary_with_device_200(account_ready_to_connect()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;

    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::DeviceTimeDesynced,
        ))
        .await;
    Ok(())
}

#[tokio::test]
async fn inactive_account_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = TestBench::new().await?;

    // Adding behavior to the VPN API
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(inactive_account()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;

    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::AccountStatusNotActive {
                status: "Inactive".into(),
            },
        ))
        .await;
    Ok(())
}

#[tokio::test]
async fn account_with_pending_sub_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = TestBench::new().await?;

    // Adding behavior to the VPN API
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(pending_subscription()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;

    test_bench
        .assert_state(AccountControllerState::PendingSubscription)
        .await;
    Ok(())
}

#[tokio::test]
async fn account_with_inactive_sub_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = TestBench::new().await?;

    // Adding behavior to the VPN API
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_with_inactive_sub()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;

    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::InactiveSubscription,
        ))
        .await;
    Ok(())
}

#[tokio::test]
async fn account_with_max_device_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = TestBench::new().await?;

    // Adding behavior to the VPN API
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_max_devices()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;

    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::MaxDeviceReached,
        ))
        .await;
    Ok(())
}

#[tokio::test]
async fn account_with_no_fair_usage_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = TestBench::new().await?;

    // Adding behavior to the VPN API
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_no_fair_usage()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;

    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::BandwidthExceeded {
                context: "SYNCING_LOCAL_STATE".into(),
            },
        ))
        .await;
    Ok(())
}

#[tokio::test]
async fn data_unavailable_skips_fair_usage_depleted_test() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;
    let credential_proxy = test_bench.credential_proxy.clone();

    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_data_unavailable_exhausted_quota()),
        endpoints::register_account_200(mock_api_device(NymVpnDeviceStatus::Active)),
        endpoints::zknym_available_200(credential_proxy.clone()),
        endpoints::zknym_post(credential_proxy.clone()),
        endpoints::zknym_id(credential_proxy.clone()),
        endpoints::partial_verification_key_200(credential_proxy.clone()),
        endpoints::confirm_zk_nym_download_by_id_200(credential_proxy.clone()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;

    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;
    Ok(())
}

#[tokio::test]
async fn zk_nym_issuance_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = TestBench::new().await?;

    let credential_proxy = test_bench.credential_proxy.clone();

    // Adding behavior to the VPN API
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_ready_to_connect()),
        endpoints::zknym_available_200(credential_proxy.clone()),
        endpoints::zknym_post(credential_proxy.clone()),
        endpoints::zknym_id(credential_proxy.clone()),
        endpoints::partial_verification_key_200(credential_proxy.clone()),
        endpoints::confirm_zk_nym_download_by_id_200(credential_proxy.clone()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    // Commands start there
    test_bench.store_mock_account().await?;

    // Resulting state
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;
    Ok(())
}

#[tokio::test]
async fn e2e_new_device_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = TestBench::new().await?;

    let credential_proxy = test_bench.credential_proxy.clone();

    // Adding behavior to the VPN API
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_with_unregistered_device()),
        endpoints::register_account_200(mock_api_device(NymVpnDeviceStatus::Active)),
        endpoints::zknym_available_200(credential_proxy.clone()),
        endpoints::zknym_post(credential_proxy.clone()),
        endpoints::zknym_id(credential_proxy.clone()),
        endpoints::partial_verification_key_200(credential_proxy.clone()),
        endpoints::confirm_zk_nym_download_by_id_200(credential_proxy.clone()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    // Commands start there
    test_bench.store_mock_account().await?;

    test_bench.go_offline()?;
    test_bench
        .assert_state(AccountControllerState::Offline)
        .await;

    test_bench.go_online()?;
    test_bench
        .assert_state(AccountControllerState::Syncing)
        .await;

    // Resulting state
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;
    Ok(())
}

#[tokio::test]
async fn e2e_register_device_conflict_already_active_succeeds() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;
    let credential_proxy = test_bench.credential_proxy.clone();

    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_with_unregistered_device()),
        endpoints::register_account_403(unrelated_error()),
        endpoints::get_device_by_id_200(mock_api_device(NymVpnDeviceStatus::Active)),
        endpoints::zknym_available_200(credential_proxy.clone()),
        endpoints::zknym_post(credential_proxy.clone()),
        endpoints::zknym_id(credential_proxy.clone()),
        endpoints::partial_verification_key_200(credential_proxy.clone()),
        endpoints::confirm_zk_nym_download_by_id_200(credential_proxy.clone()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;
    Ok(())
}

#[tokio::test]
async fn e2e_register_device_unique_constraint_without_get_succeeds() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;
    let credential_proxy = test_bench.credential_proxy.clone();

    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_with_unregistered_device()),
        endpoints::register_account_403(register_device_unique_constraint_error()),
        endpoints::get_device_by_id_404(),
        endpoints::zknym_available_200(credential_proxy.clone()),
        endpoints::zknym_post(credential_proxy.clone()),
        endpoints::zknym_id(credential_proxy.clone()),
        endpoints::partial_verification_key_200(credential_proxy.clone()),
        endpoints::confirm_zk_nym_download_by_id_200(credential_proxy.clone()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;
    Ok(())
}

#[tokio::test]
async fn decentralised_account_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = TestBench::new().await?;

    let mocks = vec![nyxd_endpoints::get_account_exists()];
    test_bench.register_nyxd_mocks(mocks).await;

    // Simulating offline mode
    test_bench.go_offline()?;
    test_bench
        .assert_state(AccountControllerState::Offline)
        .await;

    test_bench.go_online()?;
    test_bench
        .assert_state(AccountControllerState::LoggedOut)
        .await;

    test_bench.store_mock_decentralised_account().await?;

    test_bench
        .assert_state(AccountControllerState::Decentralised)
        .await;

    test_bench.go_offline()?;
    test_bench
        .assert_state(AccountControllerState::Offline)
        .await;

    test_bench.go_online()?;
    test_bench
        .assert_state(AccountControllerState::Decentralised)
        .await;

    test_bench.forget_account().await?;
    test_bench
        .assert_state(AccountControllerState::LoggedOut)
        .await;
    Ok(())
}

/// A force refresh drops the cached summary and does a mandatory re-fetch, so a server-side change
/// (here: the account going inactive) is picked up and surfaced.
#[tokio::test]
async fn force_refresh_detects_inactive_account_test() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;
    let credential_proxy = test_bench.credential_proxy.clone();

    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_ready_to_connect()),
        endpoints::register_account_200(mock_api_device(NymVpnDeviceStatus::Active)),
        endpoints::zknym_available_200(credential_proxy.clone()),
        endpoints::zknym_post(credential_proxy.clone()),
        endpoints::zknym_id(credential_proxy.clone()),
        endpoints::partial_verification_key_200(credential_proxy.clone()),
        endpoints::confirm_zk_nym_download_by_id_200(credential_proxy.clone()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;

    // The account becomes inactive server-side.
    test_bench.vpn_api_server.reset().await;
    test_bench
        .register_vpn_api_mocks(vec![
            endpoints::synced_health(),
            endpoints::account_summary_with_device_200(inactive_account()),
        ])
        .await;

    assert_eq!(
        test_bench.command_sender.refresh_account_state(true).await,
        Ok(())
    );
    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::AccountStatusNotActive {
                status: "Inactive".into(),
            },
        ))
        .await;
    Ok(())
}

/// When the VPN API is unreachable, an (optimistic) refresh falls back to the cached summary and
/// stays ready, whereas a force refresh - which must obtain fresh data - surfaces the API error.
#[tokio::test]
async fn optimistic_refresh_falls_back_but_force_refresh_surfaces_error_test() -> anyhow::Result<()>
{
    let mut test_bench = TestBench::new().await?;
    let credential_proxy = test_bench.credential_proxy.clone();

    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_ready_to_connect()),
        endpoints::register_account_200(mock_api_device(NymVpnDeviceStatus::Active)),
        endpoints::zknym_available_200(credential_proxy.clone()),
        endpoints::zknym_post(credential_proxy.clone()),
        endpoints::zknym_id(credential_proxy.clone()),
        endpoints::partial_verification_key_200(credential_proxy.clone()),
        endpoints::confirm_zk_nym_download_by_id_200(credential_proxy.clone()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;

    // The account summary endpoint starts failing.
    test_bench.vpn_api_server.reset().await;
    test_bench
        .register_vpn_api_mocks(vec![
            endpoints::synced_health(),
            endpoints::account_summary_with_device_403(unrelated_error()),
        ])
        .await;

    // Optimistic refresh: the fetch fails, but we fall back to the cached summary and stay ready.
    assert_eq!(
        test_bench.command_sender.refresh_account_state(false).await,
        Ok(())
    );
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;

    // Force refresh: no cache fallback, so the API error is surfaced (after exhausting retries).
    assert_eq!(
        test_bench.command_sender.refresh_account_state(true).await,
        Ok(())
    );
    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::ApiFailure {
                context: "SYNCING_NETWORK_STATE".into(),
                details: "API returned an error: 55cbd0ee-4ff5-4f3d-930e-6f6a95ce849f".into(),
            },
        ))
        .await;
    Ok(())
}
