// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Regression for the iOS UniFFI prefetch flow (`app_prefetch_zk_nyms_after_fresh_summary`).
//! Stale on-disk summary must not skip device registration when the network sync reports
//! an inactive device.

use std::sync::Arc;

use nym_vpn_account_controller::{
    DeviceRegistrationReadiness, LocalSyncCheck, PrefetchZkNymOutcome,
    app_prefetch_zk_nyms_after_fresh_summary, classify_local_sync, device_registration_readiness,
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

async fn fetch_fresh_summary(
    vpn_api_client: &nym_vpn_api_client::VpnApiClient,
    account: &VpnAccount,
    device: &Device,
) -> anyhow::Result<VpnAccountSummary> {
    let remote_time = vpn_api_client.get_remote_time().await?;
    let api_summary = vpn_api_client
        .get_account_summary_with_device(account, device)
        .await?;
    Ok(VpnAccountSummary::from_parts(
        &api_summary,
        account.mode(),
        remote_time,
    )?)
}

#[tokio::test]
async fn app_prefetch_registers_device_when_stale_cache_would_skip() -> anyhow::Result<()> {
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

    let account = Arc::new(VpnAccount::try_from(mock_account(StoredAccountMode::Api))?);
    let device_keys = DeviceKeys::generate_new(&mut rand::thread_rng());
    let device = Device::from(device_keys.device_keypair().clone());

    let remote_time = vpn_api_client.get_remote_time().await?;
    let stale_summary =
        VpnAccountSummary::from_parts(&account_ready_to_connect(), account.mode(), remote_time)?;
    assert!(stale_summary.is_device_active);
    assert_eq!(
        device_registration_readiness(&stale_summary)?,
        DeviceRegistrationReadiness::AlreadyRegistered
    );

    let tempdir = tempfile::tempdir()?;
    let data_dir = tempdir.path().to_path_buf();
    let summary_store = OnDiskAccountSummaryStorage::new(data_dir.join("account_summary.json"));
    summary_store
        .store_summary(stale_summary)
        .await
        .expect("persist stale cache");

    let fresh_summary = fetch_fresh_summary(&vpn_api_client, account.as_ref(), &device).await?;
    assert!(!fresh_summary.is_device_active);
    assert_eq!(
        classify_local_sync(&fresh_summary),
        LocalSyncCheck::MustRegisterDevice
    );

    let resync_client = vpn_api_client.clone();
    let resync_account = Arc::clone(&account);
    let resync_device = device.clone();

    let outcome = app_prefetch_zk_nyms_after_fresh_summary(
        data_dir.clone(),
        vpn_api_client,
        account,
        device,
        fresh_summary,
        move |summary| {
            let path = data_dir.join("account_summary.json");
            async move {
                OnDiskAccountSummaryStorage::new(path)
                    .store_summary(summary)
                    .await
                    .map_err(|err| nym_vpn_account_controller::Error::Internal(err.to_string()))
            }
        },
        move || {
            let resync_client = resync_client.clone();
            let resync_account = Arc::clone(&resync_account);
            let resync_device = resync_device.clone();
            async move {
                fetch_fresh_summary(&resync_client, resync_account.as_ref(), &resync_device)
                    .await
                    .map_err(|err| nym_vpn_account_controller::Error::Internal(err.to_string()))
            }
        },
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
        "app prefetch must register from fresh summary before zk-nym despite stale cache"
    );

    Ok(())
}
