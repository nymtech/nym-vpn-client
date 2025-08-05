// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use common::account_summary::*;
use common::endpoints::*;
use nym_vpn_lib_types::AccountControllerErrorStateReason;
use nym_vpn_lib_types::AccountControllerState;

use crate::common::mock_mnemonic;

mod common;

#[tokio::test]
async fn offline_test() {
    // Get the test_bench
    let mut test_bench = common::mock_account_controller().await;

    // Simulating offline mode
    test_bench.connectivity.go_offline();

    test_bench
        .assert_state(AccountControllerState::Offline)
        .await;

    test_bench.connectivity.go_online();

    test_bench
        .assert_state(AccountControllerState::LoggedOut)
        .await;
}

#[tokio::test]
async fn offline2_test() {
    // Get the test_bench
    let mut test_bench = common::mock_account_controller().await;

    // Adding behavior to the VPN API
    account_summary_with_device_200(&test_bench.vpn_api_server, account_with_inactive_sub());
    synced_health(&test_bench.vpn_api_server); // this is only valid for some time

    test_bench
        .command_sender
        .store_account(mock_mnemonic())
        .await
        .unwrap();

    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::InactiveSubscription,
        ))
        .await;
}

#[tokio::test]
async fn offline3_test() {
    // Get the test_bench
    let mut test_bench = common::mock_account_controller().await;

    // Adding behaviors to the VPN API
    synced_health(&test_bench.vpn_api_server); // this is only valid for some time
    account_summary_with_device_200(&test_bench.vpn_api_server, account_ready_to_connect());

    test_bench
        .command_sender
        .store_account(mock_mnemonic())
        .await
        .unwrap();

    test_bench.command_sender.forget_account().await.unwrap();

    test_bench
        .assert_state(AccountControllerState::LoggedOut)
        .await;
}
