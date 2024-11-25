// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::{
    response::NymVpnDevice,
    types::{Device, VpnApiAccount},
    VpnApiClient,
};
use serde::{Deserialize, Serialize};

use crate::{
    shared_state::{DeviceState, RegisterDeviceResult},
    SharedAccountState,
};

use super::{AccountCommandError, AccountCommandResult, VpnApiEndpointFailure};

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
    async fn register_device(self) -> Result<NymVpnDevice, AccountCommandError> {
        tracing::debug!("Running register device command handler: {}", self.id);

        // Defensive check for something that should not be possible
        if let Some(RegisterDeviceResult::InProgress) =
            self.account_state.lock().await.register_device_result
        {
            return Err(AccountCommandError::internal(
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
                Err(AccountCommandError::from(err))
            }
        }
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterDeviceError {
    #[error("failed to register device: {0}")]
    RegisterDeviceEndpointFailure(VpnApiEndpointFailure),

    #[error("failed to register device: {0}")]
    General(String),
}

impl RegisterDeviceError {
    pub(crate) fn general(message: impl ToString) -> Self {
        RegisterDeviceError::General(message.to_string())
    }

    pub fn message(&self) -> String {
        match self {
            RegisterDeviceError::RegisterDeviceEndpointFailure(failure) => failure.message.clone(),
            RegisterDeviceError::General(message) => message.clone(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        match self {
            RegisterDeviceError::RegisterDeviceEndpointFailure(failure) => {
                failure.message_id.clone()
            }
            RegisterDeviceError::General(_) => None,
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
            nym_vpn_api_client::response::extract_error_response(&err)
                .map(|e| {
                    tracing::warn!(
                        "nym-vpn-api reports: message={}, message_id={:?}, code_reference_id={:?}",
                        e.message,
                        e.message_id,
                        e.code_reference_id,
                    );
                    RegisterDeviceError::RegisterDeviceEndpointFailure(VpnApiEndpointFailure {
                        message_id: e.message_id.clone(),
                        message: e.message.clone(),
                        code_reference_id: e.code_reference_id.clone(),
                    })
                })
                .unwrap_or_else(|| RegisterDeviceError::general(err))
        })?;

    tracing::info!("Response: {:#?}", response);
    tracing::info!("Device registered: {}", response.device_identity_key);
    Ok(response)
}
