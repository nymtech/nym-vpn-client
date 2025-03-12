// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::{
    response::NymVpnDevice,
    types::{Device, VpnApiAccount},
    VpnApiClient,
};
use nym_vpn_lib_types::{RegisterDeviceError, VpnApiErrorResponse};

use crate::{
    shared_state::{DeviceState, RegisterDeviceResult},
    SharedAccountState,
};

use super::AccountCommandResult;

pub(crate) struct RegisterDeviceCommandHandler {
    id: uuid::Uuid,
    account: VpnApiAccount,
    device: Device,
    account_state: SharedAccountState,
    vpn_api_client: VpnApiClient,
}

impl RegisterDeviceCommandHandler {
    pub(crate) fn new(
        account: VpnApiAccount,
        device: Device,
        account_state: SharedAccountState,
        vpn_api_client: VpnApiClient,
    ) -> Self {
        let id = uuid::Uuid::new_v4();
        tracing::debug!("Created new register device command handler: {}", id);
        RegisterDeviceCommandHandler {
            id,
            account,
            device,
            account_state,
            vpn_api_client,
        }
    }

    fn id_str(&self) -> String {
        format!("{:.8}", self.id.to_string())
    }

    pub(crate) async fn run(self) -> AccountCommandResult {
        AccountCommandResult::RegisterDevice(self.register_device().await)
    }

    #[tracing::instrument(
        skip(self),
        fields(id = %self.id_str()),
        ret,
        err,
    )]
    async fn register_device(self) -> Result<NymVpnDevice, RegisterDeviceError> {
        tracing::debug!("Running register device command handler: {}", self.id);

        // Defensive check for something that should not be possible
        if let Some(RegisterDeviceResult::InProgress) =
            self.account_state.lock().await.register_device_result
        {
            return Err(RegisterDeviceError::internal(
                "duplicate register device command",
            ));
        }

        self.account_state
            .set_device_registration(RegisterDeviceResult::InProgress)
            .await;

        match register_device(&self.account, &self.device, &self.vpn_api_client).await {
            Ok(device) => {
                self.account_state
                    .set_device_registration(RegisterDeviceResult::Success)
                    .await;
                self.account_state
                    .set_device(DeviceState::from(device.status))
                    .await;
                Ok(device)
            }
            Err(err) => {
                tracing::warn!("Failed to register device: {}", err);
                self.account_state
                    .set_device_registration(RegisterDeviceResult::Failed(err.clone()))
                    .await;
                Err(err)
            }
        }
    }
}

pub(crate) async fn register_device(
    account: &VpnApiAccount,
    device: &Device,
    vpn_api_client: &nym_vpn_api_client::VpnApiClient,
) -> Result<NymVpnDevice, RegisterDeviceError> {
    tracing::info!("Registering device: {:?}", device);
    let response = vpn_api_client
        .register_device(account, device)
        .await
        .map_err(|err| {
            VpnApiErrorResponse::try_from(err)
                .map(RegisterDeviceError::RegisterDeviceEndpointFailure)
                .unwrap_or_else(RegisterDeviceError::unexpected_response)
        })?;

    tracing::debug!("Response: {:#?}", response);
    tracing::info!("Device registered: {}", response.device_identity_key);
    Ok(response)
}
