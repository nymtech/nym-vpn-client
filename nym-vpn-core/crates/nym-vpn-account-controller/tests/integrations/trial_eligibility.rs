// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Trial eligibility is read from the VPN API's `subscription/active` route. The bodies used
//! here are the exact shapes that route returns, so these tests fail if the Rust response type
//! ever requires a field the API does not send.

use crate::common::{TestBench, account_summary::*, endpoints};

use nym_vpn_api_client::response::NymErrorResponse;
use nym_vpn_lib_types::{
    AccountCommandError, AccountControllerErrorStateReason, AccountControllerState,
};

fn account_not_found() -> NymErrorResponse {
    NymErrorResponse {
        message: "Account not found".into(),
        message_id: None,
        code_reference_id: None,
        status: "access_denied".into(),
    }
}

#[tokio::test]
async fn registered_account_without_history_is_trial_eligible() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;

    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(pending_subscription()),
        endpoints::active_subscription_200_raw(
            r#"{"isSubscriptionActive":false,"trialEligible":true}"#,
        ),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::PendingSubscription)
        .await;

    assert_eq!(
        test_bench.command_sender.is_trial_eligible().await,
        Ok(true)
    );
    Ok(())
}

#[tokio::test]
async fn account_with_subscription_history_is_not_trial_eligible() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;

    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_with_inactive_sub()),
        endpoints::active_subscription_200_raw(
            r#"{"isSubscriptionActive":false,"trialEligible":false}"#,
        ),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::InactiveSubscription,
        ))
        .await;

    assert_eq!(
        test_bench.command_sender.is_trial_eligible().await,
        Ok(false)
    );
    Ok(())
}

#[tokio::test]
async fn active_subscription_is_not_trial_eligible() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;

    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(pending_subscription()),
        endpoints::active_subscription_200_raw(
            r#"{"isSubscriptionActive":true,"subscription":{"id":"sub_1","kind":"one_year","status":"complete","isRecurring":true,"isTrialing":true,"is_freepass_period":false,"valid_from_utc":"2026-10-01 00:00:00.000Z","valid_until_utc":"2027-10-01 00:00:00.000Z","created_on_utc":"2026-10-01 00:00:00.000Z","last_updated_utc":"2026-10-01 00:00:00.000Z"},"trialEligible":false}"#,
        ),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::PendingSubscription)
        .await;

    assert_eq!(
        test_bench.command_sender.is_trial_eligible().await,
        Ok(false)
    );
    Ok(())
}

#[tokio::test]
async fn older_api_without_trial_eligible_field_is_not_trial_eligible() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;

    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(pending_subscription()),
        endpoints::active_subscription_200_raw(r#"{"isSubscriptionActive":false}"#),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::PendingSubscription)
        .await;

    assert_eq!(
        test_bench.command_sender.is_trial_eligible().await,
        Ok(false)
    );
    Ok(())
}

#[tokio::test]
async fn unregistered_account_is_trial_eligible() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;

    // The API has never seen this account: both the summary and the active subscription
    // routes answer "Account not found".
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_403(account_not_found()),
        endpoints::active_subscription_403(account_not_found()),
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

    assert_eq!(
        test_bench.command_sender.is_trial_eligible().await,
        Ok(true)
    );
    Ok(())
}

#[tokio::test]
async fn logged_out_account_has_no_trial_eligibility() -> anyhow::Result<()> {
    let test_bench = TestBench::new().await?;
    test_bench
        .register_vpn_api_mocks(vec![endpoints::synced_health()])
        .await;

    assert_eq!(
        test_bench.command_sender.is_trial_eligible().await,
        Err(AccountCommandError::NoAccountStored)
    );
    Ok(())
}
