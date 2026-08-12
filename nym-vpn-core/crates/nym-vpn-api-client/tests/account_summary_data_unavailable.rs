use std::str::FromStr;

use bip39::Mnemonic;
use nym_crypto::asymmetric::ed25519;
use nym_http_api_client::{Url, UserAgent};
use nym_vpn_api_client::{
    VpnApiClient,
    response::NymVpnAccountSummaryWithDeviceResponse,
    types::{Device, VpnAccount, VpnAccountMode},
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path_regex},
};

fn device_from_mnemonic(phrase: &str) -> Device {
    let mnemonic = Mnemonic::parse(phrase).expect("device mnemonic");
    let (entropy, _) = mnemonic.to_entropy_array();
    let seed = &entropy[0..32];
    let signing_key = ed25519::PrivateKey::from_bytes(seed).expect("ed25519 seed");
    let verifying_key = signing_key.public_key();
    let privkey = signing_key.to_bytes().to_vec();
    let pubkey = verifying_key.to_bytes().to_vec();
    let keypair = ed25519::KeyPair::from_bytes(&privkey, &pubkey).expect("keypair");
    Device::from(keypair)
}

#[tokio::test]
async fn account_summary_with_device_round_trips_data_unavailable() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "account": {
            "created_on_utc": "2026-05-04T11:15:14.681Z",
            "last_updated_utc": "2026-05-04T11:15:14.681Z",
            "account_addr": "n1stubstubstubstubstubstubstubstubst",
            "status": "active",
            "canonical_account_addr": "n1stubstubstubstubstubstubstubstubst",
            "auth_methods": [{
                "id": "stub",
                "pubkey": "stub",
                "kind": "user_generated_secp256k1",
                "label": "Stub",
                "status": "active",
                "created": "2026-05-04T11:15:14.727Z",
            }],
        },
        "subscription": {
            "isActive": true,
            "isStacked": false,
            "active": {
                "created_on_utc": "2026-05-04T11:15:51.388Z",
                "last_updated_utc": "2026-05-04T11:15:58.581Z",
                "id": "stub-sub",
                "valid_until_utc": "2099-06-04T11:15:51.000Z",
                "valid_from_utc": "2026-05-04T11:15:51.000Z",
                "status": "active",
                "kind": "one_month",
                "isRecurring": true,
            },
        },
        "devices": { "active": 1, "max": 10, "remaining": 9 },
        "fairUsage": {
            "usedGB": 0,
            "limitGB": 0,
            "dataUnavailable": true,
        },
        "activeDevice": {
            "created_on_utc": "2026-05-04T15:11:56.013Z",
            "last_updated_utc": "2026-05-04T15:11:56.013Z",
            "device_identity_key": "FJDUECYAeosXhNGjxf8w5MJM7N2DfDwQznvWwTxJz6ft",
            "status": "active",
        },
    });

    Mock::given(method("GET"))
        .and(path_regex("^/public/v1/account/.+/device/.+/summary$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let base = format!("{}/", server.uri().trim_end_matches('/'));
    let url = Url::new(&base, None).expect("mock base url");
    let client = VpnApiClient::new(
        vec![url],
        Some(UserAgent::from_str("nym-test/0.1.0/ci/wiremock").expect("user agent")),
    )
    .await
    .expect("vpn api client");

    let account = VpnAccount::new(
        Mnemonic::parse(
            "range mystery picture decline olympic acoustic lesson quick rebuild panda royal fold start leader egg hammer width olympic worry length crawl couch link mobile",
        )
        .expect("account mnemonic"),
        VpnAccountMode::Api,
    )
    .expect("vpn account");

    let device = device_from_mnemonic(
        "pitch deputy proof fire movie put bread ribbon what chef zebra car vacuum gadget steak board state oyster layer glory barely thrive nice box",
    );

    let summary: NymVpnAccountSummaryWithDeviceResponse = client
        .get_account_summary_with_device(&account, &device)
        .await
        .expect("get_account_summary_with_device");

    let fu = &summary.account_summary.fair_usage;
    assert!(fu.data_unavailable, "expected dataUnavailable round-trip");
    assert_eq!(fu.usedGB, 0);
    assert_eq!(fu.limitGB, 0);

    let active = summary
        .active_device
        .as_ref()
        .expect("stub returns activeDevice");
    assert_eq!(
        active.device_identity_key.as_str(),
        device.identity_key().to_string(),
        "mock body device key must match URL segment device"
    );
}
