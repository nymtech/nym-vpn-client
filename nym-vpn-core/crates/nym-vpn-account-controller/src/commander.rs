// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::response::{NymVpnAccountSummaryResponse, NymVpnDevice};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    commands::{AccountCommand, AccountCommandError, RequestZkNymSuccessSummary, ReturnSender},
    error::Error,
    shared_state::{AccountRegistered, DeviceState, SharedAccountState},
    AvailableTicketbooks,
};

#[derive(Clone)]
pub struct AccountControllerCommander {
    pub(super) command_tx: UnboundedSender<AccountCommand>,
    pub(super) shared_state: SharedAccountState,
}

// Basic set of commands that can be sent to the account controller

impl AccountControllerCommander {
    // Send a basic command without waiting for a response
    pub fn send(&self, command: AccountCommand) -> Result<(), Error> {
        self.command_tx
            .send(command)
            .map_err(|source| Error::AccountCommandSend { source })
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

    pub async fn update_device(&self) -> Result<DeviceState, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::UpdateDeviceState(Some(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn register_device(&self) -> Result<NymVpnDevice, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RegisterDevice(Some(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn get_available_tickets(&self) -> Result<AvailableTicketbooks, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::GetAvailableTickets(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn request_zk_nyms(&self) -> Result<RequestZkNymSuccessSummary, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RequestZkNym(Some(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }
}

// Set of commands used to ensure that the account controller is in the correct state before
// proceeding with other operations

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

    pub async fn ensure_available_zk_nyms(&self) -> Result<(), AccountCommandError> {
        if self
            .get_available_tickets()
            .await?
            .is_all_ticket_types_above_threshold(0)
        {
            // If all ticket types are above zero, we're good to go. Additional ticketbooks will
            // be requested in the background, but we should have enough to connect.
            return Ok(());
        }
        self.request_zk_nyms().await.map(|_res| ())
    }

    pub async fn wait_for_account_ready_to_connect(
        &self,
        credential_mode: bool,
    ) -> Result<(), AccountCommandError> {
        self.ensure_update_account().await?;
        self.ensure_update_device().await?;
        self.ensure_register_device().await?;
        if credential_mode {
            self.ensure_available_zk_nyms().await?;
        }
        Ok(())
    }
}
