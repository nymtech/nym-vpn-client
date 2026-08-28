// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! End-to-end tests of the zk-nym request flow against a mock VPN API, covering the request-storm
//! regression from incident NYM-APAC-2026-001: an interrupted fetch must resume its pending
//! zk-nym request on the server instead of issuing a fresh one, because every abandoned request
//! counts against the per-device rate limiter.

mod common;

use std::{sync::Arc, time::Duration};

use nym_bandwidth_controller::CredentialFetcher;
use nym_credentials_interface::TicketType;
use nym_crypto::asymmetric::ed25519;
use nym_vpn_api_client::{
    VpnApiClient, api_urls_to_urls,
    types::{Device, VpnAccount, VpnAccountMode},
};
use nym_vpn_credential_fetcher::VpnApiCredentialFetcher;
use tokio_util::sync::CancellationToken;
use wiremock::MockServer;

use common::MockZkNymApi;

const TEST_MNEMONIC: &str = "range mystery picture decline olympic acoustic lesson quick rebuild panda royal fold start leader egg hammer width olympic worry length crawl couch link mobile";

struct TestBench {
    api: MockZkNymApi,
    fetcher: Arc<VpnApiCredentialFetcher>,
    _server: MockServer,
    _data_dir: tempfile::TempDir,
}

async fn setup() -> anyhow::Result<TestBench> {
    let server = MockServer::start().await;
    let api = MockZkNymApi::new()?;
    api.register(&server).await;

    let api_url = nym_network_defaults::ApiUrl {
        url: server.uri(),
        front_hosts: None,
    };
    let client = VpnApiClient::new(
        api_urls_to_urls(&[api_url])?,
        Some(nym_http_api_client::UserAgent {
            application: "nym-vpn-credential-fetcher-tests".to_string(),
            version: "0.0.0".to_string(),
            platform: "test".to_string(),
            git_commit: "0000000".to_string(),
        }),
    )?;

    let account = VpnAccount::new(
        bip39::Mnemonic::parse::<&str>(TEST_MNEMONIC)?,
        VpnAccountMode::Api,
    )?;
    let device = Device::from(ed25519::KeyPair::new(&mut rand::rngs::OsRng));

    let data_dir = tempfile::tempdir()?;
    let fetcher = VpnApiCredentialFetcher::new(
        client,
        Arc::new(account),
        device,
        data_dir.path(),
        CancellationToken::new(),
    )
    .await?;

    Ok(TestBench {
        api,
        fetcher: Arc::new(fetcher),
        _server: server,
        _data_dir: data_dir,
    })
}

/// The NYM-APAC-2026-001 regression: pausing and resuming the fetcher mid-issuance (which the
/// tunnel state machine does on every connect, via the firewall commands) must not create a
/// second zk-nym request on the VPN API — the pending one has to be resumed.
#[tokio::test(flavor = "multi_thread")]
async fn pause_resume_reuses_the_pending_zknym_request() -> anyhow::Result<()> {
    let bench = setup().await?;

    let fetch = tokio::spawn({
        let fetcher = bench.fetcher.clone();
        async move { fetcher.fetch_ticketbooks(TicketType::V1WireguardEntry).await }
    });

    // Wait until the request was posted and polled once (still pending), then interrupt the
    // in-flight fetch the way a connect attempt does.
    bench.api.wait_for_polls(1, Duration::from_secs(10)).await;
    bench.fetcher.pause();
    // Give the runtime a moment to actually drop the in-flight fetch future.
    tokio::time::sleep(Duration::from_millis(250)).await;
    // Once the fetch is running again, let the issuance finish on the next poll.
    bench.api.activate_pending_on_next_poll();
    bench.fetcher.resume();

    let credentials = fetch.await?.expect("fetch should succeed after resuming");
    assert_eq!(credentials.len(), 1);
    assert_eq!(
        bench.api.post_count(),
        1,
        "an interrupted fetch must resume its pending zk-nym request, not post a new one"
    );

    Ok(())
}

/// A pending request the server has marked failed must not be resumed forever: the next fetch
/// discards it and issues a fresh request.
#[tokio::test(flavor = "multi_thread")]
async fn errored_zknym_request_is_discarded_and_a_fresh_one_issued() -> anyhow::Result<()> {
    let bench = setup().await?;

    let fetch = tokio::spawn({
        let fetcher = bench.fetcher.clone();
        async move { fetcher.fetch_ticketbooks(TicketType::V1WireguardEntry).await }
    });

    // Let the first request get posted, then fail it server-side.
    bench.api.wait_for_polls(1, Duration::from_secs(10)).await;
    bench.api.fail_all_requests();
    assert!(
        fetch.await?.is_err(),
        "a server-side issuance failure is terminal for the running fetch"
    );

    // The next fetch must not get stuck on the dead request: fresh request, successful issuance.
    bench.api.activate_pending_on_next_poll();
    let credentials = bench
        .fetcher
        .fetch_ticketbooks(TicketType::V1WireguardEntry)
        .await
        .expect("a fresh fetch should succeed");
    assert_eq!(credentials.len(), 1);
    assert_eq!(
        bench.api.post_count(),
        2,
        "the dead request must be replaced by exactly one fresh request"
    );

    Ok(())
}
