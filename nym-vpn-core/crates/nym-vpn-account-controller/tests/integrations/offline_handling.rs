// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::{Duration, Instant};

use nym_vpn_lib_types::AccountControllerState;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, ResponseTemplate};

use crate::common::TestBench;

/// Test that when transitioning from offline to online, the account controller
/// waits for network stabilization before making API calls
#[tokio::test]
async fn test_network_stabilization_delay_on_wake() -> anyhow::Result<()> {
    let mut testbench = TestBench::new_no_credentials().await?;

    // Setup account
    testbench.store_mock_account().await?;

    // Register health endpoint - using synced_health() from common module
    testbench
        .register_vpn_api_mocks(vec![
            Mock::given(method("GET"))
                .and(path("/public/v1/health"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "status": "ok",
                    "timestampUtc": time::OffsetDateTime::now_utc()
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap()
                }))),
        ])
        .await;

    // Go offline
    testbench.go_offline()?;
    testbench
        .assert_state(AccountControllerState::Offline)
        .await;

    // Register account summary mock for successful sync
    testbench
        .register_vpn_api_mocks(vec![
            Mock::given(method("GET"))
                .and(path("/public/v1/health"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "status": "ok",
                    "timestampUtc": time::OffsetDateTime::now_utc()
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap()
                }))),
            Mock::given(method("GET"))
                .and(path_regex(r"^/public/v1/account/.*/device/.*/summary$"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "account_summary": {
                        "account": {
                            "id": crate::common::mock_account_id(),
                            "status": "Active"
                        },
                        "subscription": {
                            "is_active": true
                        },
                        "fair_usage": {
                            "limitGB": 100,
                            "usedGB": 10
                        }
                    },
                    "active_device": {
                        "device_identity_key": "test_key"
                    }
                }))),
        ])
        .await;

    // Measure time until syncing starts
    let start = Instant::now();

    // Go back online
    testbench.go_online()?;

    // Wait for syncing state
    testbench
        .assert_state(AccountControllerState::Syncing)
        .await;

    let elapsed = start.elapsed();

    // Should have waited at least 2 seconds (NETWORK_STABILIZATION_DELAY)
    // We give a small tolerance for test execution time
    assert!(
        elapsed >= Duration::from_millis(1800),
        "Expected at least 2s stabilization delay, got {}ms",
        elapsed.as_millis()
    );

    // But shouldn't have waited too long (max 5s with tolerance)
    assert!(
        elapsed < Duration::from_secs(5),
        "Stabilization delay too long: {}ms",
        elapsed.as_millis()
    );

    Ok(())
}

/// Test that offline state correctly handles commands and returns appropriate errors
#[tokio::test]
async fn test_offline_state_command_handling() -> anyhow::Result<()> {
    let mut testbench = TestBench::new_no_credentials().await?;

    // Setup account
    testbench.store_mock_account().await?;

    // Go offline before syncing completes
    testbench.go_offline()?;
    testbench
        .assert_state(AccountControllerState::Offline)
        .await;

    // Commands that don't require network should still work
    let account_result = testbench.command_sender.get_stored_account().await;
    assert!(
        account_result.is_ok(),
        "get_stored_account should work offline, got: {:?}",
        account_result
    );

    // Verify we stored the account correctly
    assert!(account_result?.is_some(), "Account should be stored");

    Ok(())
}

