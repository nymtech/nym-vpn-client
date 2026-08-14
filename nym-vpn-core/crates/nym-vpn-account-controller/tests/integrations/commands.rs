// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::common::{TestBench, account_summary::*, endpoints, mock_account, mock_account_id};

use nym_vpn_api_client::response::NymVpnDeviceStatus;
use nym_vpn_lib_types::{
    AccountCommandError, AccountControllerErrorStateReason, AccountControllerState,
    StoredAccountMode,
};

#[tokio::test]
async fn logged_out_state_command() -> anyhow::Result<()> {
    // Get the test_bench without credential for easier testing
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

    assert_eq!(
        test_bench.command_sender.refresh_account_state(false).await,
        Err(AccountCommandError::NoAccountStored)
    );

    assert_eq!(test_bench.command_sender.forget_account().await, Ok(()));
    assert_eq!(test_bench.command_sender.rotate_keys().await, Ok(()));
    assert_eq!(test_bench.command_sender.get_account_id().await, Ok(None));
    assert_eq!(
        test_bench.command_sender.get_stored_account().await,
        Ok(None)
    );
    assert_eq!(
        test_bench.command_sender.get_device_identity().await,
        Ok(None)
    );

    assert_eq!(
        test_bench.command_sender.get_active_devices().await,
        Err(AccountCommandError::NoAccountStored)
    );
    assert_eq!(
        test_bench.command_sender.get_devices().await,
        Err(AccountCommandError::NoAccountStored)
    );
    assert_eq!(
        test_bench.command_sender.get_usage().await,
        Err(AccountCommandError::NoAccountStored)
    );

    assert_eq!(
        test_bench.command_sender.reset_device_identity(None).await,
        Ok(())
    );

    assert!(
        test_bench
            .command_sender
            .create_account_command()
            .await
            .is_ok()
    );
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;
    test_bench.forget_account().await?; // Resetting back to logged out

    assert!(
        test_bench
            .command_sender
            .store_account(mock_account(StoredAccountMode::Api))
            .await
            .is_ok()
    );
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;

    Ok(())
}

#[tokio::test]
async fn offline_state_command() -> anyhow::Result<()> {
    // Get the test_bench without credential for easier testing
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
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    // Straight up go offline
    test_bench.go_offline()?;
    test_bench
        .assert_state(AccountControllerState::Offline)
        .await;

    assert_eq!(
        test_bench.command_sender.refresh_account_state(false).await,
        Err(AccountCommandError::Offline)
    );

    assert_eq!(test_bench.command_sender.rotate_keys().await, Ok(()));

    assert_eq!(
        test_bench.command_sender.get_active_devices().await,
        Err(AccountCommandError::Offline)
    );
    assert_eq!(
        test_bench.command_sender.get_devices().await,
        Err(AccountCommandError::Offline)
    );
    assert_eq!(
        test_bench.command_sender.get_usage().await,
        Err(AccountCommandError::Offline)
    );

    assert_eq!(
        test_bench.command_sender.reset_device_identity(None).await,
        Err(AccountCommandError::Offline)
    );

    // Offline, no account stored
    assert_eq!(test_bench.command_sender.get_account_id().await, Ok(None));
    assert_eq!(
        test_bench.command_sender.get_stored_account().await,
        Ok(None)
    );
    assert_eq!(
        test_bench.command_sender.get_device_identity().await,
        Ok(None)
    );

    // Offline, but storing an account
    assert!(
        test_bench
            .command_sender
            .store_account(mock_account(StoredAccountMode::Api))
            .await
            .is_ok()
    );
    test_bench
        .assert_state(AccountControllerState::Offline)
        .await;

    assert_eq!(
        test_bench.command_sender.get_account_id().await,
        Ok(Some(mock_account_id()))
    );
    assert_eq!(
        test_bench.command_sender.get_stored_account().await,
        Ok(Some(mock_account(StoredAccountMode::Api)))
    );
    assert!(
        test_bench
            .command_sender
            .get_device_identity()
            .await
            .is_ok(),
    ); // Device ID is random so we can only test that there exists one

    // Forget account to reset back to no account
    test_bench.go_online()?;
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;
    test_bench.forget_account().await?;
    test_bench.go_offline()?;
    test_bench
        .assert_state(AccountControllerState::Offline)
        .await;

    assert!(
        test_bench
            .command_sender
            .create_account_command()
            .await
            .is_ok()
    );
    test_bench
        .assert_state(AccountControllerState::Offline)
        .await;

    assert_eq!(test_bench.command_sender.forget_account().await, Ok(()));
    test_bench
        .assert_state(AccountControllerState::LoggedOut)
        .await;
    assert_eq!(
        test_bench.command_sender.get_stored_account().await,
        Ok(None)
    );

    Ok(())
}

