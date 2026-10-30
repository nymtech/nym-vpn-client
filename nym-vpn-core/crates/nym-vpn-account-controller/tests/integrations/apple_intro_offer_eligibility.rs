// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! The Apple introductory-offer eligibility token comes from the VPN API's
//! `subscription/apple/intro-offer-eligibility` route. The request and response bodies used here
//! are the exact shapes that route accepts and returns, so these tests fail if either side of
//! the contract drifts.

use crate::common::{TestBench, account_summary::*, endpoints};
use std::assert_matches;

use nym_vpn_api_client::response::NymErrorResponse;
use nym_vpn_lib_types::{
    AccountCommandError, AccountControllerErrorStateReason, AccountControllerState,
};

const PRODUCT_ID: &str = "1_year_may_2025";
const APP_TRANSACTION_ID: &str = "1000011859217";
const DENYING_JWS: &str = "eyJhbGciOiJFUzI1NiIsImtpZCI6IktFWUlEIiwidHlwIjoiSldUIn0.e30.sig";

fn expected_request_body() -> serde_json::Value {
    serde_json::json!({
        "productId": PRODUCT_ID,
        "transactionId": APP_TRANSACTION_ID,
    })
}

fn account_not_found() -> NymErrorResponse {
    NymErrorResponse {
        message: "Account not found".into(),
        message_id: None,
        code_reference_id: None,
        status: "access_denied".into(),
    }
}

fn signing_not_configured() -> NymErrorResponse {
    NymErrorResponse {
        message: "Apple introductory-offer signing is not configured".into(),
        message_id: None,
        code_reference_id: None,
        status: "error".into(),
    }
}

async fn check(test_bench: &TestBench) -> Result<Option<String>, AccountCommandError> {
    test_bench
        .command_sender
        .check_apple_intro_offer_eligibility(PRODUCT_ID.into(), APP_TRANSACTION_ID.into())
        .await
}

#[tokio::test]
async fn ineligible_account_gets_a_denying_token() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;

    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_with_inactive_sub()),
        endpoints::apple_intro_offer_eligibility_200_raw(
            expected_request_body(),
            format!(r#"{{"trialEligible":false,"compactJws":"{DENYING_JWS}"}}"#),
        ),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::InactiveSubscription,
        ))
        .await;

    assert_eq!(check(&test_bench).await, Ok(Some(DENYING_JWS.into())));
    Ok(())
}

#[tokio::test]
async fn eligible_account_gets_no_token() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;

    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(pending_subscription()),
        endpoints::apple_intro_offer_eligibility_200_raw(
            expected_request_body(),
            r#"{"trialEligible":true,"compactJws":null}"#.into(),
        ),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::PendingSubscription)
        .await;

    assert_eq!(check(&test_bench).await, Ok(None));
    Ok(())
}

#[tokio::test]
async fn unregistered_account_is_eligible_without_a_token() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;

    // The API has never seen this account: both the summary and the eligibility routes answer
    // "Account not found". No account means no history, so no denying token.
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_403(account_not_found()),
        endpoints::apple_intro_offer_eligibility_error(403, account_not_found()),
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

    assert_eq!(check(&test_bench).await, Ok(None));
    Ok(())
}

#[tokio::test]
async fn signing_not_configured_is_an_error_not_a_verdict() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;

    // The API answers 500 while the signing key is not provisioned. That must surface as an
    // error so the app purchases without the option (Apple's rule applies), never as "no token
    // needed".
    let mocks = vec![
        endpoints::synced_health(),
        endpoints::account_summary_with_device_200(account_with_inactive_sub()),
        endpoints::apple_intro_offer_eligibility_error(500, signing_not_configured()),
    ];
    test_bench.register_vpn_api_mocks(mocks).await;

    test_bench.store_mock_account().await?;
    test_bench
        .assert_state(AccountControllerState::Error(
            AccountControllerErrorStateReason::InactiveSubscription,
        ))
        .await;

    let result = check(&test_bench).await;
    assert_matches!(
        result,
        Err(AccountCommandError::VpnApi(_)),
        "expected a VPN API error, got {result:?}"
    );
    Ok(())
}

#[tokio::test]
async fn logged_out_account_cannot_check_eligibility() -> anyhow::Result<()> {
    let test_bench = TestBench::new().await?;
    test_bench
        .register_vpn_api_mocks(vec![endpoints::synced_health()])
        .await;

    assert_eq!(
        check(&test_bench).await,
        Err(AccountCommandError::NoAccountStored)
    );
    Ok(())
}
