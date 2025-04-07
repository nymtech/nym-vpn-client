// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::SocketAddr;

use nym_offline_monitor::ConnectivityHandle;
use nym_vpn_api_client::response::{NymVpnAccountSummaryResponse, NymVpnDevice, NymVpnUsage};
use nym_vpn_lib_types::{
    AccountCommandError, RegisterDeviceError, RequestZkNymError, SyncAccountError, SyncDeviceError,
};
use nym_vpn_store::mnemonic::Mnemonic;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    commands::{tasks::request_zknym::RequestZkNymSummary, AccountCommand, ReturnSender},
    shared_state::{AccountRegistered, DeviceState, SharedAccountState},
    AvailableTicketbooks,
};

#[derive(Clone)]
pub struct AccountCommandSender {
    command_tx: UnboundedSender<AccountCommand>,
    shared_state: SharedAccountState,
}

// Basic set of commands that can be sent to the account controller

impl AccountCommandSender {
    pub fn new(
        command_tx: UnboundedSender<AccountCommand>,
        shared_state: SharedAccountState,
    ) -> Self {
        Self {
            command_tx,
            shared_state,
        }
    }

    pub async fn store_account(&self, mnemonic: Mnemonic) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::StoreAccount(tx, mnemonic))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn login(&self, mnemonic: Mnemonic) -> Result<(), AccountCommandError> {
        self.store_account(mnemonic).await?;
        self.ensure_update_account().await?;
        self.ensure_update_device().await?;
        Ok(())
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
    ) -> Result<NymVpnAccountSummaryResponse, SyncAccountError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::SyncAccountState(Some(tx)))
            .map_err(SyncAccountError::internal)?;
        rx.await.map_err(SyncAccountError::internal)?
    }

    pub fn background_sync_account_state(&self) {
        let _ = self
            .command_tx
            .send(AccountCommand::SyncAccountState(None))
            .inspect_err(|err| tracing::error!("Failed to send sync account state command: {err}"));
    }

    pub async fn sync_device_state(&self) -> Result<DeviceState, SyncDeviceError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::SyncDeviceState(Some(tx)))
            .map_err(SyncDeviceError::internal)?;
        rx.await.map_err(SyncDeviceError::internal)?
    }

    pub fn background_sync_device_state(&self) {
        self.command_tx
            .send(AccountCommand::SyncDeviceState(None))
            .inspect_err(|err| tracing::error!("Failed to send sync device state command: {err}"))
            .ok();
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

    pub async fn register_device(&self) -> Result<NymVpnDevice, RegisterDeviceError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RegisterDevice(Some(tx)))
            .map_err(RegisterDeviceError::internal)?;
        rx.await.map_err(RegisterDeviceError::internal)?
    }

    pub fn background_register_device(&self) {
        self.command_tx
            .send(AccountCommand::RegisterDevice(None))
            .inspect_err(|err| tracing::error!("Failed to send register device command: {err}"))
            .ok();
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

    pub async fn request_zk_nyms(&self) -> Result<RequestZkNymSummary, RequestZkNymError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RequestZkNym(Some(tx)))
            .map_err(RequestZkNymError::internal)?;
        rx.await.map_err(RequestZkNymError::internal)?
    }

    pub fn background_request_zk_nyms(&self) {
        self.command_tx
            .send(AccountCommand::RequestZkNym(None))
            .inspect_err(|err| tracing::error!("Failed to send request zk-nyms command: {err}"))
            .ok();
    }

    // TODO: also return the result
    pub fn get_device_zk_nym(&self) -> Result<(), AccountCommandError> {
        self.command_tx
            .send(AccountCommand::GetDeviceZkNym)
            .map_err(AccountCommandError::internal)
    }

    // TODO: also return the result
    pub fn get_zk_nyms_available_for_download(&self) -> Result<(), AccountCommandError> {
        self.command_tx
            .send(AccountCommand::GetZkNymsAvailableForDownload)
            .map_err(AccountCommandError::internal)
    }

    // TODO: also return the result
    pub fn get_zk_nym_by_id(&self, id: String) -> Result<(), AccountCommandError> {
        self.command_tx
            .send(AccountCommand::GetZkNymById(id))
            .map_err(AccountCommandError::internal)
    }

    // TODO: also return the result.
    // TODO: map the error
    pub fn confirm_zk_nym_id_downloaded(&self, id: String) -> Result<(), AccountCommandError> {
        self.command_tx
            .send(AccountCommand::ConfirmZkNymIdDownloaded(id))
            .map_err(AccountCommandError::internal)
    }

    pub async fn set_static_api_addresses(
        &self,
        static_addresses: Option<Vec<SocketAddr>>,
    ) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::SetStaticApiAddresses(tx, static_addresses))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn register_offline_monitor(
        &self,
        offline_monitor: ConnectivityHandle,
    ) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RegisterOfflineMonitor(tx, offline_monitor))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }
}

// Set of commands used to ensure that the account controller is in the correct state before
// proceeding with other operations

impl AccountCommandSender {
    pub async fn ensure_update_account(
        &self,
    ) -> Result<Option<NymVpnAccountSummaryResponse>, SyncAccountError> {
        tracing::debug!("Ensuring account is synced");
        let state = self.shared_state.lock().await.clone();
        match state.account_registered {
            Some(AccountRegistered::Registered) => return Ok(None),
            Some(AccountRegistered::NotRegistered) | None => {}
        }
        self.sync_account_state().await.map(Some)
    }

    pub async fn ensure_update_device(&self) -> Result<DeviceState, SyncDeviceError> {
        tracing::debug!("Ensuring device is synced");
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

    pub async fn ensure_register_device(&self) -> Result<(), RegisterDeviceError> {
        tracing::debug!("Ensuring device is registered");
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

    pub async fn ensure_available_zk_nyms(&self) -> Result<(), RequestZkNymError> {
        tracing::debug!("Ensuring available zk-nyms in the local credential store");
        if self
            .get_available_tickets()
            .await
            .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?
            .is_all_ticket_types_above_soft_threshold()
        {
            // If we have enough zk-nym ticketbooks, we can just return. Additional ticketbooks
            // will be requested in the background later, if needed.
            return Ok(());
        }

        // Request new zk-nym ticketbooks
        let results = self.request_zk_nyms().await?;

        // If any of them failed, return an error
        if let Some(Err(err)) = results.into_iter().find(Result::is_err) {
            Err(err.clone())
        } else {
            Ok(())
        }
    }

    pub async fn wait_for_account_ready_to_connect(
        &self,
        credential_mode: bool,
    ) -> Result<(), AccountCommandError> {
        tracing::debug!("Waiting for account to be ready to connect");
        self.ensure_update_account().await?;
        self.ensure_update_device().await?;
        self.ensure_register_device().await?;
        if credential_mode {
            self.ensure_available_zk_nyms().await?;
        }
        Ok(())
    }
}
