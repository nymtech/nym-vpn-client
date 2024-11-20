// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::response::{NymVpnAccountSummaryResponse, NymVpnDevice};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    commands::{AccountCommand, AccountCommandError, ReturnSender},
    error::Error,
    shared_state::{AccountRegistered, DeviceState, SharedAccountState},
};

pub struct AccountControllerCommander {
    pub(super) command_tx: UnboundedSender<AccountCommand>,
    pub(super) shared_state: SharedAccountState,
}

impl AccountControllerCommander {
    pub async fn ensure_update_account(
        &self,
    ) -> Result<Option<NymVpnAccountSummaryResponse>, AccountCommandError> {
        let state = self.shared_state.lock().await.clone();
        match state.account_registered {
            Some(AccountRegistered::Registered) => return Ok(None),
            Some(AccountRegistered::NotRegistered) | None => {}
        }
        self.update_account().await.map(Some)
    }

    pub async fn update_account(
        &self,
    ) -> Result<NymVpnAccountSummaryResponse, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::UpdateAccountState(Some(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn ensure_update_device(&self) -> Result<DeviceState, AccountCommandError> {
        let state = self.shared_state.lock().await.clone();
        match state.device {
            Some(DeviceState::Active) => return Ok(DeviceState::Active),
            Some(DeviceState::NotRegistered)
            | Some(DeviceState::Inactive)
            | Some(DeviceState::DeleteMe)
            | None => {}
        }
        self.update_device().await
    }

    pub async fn update_device(&self) -> Result<DeviceState, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::UpdateDeviceState(Some(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn ensure_register_device(&self) -> Result<(), AccountCommandError> {
        let state = self.shared_state.lock().await.clone();
        match state.device {
            Some(DeviceState::Active) => return Ok(()),
            Some(DeviceState::NotRegistered)
            | Some(DeviceState::Inactive)
            | Some(DeviceState::DeleteMe)
            | None => {}
        }
        self.register_device().await.map(|_device| ())
    }

    pub async fn register_device(&self) -> Result<NymVpnDevice, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RegisterDevice(Some(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn ensure_available_tickets(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RequestZkNym(Some(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    // Send a basic command without waiting for a response
    pub fn send(&self, command: AccountCommand) -> Result<(), Error> {
        self.command_tx
            .send(command)
            .map_err(|source| Error::AccountCommandSend { source })
    }
}
