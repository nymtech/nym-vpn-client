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
    matchers::{header_exists, method, path_regex},
};

// this is not a valid mnemonic - its just a test account
const TEST_MNEMONIC: &str = "range mystery picture decline olympic acoustic lesson quick rebuild panda royal fold start leader egg hammer width olympic worry length crawl couch link mobile";

struct HeaderAbsent(&'static str);

impl Match for HeaderAbsent {
    fn matches(&self, request: &Request) -> bool {
        !request.headers.contains_key(self.0)
    }
}

fn client_and_account(server: &MockServer) -> (VpnApiClient, VpnAccount) {
    let base = format!("{}/", server.uri().trim_end_matches('/'));
    let url = Url::new(&base, None).expect("base url");
    let client = VpnApiClient::new(
        vec![url],
        Some(UserAgent::from_str("nym-test/0.1.0/ci/wiremock").expect("user agent")),
    )
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
    let (client, account) = client_and_account(&server);

    Mock::given(method("DELETE"))
        .and(path_regex(r"/public/v1/account/.+/device/.+"))
        .and(header_exists("authorization"))
        .and(HeaderAbsent("x-device-authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let result = client
        .delete_device(&account, "SomeOrphanedDeviceIdentityKey")
        .await;

    assert!(result.is_ok(), "delete_device should succeed: {result:?}");
}
