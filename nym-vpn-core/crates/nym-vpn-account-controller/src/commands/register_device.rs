// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::{
    response::NymVpnDevice,
    types::{Device, VpnApiAccount},
    VpnApiClient,
};

use crate::{
    shared_state::{DeviceRegistration, DeviceState},
    SharedAccountState,
};

use super::{AccountCommandError, AccountCommandResult};

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
        AccountCommandResult::RegisterDevice(self.run_inner().await)
    }

    #[tracing::instrument(
        skip(self),
        name = "register_device",
        fields(id = %self.id_str()),
        ret,
        err,
    )]
    async fn run_inner(self) -> Result<NymVpnDevice, AccountCommandError> {
        tracing::debug!("Running register device command handler: {}", self.id);

        // TODO: assert that it's not already in progress. It shouldn't happen since only one
        // command per type is supposed to run at a time.
        self.account_state
            .set_device_registration(DeviceRegistration::InProgress)
            .await;

        match register_device(&self.account, &self.device, &self.vpn_api_client).await {
            Ok(device) => {
                self.account_state
                    .set_device_registration(DeviceRegistration::Success)
                    .await;
                self.account_state
                    .set_device(DeviceState::from(device.status))
                    .await;
                Ok(device)
            }
            Err(err) => {
                tracing::warn!("Failed to register device: {}", err);
                self.account_state
                    .set_device_registration(DeviceRegistration::from(&err))
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
) -> Result<NymVpnDevice, AccountCommandError> {
    tracing::info!("Registering device: {:?}", device);
    let response = vpn_api_client
        .register_device(account, device)
        .await
        .map_err(|err| {
            nym_vpn_api_client::response::extract_error_response(&err)
                .map(|e| {
                    tracing::warn!(
                        "nym-vpn-api reports: message={}, message_id={:?}",
                        e.message,
                        e.message_id,
                    );
                    AccountCommandError::RegisterDeviceEndpointFailure {
                        message_id: e.message_id.clone(),
                        message: e.message.clone(),
                    }
                })
                .unwrap_or(AccountCommandError::General(err.to_string()))
        })?;

    tracing::info!("Response: {:#?}", response);
    tracing::info!("Device registered: {}", response.device_identity_key);
    Ok(response)
}
