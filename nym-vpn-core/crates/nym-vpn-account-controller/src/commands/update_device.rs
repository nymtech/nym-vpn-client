// Copyright 2024 - Nym Technologies SA<contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_api_client::{
    response::NymVpnDevicesResponse,
    types::{Device, VpnApiAccount},
};

use crate::shared_state::{DeviceState, SharedAccountState};

use super::{AccountCommandError, AccountCommandResult};

type PreviousDevicesResponse = Arc<tokio::sync::Mutex<Option<NymVpnDevicesResponse>>>;

pub(crate) struct WaitingUpdateDeviceCommandHandler {
    account_state: SharedAccountState,
    vpn_api_client: nym_vpn_api_client::VpnApiClient,

    previous_devices_response: PreviousDevicesResponse,
}

impl WaitingUpdateDeviceCommandHandler {
    pub(crate) fn new(
        account_state: SharedAccountState,
        vpn_api_client: nym_vpn_api_client::VpnApiClient,
    ) -> Self {
        Self {
            account_state,
            vpn_api_client,
            previous_devices_response: Default::default(),
        }
    }

    pub(crate) fn build(
        &self,
        account: VpnApiAccount,
        device: Device,
    ) -> UpdateDeviceStateCommandHandler {
        let id = uuid::Uuid::new_v4();
        tracing::debug!("Created new update state command handler: {}", id);
        UpdateDeviceStateCommandHandler {
            id,
            account,
            device,
            account_state: self.account_state.clone(),
            vpn_api_client: self.vpn_api_client.clone(),
            previous_devices_response: self.previous_devices_response.clone(),
        }
    }
}

pub(crate) struct UpdateDeviceStateCommandHandler {
    id: uuid::Uuid,
    account: VpnApiAccount,
    device: Device,
    account_state: SharedAccountState,
    vpn_api_client: nym_vpn_api_client::VpnApiClient,

    previous_devices_response: PreviousDevicesResponse,
}

impl UpdateDeviceStateCommandHandler {
    pub(crate) async fn run(self) -> AccountCommandResult {
        AccountCommandResult::UpdateDeviceState(self.run_inner().await)
    }

    fn id_str(&self) -> String {
        format!("{:.8}", self.id.to_string())
    }

    #[tracing::instrument(
        skip(self),
        name = "update_device",
        fields(id = %self.id_str()),
        ret,
        err,
    )]
    async fn run_inner(self) -> Result<DeviceState, AccountCommandError> {
        tracing::debug!("Running update device state command handler: {}", self.id);
        update_state(
            &self.account,
            &self.device,
            &self.account_state,
            &self.vpn_api_client,
            &self.previous_devices_response,
        )
        .await
    }
}

pub(crate) async fn update_state(
    account: &VpnApiAccount,
    device: &Device,
    account_state: &SharedAccountState,
    vpn_api_client: &nym_vpn_api_client::VpnApiClient,
    previous_devices_response: &PreviousDevicesResponse,
) -> Result<DeviceState, AccountCommandError> {
    tracing::info!("Updating device state");

    let devices = vpn_api_client.get_devices(account).await.map_err(|err| {
        nym_vpn_api_client::response::extract_error_response(&err)
            .map(|e| {
                tracing::warn!(
                    "nym-vpn-api reports: message={}, message_id={:?}",
                    e.message,
                    e.message_id
                );
                AccountCommandError::UpdateDeviceEndpointFailure {
                    message_id: e.message_id.clone(),
                    message: e.message.clone(),
                }
            })
            .unwrap_or(AccountCommandError::General(err.to_string()))
    })?;

    if previous_devices_response
        .lock()
        .await
        .replace(devices.clone())
        .as_ref()
        != Some(&devices)
    {
        tracing::info!("Updated devices: {:?}", devices);
    }

    // TODO: pagination
    // In this case it's minor, since the page size is likely an order of magniture larger the the
    // max number of allowed devices
    let found_device = devices
        .items
        .iter()
        .find(|d| d.device_identity_key == device.identity_key().to_base58_string());

    let new_device_state = if let Some(found_device) = found_device {
        DeviceState::from(found_device.status)
    } else {
        tracing::info!("Our device is not registered");
        DeviceState::NotRegistered
    };

    account_state.set_device(new_device_state.clone()).await;
    Ok(new_device_state)
}
