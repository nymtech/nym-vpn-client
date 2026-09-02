// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Discovers the current epoch's verified zk-nym ecash signers from the Nyx
//! DKG contract and registers their announce addresses as additional nym-api
//! endpoints, so the general nym-api pool stays in sync with the validator
//! set without app releases.

use nym_endpoint_health::{EndpointClass, EndpointHealthTracker, FailureKind};
use nym_validator_client::{
    QueryHttpRpcNyxdClient,
    nyxd::{
        Config,
        contract_traits::{DkgQueryClient, PagedDkgQueryClient},
    },
};
use nym_vpn_network_config::Network;
use std::sync::Arc;
use url::Url;

/// Normalize an on-chain announce address for use as an HTTP API base url:
/// parse, require http(s), and ensure the path ends with '/'.
fn normalize_announce_address(raw: &str) -> Option<Url> {
    let mut url: Url = raw.trim().parse().ok()?;
    if url.scheme() != "https" && url.scheme() != "http" {
        return None;
    }
    if !url.path().ends_with('/') {
        let path = format!("{}/", url.path());
        url.set_path(&path);
    }
    Some(url)
}

/// Connect a query-only nyxd client to `url` and fetch the current epoch's
/// verified verification-key shares.
async fn fetch_verified_shares(
    config: &Config,
    url: &Url,
) -> Result<Vec<nym_coconut_dkg_common::verification_key::ContractVKShare>, String> {
    let client = QueryHttpRpcNyxdClient::connect(config.clone(), url.as_str())
        .map_err(|err| err.to_string())?;
    let epoch = client
        .get_current_epoch()
        .await
        .map_err(|err| err.to_string())?;
    let shares = client
        .get_all_verification_key_shares(epoch.epoch_id)
        .await
        .map_err(|err| err.to_string())?;
    Ok(shares)
}

/// Query the DKG contract for the current epoch's verified signer set and
/// register their nym-api announce addresses into the tracker's NymApi pool.
/// Tries the healthy nyxd endpoints in order (one pass); returns how many
/// endpoints were registered.
pub async fn discover_ecash_signer_apis(
    network: &Network,
    tracker: &Arc<EndpointHealthTracker>,
) -> Result<usize, String> {
    let network_details = network.nym_network_details();
    let client_config = Config::try_from_nym_network_details(network_details)
        .map_err(|err| format!("invalid network information: {err}"))?;

    let mut candidates = tracker.select(EndpointClass::NyxdRpc);
    if candidates.is_empty() {
        candidates = vec![network.nyxd_url()];
    }

    let mut last_error = "no nyxd endpoints available".to_string();
    for url in candidates {
        match fetch_verified_shares(&client_config, &url).await {
            Ok(shares) => {
                tracker.report_success(EndpointClass::NyxdRpc, &url, None);

                let normalized: Vec<Url> = shares
                    .into_iter()
                    .filter(|share| share.verified)
                    .filter_map(|share| normalize_announce_address(&share.announce_address))
                    .collect();
                let count = normalized.len();
                tracker.register(EndpointClass::NymApi, normalized);
                tracing::info!(
                    "discovered {count} verified ecash signer nym-api endpoint(s) from the DKG contract at {url}"
                );
                return Ok(count);
            }
            Err(err) => {
                tracing::warn!(
                    endpoint = %url,
                    "nyxd endpoint failed during signer discovery, trying next: {err}"
                );
                tracker.report_failure(EndpointClass::NyxdRpc, &url, FailureKind::Connect);
                last_error = err;
            }
        }
    }
    Err(last_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_adds_trailing_slash_when_missing() {
        let url = normalize_announce_address("https://x.example/api").unwrap();
        assert_eq!(url.as_str(), "https://x.example/api/");
    }

    #[test]
    fn normalize_keeps_already_trailing_slash_unchanged() {
        let url = normalize_announce_address("https://x.example/").unwrap();
        assert_eq!(url.as_str(), "https://x.example/");
    }

    #[test]
    fn normalize_rejects_non_http_scheme() {
        assert!(normalize_announce_address("ftp://x.example/").is_none());
    }

    #[test]
    fn normalize_rejects_garbage() {
        assert!(normalize_announce_address("not a url at all").is_none());
    }

    /// Live network test: queries the real mainnet DKG contract for the
    /// current epoch's verified ecash signers and registers their nym-api
    /// announce addresses into a fresh tracker. Run manually:
    /// `cargo test -p nym-vpn-account-controller --lib -- --ignored --nocapture signer`
    #[tokio::test]
    #[ignore = "live network test; run manually"]
    async fn live_discover_ecash_signer_apis_against_mainnet() {
        let network = Network::mainnet_default().unwrap();
        let tracker = Arc::new(EndpointHealthTracker::new());

        let count = discover_ecash_signer_apis(&network, &tracker)
            .await
            .expect("live DKG signer discovery should succeed against mainnet");

        println!("discovered {count} verified ecash signer nym-api endpoints");
        assert!(
            count >= 10,
            "expected at least 10 verified signers, got {count}"
        );
        assert!(
            tracker.all_endpoints(EndpointClass::NymApi).len() >= count,
            "tracker should have registered at least the discovered endpoints"
        );
    }
}