#[tokio::test]
async fn ready_state_command() -> anyhow::Result<()> {
    // Get the test_bench without credential for easier testing
    let mut test_bench = TestBench::new().await?;
    let credential_proxy = test_bench.credential_proxy.clone();

    // Adding behavior to the VPN API
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_ready_to_connect()),
        endpoints::get_usage_200(),
        endpoints::get_devices_200(),
        endpoints::get_active_devices_200(),
        endpoints::register_account_200(mock_api_device(NymVpnDeviceStatus::Active)),
        endpoints::zknym_available_200(credential_proxy.clone()),
        endpoints::zknym_post(credential_proxy.clone()),
        endpoints::zknym_id(credential_proxy.clone()),
        endpoints::partial_verification_key_200(credential_proxy.clone()),
        endpoints::confirm_zk_nym_download_by_id_200(credential_proxy.clone()),
        endpoints::account_update_device_200(mock_api_device(NymVpnDeviceStatus::DeleteMe)),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;

    // A local refresh re-evaluates the cached summary and lands back in the ready state.
    assert_eq!(
        test_bench.command_sender.refresh_account_state(false).await,
        Ok(())
    );
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;

    // A force refresh re-syncs with the VPN API, going through the syncing state.
    assert_eq!(
        test_bench.command_sender.refresh_account_state(true).await,
        Ok(())
    );
    test_bench
        .assert_state(AccountControllerState::Syncing)
        .await;
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;

    assert_eq!(
        test_bench.command_sender.get_account_id().await,
        Ok(Some(mock_account_id()))
    );
    assert_eq!(
        test_bench.command_sender.get_stored_account().await,
        Ok(Some(mock_account(StoredAccountMode::Api)))
    );
    assert!(
        test_bench
            .command_sender
            .get_device_identity()
            .await
            .is_ok()
    );

    assert_eq!(
        test_bench.command_sender.get_active_devices().await,
        Ok(mock_active_devices_response().items)
    );
    assert_eq!(
        test_bench.command_sender.get_devices().await,
        Ok(mock_devices_response().items)
    );
    assert_eq!(
        test_bench.command_sender.get_usage().await,
        Ok(mock_usage_response().items)
    );

    assert_eq!(
        test_bench.command_sender.reset_device_identity(None).await,
        Ok(())
    );
    test_bench
        .assert_state(AccountControllerState::Syncing)
        .await;
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;

    assert_eq!(
        test_bench.command_sender.create_account_command().await,
        Err(AccountCommandError::ExistingAccount)
    );

    assert_eq!(
        test_bench
            .command_sender
            .store_account(mock_account(StoredAccountMode::Api))
            .await,
        Err(AccountCommandError::ExistingAccount)
    );

    assert_eq!(test_bench.command_sender.forget_account().await, Ok(()));
    assert_eq!(test_bench.command_sender.rotate_keys().await, Ok(()));
    test_bench
        .assert_state(AccountControllerState::LoggedOut)
        .await;
    assert_eq!(
        test_bench.command_sender.get_stored_account().await,
        Ok(None)
    );

    Ok(())
}

#[tokio::test]
async fn error_state_command() -> anyhow::Result<()> {
    // Get the test_bench without credential for easier testing
    let mut test_bench = TestBench::new().await?;

    // Adding behavior to the VPN API
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_max_devices()), // Get one that leads to error state
        endpoints::get_usage_200(),
        endpoints::get_devices_200(),
        endpoints::get_active_devices_200(),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::MaxDeviceReached,
        ))
        .await;

    // A local refresh re-evaluates the cached summary and lands back in the error state.
    assert_eq!(
        test_bench.command_sender.refresh_account_state(false).await,
        Ok(())
    );
    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::MaxDeviceReached,
        ))
        .await;

    // A force refresh re-syncs with the VPN API, going through the syncing state.
    assert_eq!(
        test_bench.command_sender.refresh_account_state(true).await,
        Ok(())
    );
    test_bench
        .assert_state(AccountControllerState::Syncing)
        .await;
    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::MaxDeviceReached,
        ))
        .await;

    assert_eq!(
        test_bench.command_sender.get_account_id().await,
        Ok(Some(mock_account_id()))
    );
    assert_eq!(
        test_bench.command_sender.get_stored_account().await,
        Ok(Some(mock_account(StoredAccountMode::Api)))
    );
    assert!(
        test_bench
            .command_sender
            .get_device_identity()
            .await
            .is_ok()
    );

    assert_eq!(
        test_bench.command_sender.get_active_devices().await,
        Ok(mock_active_devices_response().items)
    );
    assert_eq!(
        test_bench.command_sender.get_devices().await,
        Ok(mock_devices_response().items)
    );
    assert_eq!(
        test_bench.command_sender.get_usage().await,
        Ok(mock_usage_response().items)
    );

    assert_eq!(
        test_bench.command_sender.reset_device_identity(None).await,
        Ok(())
    );
    test_bench
        .assert_state(AccountControllerState::Syncing)
        .await;
    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::MaxDeviceReached,
        ))
        .await;

    assert_eq!(
        test_bench.command_sender.create_account_command().await,
        Err(AccountCommandError::ExistingAccount)
    );

    assert_eq!(
        test_bench
            .command_sender
            .store_account(mock_account(StoredAccountMode::Api))
            .await,
        Err(AccountCommandError::ExistingAccount)
    );

    assert_eq!(test_bench.command_sender.forget_account().await, Ok(()));
    assert_eq!(test_bench.command_sender.rotate_keys().await, Ok(()));
    test_bench
        .assert_state(AccountControllerState::LoggedOut)
        .await;

    Ok(())
}

#[tokio::test]
async fn error_state_firewall_down_triggers_sync_test() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;

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

    test_bench.command_sender.set_vpn_api_firewall_up().await?;
    test_bench
        .command_sender
        .set_vpn_api_firewall_down()
        .await?;

    // Any error reason + firewall down triggers optimistic sync (parity with local_state.rs).
    // SyncingLocalState escalates to mandatory if the cache is stale after reset.
    test_bench
        .assert_state(AccountControllerState::Syncing)
        .await;

    Ok(())
}

#[tokio::test]
async fn non_bandwidth_error_state_firewall_down_triggers_sync_test() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;

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

    test_bench.command_sender.set_vpn_api_firewall_up().await?;
    test_bench
        .command_sender
        .set_vpn_api_firewall_down()
        .await?;

    test_bench
        .assert_state(AccountControllerState::Syncing)
        .await;

    Ok(())
}
