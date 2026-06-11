// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_account_controller::{
    PrefetchZkNymOutcome, prefetch_zk_nyms, register_device_for_prefetch_if_needed,
};
use nym_vpn_api_client::{
    api_urls_to_urls,
    types::{Device, VpnAccount},
};
use nym_vpn_lib_types::{StoredAccountMode, VpnAccountSummary};
use nym_vpn_store::keys::device::DeviceKeys;
use wiremock::MockServer;

use crate::common::{
    account_summary::{account_with_unregistered_device, mock_api_device},
    credential_proxy::MockCredentialProxy,
    endpoints, init_tracing, mock_account, mock_user_agent,
};

fn post_register_device_index(requests: &[wiremock::Request]) -> Option<usize> {
    requests.iter().position(|req| {
        req.method.as_str() == "POST"
            && req.url.path().ends_with("/device")
            && !req.url.path().contains("/zknym")
    })
}

fn post_zknym_index(requests: &[wiremock::Request]) -> Option<usize> {
    requests
        .iter()
        .position(|req| req.method.as_str() == "POST" && req.url.path().ends_with("/zknym"))
}

/// Mirrors the iOS prefetch path: register the device on the API before issuing zk-nym POSTs.
#[tokio::test]
async fn register_device_precedes_zknym_requests_on_prefetch_path() -> anyhow::Result<()> {
    init_tracing();

    let server = MockServer::start().await;
    let credential_proxy = MockCredentialProxy::new()?;

    let api_url = nym_network_defaults::ApiUrl {
        url: server.uri(),
        front_hosts: None,
    };
    let urls = api_urls_to_urls(&[api_url])?;
    let vpn_api_client = nym_vpn_api_client::VpnApiClient::new(urls, Some(mock_user_agent()))?;

    server.register(endpoints::synced_health()).await;
    server
        .register(endpoints::account_summary_with_device_200(
            account_with_unregistered_device(),
        ))
        .await;
    server
        .register(endpoints::register_account_200(mock_api_device(
            nym_vpn_api_client::response::NymVpnDeviceStatus::Active,
        )))
        .await;
    server
        .register(endpoints::zknym_available_200(credential_proxy.clone()))
        .await;
    server
        .register(endpoints::zknym_post(credential_proxy.clone()))
        .await;
    server
        .register(endpoints::zknym_id(credential_proxy.clone()))
        .await;
    server
        .register(endpoints::partial_verification_key_200(
            credential_proxy.clone(),
        ))
        .await;
    server
        .register(endpoints::confirm_zk_nym_download_by_id_200(
            credential_proxy.clone(),
        ))
        .await;

    let account = VpnAccount::try_from(mock_account(StoredAccountMode::Api))?;
    let device_keys = DeviceKeys::generate_new(&mut rand::thread_rng());
    let device = Device::from(device_keys.device_keypair().clone());

    let remote_time = vpn_api_client.get_remote_time().await?;
    let api_summary = vpn_api_client
        .get_account_summary_with_device(&account, &device)
        .await?;
    let mut summary = VpnAccountSummary::from_parts(&api_summary, account.mode(), remote_time)?;

    register_device_for_prefetch_if_needed(&vpn_api_client, &account, &device, &mut summary)
        .await?;

    assert!(summary.is_device_active);

    let tempdir = tempfile::tempdir()?;
    let outcome = prefetch_zk_nyms(
        tempdir.path().to_path_buf(),
        vpn_api_client,
        Arc::new(account),
        device,
        summary.fair_usage_left(),
    )
    .await?;

    assert_eq!(outcome, PrefetchZkNymOutcome::FetchedTickets);

    let requests = server.received_requests().await.expect("wiremock requests");
    let register_idx = post_register_device_index(&requests).expect("device registration POST");
    let zknym_idx = post_zknym_index(&requests).expect("zk-nym POST");
    assert!(
        register_idx < zknym_idx,
        "expected register_device before request_zknym, got paths: {:?}",
        requests
            .iter()
            .map(|req| format!("{} {}", req.method, req.url.path()))
            .collect::<Vec<_>>()
    );

    Ok(())
}
