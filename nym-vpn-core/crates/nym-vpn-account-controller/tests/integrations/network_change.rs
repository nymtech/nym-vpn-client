// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::common::{TestBench, account_summary::*, endpoints, mock_user_agent};

use nym_vpn_api_client::{VpnApiClient, api_urls_to_urls};
use nym_vpn_lib_types::AccountControllerState;
use wiremock::MockServer;

// Account calls made after the network changed go to the new VPN API client.
#[tokio::test]
async fn e2e_account_calls_follow_the_replaced_vpn_api_client() -> anyhow::Result<()> {
    let mut test_bench = TestBench::new().await?;
    let other_host = MockServer::start().await;
    other_host.register(endpoints::synced_health()).await;
    other_host
        .register(endpoints::account_summary_with_device_200(
            pending_subscription(),
        ))
        .await;
    let urls = api_urls_to_urls(&[nym_network_defaults::ApiUrl {
        url: other_host.uri(),
        front_hosts: None,
    }])?;
    test_bench
        .vpn_api_client_sender
        .send_replace(VpnApiClient::new(urls, Some(mock_user_agent()))?);

    test_bench.store_mock_account().await?;

    test_bench
        .assert_state(AccountControllerState::PendingSubscription)
        .await;
    Ok(())
}
