// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_api_client::{
    response::{NymErrorResponse, NymVpnAccountSummaryResponse, NymVpnDevicesResponse},
    types::{Device, VpnApiAccount},
    HttpClientError,
};

use crate::{
    error::Error,
    shared_state::{AccountState, DeviceState, SharedAccountState, SubscriptionState},
};

pub(crate) async fn update_state(
    account: &VpnApiAccount,
    device: &Device,
    account_state: &SharedAccountState,
    vpn_api_client: &nym_vpn_api_client::VpnApiClient,
    last_account_summary: &Arc<tokio::sync::Mutex<Option<NymVpnAccountSummaryResponse>>>,
    last_devices: &Arc<tokio::sync::Mutex<Option<NymVpnDevicesResponse>>>,
) -> Result<(), Error> {
    update_account_state(account, account_state, vpn_api_client, last_account_summary).await?;
    update_device_state(account, device, account_state, vpn_api_client, last_devices).await?;
    Ok(())
}

async fn update_account_state(
    account: &VpnApiAccount,
    account_state: &SharedAccountState,
    vpn_api_client: &nym_vpn_api_client::VpnApiClient,
    last_account_summary: &Arc<tokio::sync::Mutex<Option<NymVpnAccountSummaryResponse>>>,
) -> Result<(), Error> {
    tracing::debug!("Updating account state");
    let response = vpn_api_client.get_account_summary(account).await;

    // Check if the response indicates that we are not registered
    if let Some(403) = &response.as_ref().err().and_then(extract_status_code) {
        tracing::warn!("NymVPN API reports: access denied (403)");
        account_state.set_account(AccountState::NotRegistered).await;
    }

    let account_summary = response.map_err(|source| {
        tracing::warn!("NymVPN API error response: {:?}", source);
        Error::GetAccountSummary {
            base_url: vpn_api_client.current_url().clone(),
            source: Box::new(source),
        }
    })?;

    if last_account_summary
        .lock()
        .await
        .replace(account_summary.clone())
        .as_ref()
        != Some(&account_summary)
    {
        tracing::info!("Account summary: {:#?}", account_summary);
    }

    account_state
        .set_account(AccountState::from(account_summary.account))
        .await;

    account_state
        .set_subscription(SubscriptionState::from(account_summary.subscription))
        .await;

    Ok(())
}

async fn update_device_state(
    account: &VpnApiAccount,
    our_device: &Device,
    account_state: &SharedAccountState,
    vpn_api_client: &nym_vpn_api_client::VpnApiClient,
    last_devices: &Arc<tokio::sync::Mutex<Option<NymVpnDevicesResponse>>>,
) -> Result<(), Error> {
    tracing::debug!("Updating device state");

    let devices = vpn_api_client
        .get_devices(account)
        .await
        .map_err(Error::GetDevices)?;

    if last_devices.lock().await.replace(devices.clone()).as_ref() != Some(&devices) {
        tracing::info!("Registered devices: {}", devices);
    }

    // TODO: pagination
    let found_device = devices
        .items
        .iter()
        .find(|device| device.device_identity_key == our_device.identity_key().to_base58_string());

    let Some(found_device) = found_device else {
        tracing::info!("Our device is not registered");
        account_state.set_device(DeviceState::NotRegistered).await;
        return Ok(());
    };

    account_state
        .set_device(DeviceState::from(found_device.status))
        .await;

    Ok(())
}

fn extract_status_code<E>(err: &E) -> Option<u16>
where
    E: std::error::Error + 'static,
{
    let mut source = err.source();
    while let Some(err) = source {
        if let Some(status) = err
            .downcast_ref::<HttpClientError<NymErrorResponse>>()
            .and_then(extract_status_code_inner)
        {
            return Some(status);
        }
        source = err.source();
    }
    None
}

fn extract_status_code_inner(
    err: &nym_vpn_api_client::HttpClientError<NymErrorResponse>,
) -> Option<u16> {
    match err {
        HttpClientError::EndpointFailure { status, .. } => Some((*status).into()),
        _ => None,
    }
}
