// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::response::{NymVpnAccountSummaryResponse, NymVpnDevice, NymVpnUsage};
use nym_vpn_store::mnemonic::Mnemonic;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    commands::{
        request_zknym::RequestZkNymSummary, AccountCommand, AccountCommandError, ReturnSender,
    },
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

    pub async fn store_account(&self, mnemonic: Mnemonic) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::StoreAccount(tx, mnemonic))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn forget_account(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::ForgetAccount(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn sync_account_state(
        &self,
    ) -> Result<NymVpnAccountSummaryResponse, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::SyncAccountState(Some(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn sync_device_state(&self) -> Result<DeviceState, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::SyncDeviceState(Some(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn get_usage(&self) -> Result<Vec<NymVpnUsage>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::GetUsage(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn get_device_identity(&self) -> Result<String, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::GetDeviceIdentity(tx))
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

    pub async fn get_devices(&self) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::GetDevices(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn get_active_devices(&self) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::GetActiveDevices(tx))
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

    pub async fn request_zk_nyms(&self) -> Result<RequestZkNymSummary, AccountCommandError> {
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
        self.sync_account_state().await.map(Some)
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
        self.sync_device_state().await
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

        // Request new zk-nym ticketbooks
        let results = self.request_zk_nyms().await?;

        // If any of them failed, return an error
        if results.iter().any(Result::is_err) {
            Err(AccountCommandError::from(results))
        } else {
            Ok(())
        }
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
