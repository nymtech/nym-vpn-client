// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::{
    response::NymVpnDevice,
    types::{Device, VpnApiAccount},
};

use crate::{error::Error, shared_state::DeviceState, SharedAccountState};

pub(crate) async fn register_device(
    account: &VpnApiAccount,
    device: &Device,
    account_state: &SharedAccountState,
    vpn_api_client: &nym_vpn_api_client::VpnApiClient,
) -> Result<NymVpnDevice, Error> {
    let response = vpn_api_client
        .register_device(account, device)
        .await
        .inspect(|device_result| {
            tracing::info!("Response: {:#?}", device_result);
            tracing::info!("Device registered: {}", device_result.device_identity_key);
        })
        .map_err(Error::RegisterDevice)?;

    let device_state = DeviceState::from(response.status);
    account_state.set_device(device_state).await;
    Ok(response)
}
