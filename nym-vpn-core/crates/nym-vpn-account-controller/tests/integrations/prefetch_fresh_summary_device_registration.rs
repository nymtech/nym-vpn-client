// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Regression: prefetch must use a network-fresh summary for device registration.
//! A cached summary with `is_device_active=true` must not skip registration when the
//! API reports the device inactive/revoked.

use std::sync::Arc;

use nym_vpn_account_controller::{
    DeviceRegistrationReadiness, LocalSyncCheck, PrefetchZkNymOutcome, classify_local_sync,
    device_registration_readiness, prefetch_zk_nyms, register_device_if_needed,
};
use nym_vpn_api_client::{
    api_urls_to_urls,
    types::{Device, VpnAccount},
};
use nym_vpn_lib_types::{StoredAccountMode, VpnAccountSummary};
use nym_vpn_store::{
    account_summary::{AccountSummaryStorage, on_disk::OnDiskAccountSummaryStorage},
    keys::device::DeviceKeys,
};
use wiremock::MockServer;

use crate::common::{
    account_summary::{
        account_ready_to_connect, account_with_unregistered_device, mock_api_device,
    },
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

#[tokio::test]
async fn prefetch_registers_device_when_stale_cache_would_skip() -> anyhow::Result<()> {
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

    let stale_summary =
        VpnAccountSummary::from_parts(&account_ready_to_connect(), account.mode(), remote_time)?;
    assert!(
        stale_summary.is_device_active,
        "stale cache fixture must look registered"
    );
    assert_eq!(
        device_registration_readiness(&stale_summary)?,
        DeviceRegistrationReadiness::AlreadyRegistered
    );

    let tempdir = tempfile::tempdir()?;
    let data_dir = tempdir.path().to_path_buf();
    let summary_store = OnDiskAccountSummaryStorage::new(data_dir.join("account_summary.json"));
    summary_store
        .store_summary(stale_summary.clone())
        .await
        .expect("persist stale cache");
    let cached = summary_store
        .load_summary()
        .await?
        .expect("stale cache round-trip");
    assert!(cached.is_device_active);

    let fresh_api = vpn_api_client
        .get_account_summary_with_device(&account, &device)
        .await?;
    let mut summary = VpnAccountSummary::from_parts(&fresh_api, account.mode(), remote_time)?;
    assert!(
        !summary.is_device_active,
        "network sync must report inactive device even when cache says active"
    );
    assert_eq!(
        classify_local_sync(&summary),
        LocalSyncCheck::MustRegisterDevice
    );

    if matches!(
        classify_local_sync(&summary),
        LocalSyncCheck::MustRegisterDevice
    ) {
        register_device_if_needed(&vpn_api_client, &account, &device, &mut summary).await?;
        summary_store.store_summary(summary.clone()).await?;
    }

    let outcome = prefetch_zk_nyms(
        data_dir,
        vpn_api_client,
        Arc::new(account),
        device,
        summary.fair_usage_left(),
    )
    .await?;

    assert_eq!(outcome, PrefetchZkNymOutcome::FetchedTickets);

    let requests = server.received_requests().await.expect("wiremock requests");
    let register_idx = post_register_device_index(&requests).expect("device registration POST");
    let zknym_idx = requests
        .iter()
        .position(|req| req.method.as_str() == "POST" && req.url.path().ends_with("/zknym"))
        .expect("zk-nym POST");
    assert!(
        register_idx < zknym_idx,
        "fresh summary must drive register_device before zk-nym despite stale cache"
    );

    Ok(())
}
