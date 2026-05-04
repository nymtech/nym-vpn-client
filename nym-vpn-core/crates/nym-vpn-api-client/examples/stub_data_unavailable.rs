//! Fetch account summary from a local HTTP stub that returns `fairUsage.dataUnavailable: true`.
//!
//! 1. Start the stub: `python3 nym-vpn-core/scripts/dev_mock_vpn_api_data_unavailable.py`
//! 2. Run: `cargo run -p nym-vpn-api-client --example stub_data_unavailable`
//!
//! Optional: `MOCK_VPN_API_BASE=http://127.0.0.1:PORT/api/` (must end with `/api/`).
//! `UserAgent::from_str` requires four segments: `app/version/platform/git_commit`.

use std::str::FromStr;

use bip39::Mnemonic;
use nym_crypto::asymmetric::ed25519;
use nym_http_api_client::{Url, UserAgent};
use nym_vpn_api_client::{
    VpnApiClient,
    response::NymVpnAccountSummaryWithDeviceResponse,
    types::{Device, VpnAccount, VpnAccountMode},
};

/// Same derivation as `Device: From<bip39::Mnemonic>` in tests (not public API).
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let base = std::env::var("MOCK_VPN_API_BASE")
        .unwrap_or_else(|_| "http://127.0.0.1:18080/api/".to_owned());
    let url = Url::new(base.as_str(), None).map_err(|e| format!("{e:?}"))?;
    let client = VpnApiClient::new(
        vec![url],
        Some(UserAgent::from_str("nym-stub-e2e/0.1.0/dev/localstub")?),
    )?;

    let account = VpnAccount::new(
        // this is a non valid mnemonic, but it's used to test the stub data unavailable case
        Mnemonic::parse(
            "range mystery picture decline olympic acoustic lesson quick rebuild panda royal fold start leader egg hammer width olympic worry length crawl couch link mobile",
        )?,
        VpnAccountMode::Api,
    )?;

    let device = device_from_mnemonic(
        // same as above this is a non valid mnemonic, but it's used to test the stub data unavailable case
        "pitch deputy proof fire movie put bread ribbon what chef zebra car vacuum gadget steak board state oyster layer glory barely thrive nice box",
    );

    let summary: NymVpnAccountSummaryWithDeviceResponse = client
        .get_account_summary_with_device(&account, &device)
        .await?;

    let fu = &summary.account_summary.fair_usage;
    println!("fair_usage.usedGB = {}", fu.usedGB);
    println!("fair_usage.limitGB = {}", fu.limitGB);
    println!("fair_usage.data_unavailable = {}", fu.data_unavailable);

    if !fu.data_unavailable {
        return Err("expected data_unavailable == true from stub".into());
    }

    println!("OK: serde + HTTP round-trip sees data_unavailable on active subscription.");
    Ok(())
}
