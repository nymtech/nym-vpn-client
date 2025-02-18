// Copyright 2024 - Nym Technologies SA<contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_api_client::{
    response::NymVpnDevicesResponse,
    types::{Device, VpnApiAccount},
};
use nym_vpn_lib_types::{SyncDeviceError, VpnApiErrorResponse};
use tracing::Level;

use crate::shared_state::{DeviceState, SharedAccountState};

use super::AccountCommandResult;

type PreviousDevicesResponse = Arc<tokio::sync::Mutex<Option<NymVpnDevicesResponse>>>;

pub(crate) struct WaitingSyncDeviceCommandHandler {
    account_state: SharedAccountState,
    vpn_api_client: nym_vpn_api_client::VpnApiClient,

    previous_devices_response: PreviousDevicesResponse,
}

impl WaitingSyncDeviceCommandHandler {
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
    ) -> SyncDeviceStateCommandHandler {
        let id = uuid::Uuid::new_v4();
        tracing::debug!("Created new sync state command handler: {}", id);
        SyncDeviceStateCommandHandler {
            id,
            account,
            device,
            account_state: self.account_state.clone(),
            vpn_api_client: self.vpn_api_client.clone(),
            previous_devices_response: self.previous_devices_response.clone(),
        }
    }

    pub(crate) fn update_vpn_api_client(
        &mut self,
        vpn_api_client: nym_vpn_api_client::VpnApiClient,
    ) {
        self.vpn_api_client = vpn_api_client;
    }
}

pub(crate) struct SyncDeviceStateCommandHandler {
    id: uuid::Uuid,
    account: VpnApiAccount,
    device: Device,
    account_state: SharedAccountState,
    vpn_api_client: nym_vpn_api_client::VpnApiClient,

    previous_devices_response: PreviousDevicesResponse,
}

impl SyncDeviceStateCommandHandler {
    pub(crate) async fn run(self) -> AccountCommandResult {
        AccountCommandResult::SyncDeviceState(self.run_inner().await)
    }

    fn id_str(&self) -> String {
        format!("{:.8}", self.id.to_string())
    }

    #[tracing::instrument(
        skip(self),
        name = "sync_device",
        fields(id = %self.id_str()),
        ret,
        err,
        level = Level::DEBUG,
    )]
    async fn run_inner(self) -> Result<DeviceState, SyncDeviceError> {
        tracing::debug!("Running sync device state command handler: {}", self.id);
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

async fn update_state(
    account: &VpnApiAccount,
    device: &Device,
    account_state: &SharedAccountState,
    vpn_api_client: &nym_vpn_api_client::VpnApiClient,
    previous_devices_response: &PreviousDevicesResponse,
) -> Result<DeviceState, SyncDeviceError> {
    tracing::debug!("Updating device state");

    let devices = vpn_api_client.get_devices(account).await.map_err(|err| {
        VpnApiErrorResponse::try_from(err)
            .map(SyncDeviceError::SyncDeviceEndpointFailure)
            .unwrap_or_else(SyncDeviceError::unexpected_response)
    })?;

    if previous_devices_response
        .lock()
        .await
        .replace(devices.clone())
        .as_ref()
        != Some(&devices)
    {
        tracing::debug!("Synced devices: {:?}", devices);
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
