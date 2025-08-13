// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use common::account_summary::*;
use common::endpoints;
use nym_vpn_api_client::response::NymVpnDeviceStatus;
use nym_vpn_lib_types::AccountControllerErrorStateReason;
use nym_vpn_lib_types::AccountControllerState;

mod common;

#[tokio::test]
async fn offline_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = common::mock_account_controller().await?;

    // Adding behavior to the VPN API
    test_bench.register_mock(endpoints::synced_health()).await;
    test_bench
        .register_mock(endpoints::account_summary_with_device_200(
            account_with_inactive_sub(),
        ))
        .await;

    // Simulating offline mode
    test_bench.connectivity.go_offline()?;
    test_bench
        .assert_state(AccountControllerState::Offline)
        .await;

    test_bench.connectivity.go_online()?;
    test_bench
        .assert_state(AccountControllerState::LoggedOut)
        .await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::Syncing)
        .await;

    test_bench.connectivity.go_offline()?;
    test_bench
        .assert_state(AccountControllerState::Offline)
        .await;

    test_bench.connectivity.go_online()?;
    test_bench
        .assert_state(AccountControllerState::Syncing)
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
    let mut test_bench = common::mock_account_controller().await?;
    test_bench.register_mock(endpoints::synced_health()).await;

    // Adding behavior to the VPN API
    test_bench
        .register_mock(endpoints::account_summary_with_device_403(unrelated_error()))
        .await;

    test_bench.store_mock_account().await?;

    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::ApiFailure {
                context: "SYNCING_STATE".into(),
                details: "55cbd0ee-4ff5-4f3d-930e-6f6a95ce849f".into(),
            },
        ))
        .await;
    Ok(())
}

#[tokio::test]
async fn unregistered_account_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = common::mock_account_controller().await?;
    test_bench.register_mock(endpoints::synced_health()).await;

    // Adding behavior to the VPN API
    test_bench
        .register_mock(endpoints::account_summary_with_device_403(
            unregistered_account(),
        ))
        .await;

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
    let mut test_bench = common::mock_account_controller().await?;
    test_bench.register_mock(endpoints::desynced_health()).await;

    // Adding behavior to the VPN API
    test_bench
        .register_mock(endpoints::account_summary_with_device_200(
            account_ready_to_connect(),
        ))
        .await;

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
    let mut test_bench = common::mock_account_controller().await?;
    test_bench.register_mock(endpoints::synced_health()).await;
    // Adding behavior to the VPN API
    test_bench
        .register_mock(endpoints::account_summary_with_device_200(
            inactive_account(),
        ))
        .await;

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
async fn account_with_max_device_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = common::mock_account_controller().await?;
    test_bench.register_mock(endpoints::synced_health()).await;
    // Adding behavior to the VPN API
    test_bench
        .register_mock(endpoints::account_summary_with_device_200(
            account_max_devices(),
        ))
        .await;

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
    let mut test_bench = common::mock_account_controller().await?;
    test_bench.register_mock(endpoints::synced_health()).await;

    // Adding behavior to the VPN API
    test_bench
        .register_mock(endpoints::account_summary_with_device_200(
            account_no_fair_usage(),
        ))
        .await;

    test_bench.store_mock_account().await?;

    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::BandwidthExceeded {
                context: "SYNCING_STATE".into(),
            },
        ))
        .await;
    Ok(())
}

#[tokio::test]
async fn full_test() -> anyhow::Result<()> {
    // Get the test_bench
    let mut test_bench = common::mock_account_controller().await?;

    // Adding behaviors to the VPN API
    test_bench.register_mock(endpoints::synced_health()).await;
    test_bench
        .register_mock(endpoints::account_summary_with_device_200(
            account_ready_to_connect(),
        ))
        .await;
    test_bench.register_mock(endpoints::zknym_200()).await;

    test_bench.store_mock_account().await?;

    // THis shouldn't work wtf?
    test_bench
        .assert_state(AccountControllerState::ReadyToConnect)
        .await;
    Ok(())
}