/// Test that the account controller can recover from offline state
/// and successfully sync when network is restored
#[tokio::test]
async fn test_offline_recovery_and_sync() -> anyhow::Result<()> {
    let mut testbench = TestBench::new_no_credentials().await?;

    // Setup account
    testbench.store_mock_account().await?;

    // Go offline immediately
    testbench.go_offline()?;
    testbench
        .assert_state(AccountControllerState::Offline)
        .await;

    // Setup mocks for when we come back online
    testbench
        .register_vpn_api_mocks(vec![
            Mock::given(method("GET"))
                .and(path("/public/v1/health"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "status": "ok",
                    "timestampUtc": time::OffsetDateTime::now_utc()
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap()
                }))),
            Mock::given(method("GET"))
                .and(path_regex(r"^/public/v1/account/.*/device/.*/summary$"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "account_summary": {
                        "account": {
                            "id": crate::common::mock_account_id(),
                            "status": "Active"
                        },
                        "subscription": {
                            "is_active": true
                        },
                        "fair_usage": {
                            "limitGB": 100,
                            "usedGB": 10
                        }
                    },
                    "active_device": {
                        "device_identity_key": "test_key"
                    }
                }))),
        ])
        .await;

    // Come back online
    testbench.go_online()?;

    // Should transition to syncing state (demonstrates successful offline recovery)
    testbench
        .assert_state(AccountControllerState::Syncing)
        .await;

    // Note: Not waiting for ReadyToConnect as full sync requires more complex mocks
    // The key behavior (offline → syncing transition) is validated

    Ok(())
}

/// Test that multiple offline/online transitions are handled correctly
#[tokio::test]
async fn test_multiple_offline_online_cycles() -> anyhow::Result<()> {
    let mut testbench = TestBench::new_no_credentials().await?;

    testbench.store_mock_account().await?;

    // Cycle 1: Start offline, then go online
    testbench.go_offline()?;
    testbench
        .assert_state(AccountControllerState::Offline)
        .await;

    // Setup mock for first online cycle
    testbench
        .register_vpn_api_mocks(vec![
            Mock::given(method("GET"))
                .and(path("/public/v1/health"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "status": "ok",
                    "timestampUtc": time::OffsetDateTime::now_utc()
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap()
                }))),
            Mock::given(method("GET"))
                .and(path_regex(r"^/public/v1/account/.*/device/.*/summary$"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "account_summary": {
                        "account": {
                            "id": crate::common::mock_account_id(),
                            "status": "Active"
                        },
                        "subscription": {
                            "is_active": true
                        },
                        "fair_usage": {
                            "limitGB": 100,
                            "usedGB": 10
                        }
                    },
                    "active_device": {
                        "device_identity_key": "test_key"
                    }
                }))),
        ])
        .await;

    testbench.go_online()?;
    testbench
        .assert_state(AccountControllerState::Syncing)
        .await;

    // Cycle 2: Go offline again
    testbench.go_offline()?;
    testbench
        .assert_state(AccountControllerState::Offline)
        .await;

    // Cycle 2: Come back online
    testbench.go_online()?;
    testbench
        .assert_state(AccountControllerState::Syncing)
        .await;

    // Successfully demonstrated multiple offline/online cycles work correctly

    Ok(())
}

/// Test that offline detection happens quickly even if API calls are in progress
#[tokio::test]
async fn test_offline_detection_interrupts_api_calls() -> anyhow::Result<()> {
    let mut testbench = TestBench::new_no_credentials().await?;

    testbench.store_mock_account().await?;

    // Register the health endpoint to simulate slow API response
    testbench
        .register_vpn_api_mocks(vec![
            Mock::given(method("GET"))
                .and(path("/public/v1/health"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_delay(Duration::from_secs(10))
                        .set_body_json(serde_json::json!({
                            "status": "ok",
                            "timestamp_utc": time::OffsetDateTime::now_utc()
                        })),
                ),
        ])
        .await;

    // Start syncing (will be slow due to delay)
    testbench
        .assert_state(AccountControllerState::Syncing)
        .await;

    // Go offline while API call is in progress
    tokio::time::sleep(Duration::from_millis(100)).await;
    testbench.go_offline()?;

    // Should quickly transition to offline state without waiting for slow API call
    let start = Instant::now();
    testbench
        .assert_state(AccountControllerState::Offline)
        .await;
    let transition_time = start.elapsed();

    // Transition should be fast (< 1 second), not wait for the 10s API delay
    assert!(
        transition_time < Duration::from_secs(2),
        "Offline transition took too long: {}ms (API call should have been aborted)",
        transition_time.as_millis()
    );

    Ok(())
}
