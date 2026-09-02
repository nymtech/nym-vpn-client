// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::str::FromStr;

use bip39::Mnemonic;
use nym_http_api_client::{Url, UserAgent};
use nym_vpn_api_client::{
    VpnApiClient,
    types::{VpnAccount, VpnAccountMode},
};
use wiremock::{
    Match, Mock, MockServer, Request, ResponseTemplate,
    matchers::{body_json, header_exists, method, path},
};

// this is not a valid mnemonic - its just a test account
const TEST_MNEMONIC: &str = "range mystery picture decline olympic acoustic lesson quick rebuild panda royal fold start leader egg hammer width olympic worry length crawl couch link mobile";

struct HeaderAbsent(&'static str);

impl Match for HeaderAbsent {
    fn matches(&self, request: &Request) -> bool {
        !request.headers.contains_key(self.0)
    }
}

async fn client_and_account(server: &MockServer) -> (VpnApiClient, VpnAccount) {
    let base = format!("{}/", server.uri().trim_end_matches('/'));
    let url = Url::new(&base, None).expect("base url");
    let client = VpnApiClient::new(
        vec![url],
        Some(UserAgent::from_str("nym-test/0.1.0/ci/wiremock").expect("user agent")),
    )
    .await
    .expect("vpn api client");
    let account = VpnAccount::new(
        Mnemonic::parse(TEST_MNEMONIC).expect("mnemonic"),
        VpnAccountMode::Api,
    )
    .expect("account");
    (client, account)
}

#[tokio::test]
async fn delete_device_uses_account_auth_only() {
    let server = MockServer::start().await;
    let (client, account) = client_and_account(&server).await;
    let device_identity = "SomeOrphanedDeviceIdentityKey";
    let expected_path = format!(
        "/public/v1/account/{}/device/{device_identity}",
        account.id()
    );

    Mock::given(method("PATCH"))
        .and(path(expected_path))
        .and(body_json(serde_json::json!({ "status": "delete_me" })))
        .and(header_exists("authorization"))
        .and(HeaderAbsent("x-device-authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let response = client
        .delete_device(&account, device_identity)
        .await
        .expect("delete_device should succeed");

    assert_eq!(response, serde_json::json!({}));
}

#[tokio::test]
async fn delete_device_propagates_non_success_response() {
    let server = MockServer::start().await;
    let (client, account) = client_and_account(&server).await;

    Mock::given(method("PATCH"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "message": "cleanup failed"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let error = client
        .delete_device(&account, "SomeOrphanedDeviceIdentityKey")
        .await
        .expect_err("non-success response must fail");

    assert!(error.to_string().contains("failed to delete device"));
}
