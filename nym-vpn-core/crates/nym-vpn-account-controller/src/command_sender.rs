// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::SocketAddr;

use nym_vpn_api_client::{
    response::{NymVpnDevice, NymVpnUsage},
    types::Platform,
};
use nym_vpn_lib_types::{
    AccountCommandError, CreateAccountError, ForgetAccountError, GetMnemonicError,
    RegisterAccountError, StoreAccountError, SyncAccountError,
};
use nym_vpn_store::mnemonic::Mnemonic;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    AvailableTicketbooks, RegisterAccountResponse,
    commands::{AccountCommand, CommonCommand, ReturnSender},
};

#[derive(Clone)]
pub struct AccountCommandSender {
    command_tx: UnboundedSender<AccountCommand>,
}

// Basic set of commands that can be sent to the account controller

impl AccountCommandSender {
    pub fn new(command_tx: UnboundedSender<AccountCommand>) -> Self {
        Self { command_tx }
    }

    pub async fn store_account(&self, mnemonic: Mnemonic) -> Result<(), StoreAccountError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::StoreAccount(tx, mnemonic))
            .map_err(StoreAccountError::internal)?;
        rx.await.map_err(StoreAccountError::internal)?
    }

    pub async fn register_account(
        &self,
        mnemonic: Mnemonic,
        platform: Platform,
    ) -> Result<RegisterAccountResponse, RegisterAccountError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RegisterAccount(tx, mnemonic, platform))
            .map_err(RegisterAccountError::internal)?;
        rx.await.map_err(RegisterAccountError::internal)?
    }

    pub async fn login(&self, mnemonic: Mnemonic) -> Result<(), AccountCommandError> {
        self.store_account(mnemonic).await?;
        // self.ensure_update_account().await?;
        // self.ensure_update_device().await?;
        Ok(())
    }

    pub async fn get_stored_mnemonic(&self) -> Result<Mnemonic, GetMnemonicError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetStoredMnemonic(tx)))
            .map_err(GetMnemonicError::internal)?;
        rx.await.map_err(GetMnemonicError::internal)?
    }

    pub async fn create_account_command(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::CreateAccount(tx))
            .map_err(CreateAccountError::internal)?;
        rx.await.map_err(CreateAccountError::internal)??;
        Ok(())
    }

    pub async fn register_account_command(
        &self,
        mnemonic: Mnemonic,
        platform: Platform,
    ) -> Result<RegisterAccountResponse, AccountCommandError> {
        let response = self.register_account(mnemonic, platform).await?;
        // self.ensure_update_account().await?;
        // self.ensure_update_device().await?;
        Ok(response)
    }

    pub async fn get_stored_mnemonic_command(&self) -> Result<Mnemonic, AccountCommandError> {
        let mnemonic = self.get_stored_mnemonic().await?;
        Ok(mnemonic)
    }

    pub async fn get_account_id(&self) -> Result<Option<String>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetAccountIdentity(
                tx,
            )))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn forget_account(&self) -> Result<(), ForgetAccountError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::ForgetAccount(tx))
            .map_err(ForgetAccountError::internal)?;
        rx.await.map_err(ForgetAccountError::internal)?
    }

    pub async fn background_refresh_account_state(&self) -> Result<(), SyncAccountError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RefreshAccountState(tx))
            .map_err(SyncAccountError::internal)?;
        rx.await.map_err(SyncAccountError::internal)?
    }

    pub async fn reset_device_identity(
        &self,
        seed: Option<[u8; 32]>,
    ) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::ResetDeviceIdentity(tx, seed))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn get_usage(&self) -> Result<Vec<NymVpnUsage>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetUsage(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn get_device_identity(&self) -> Result<String, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetDeviceIdentity(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn get_devices(&self) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetDevices(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn get_active_devices(&self) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetActiveDevices(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn get_available_tickets(&self) -> Result<AvailableTicketbooks, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetAvailableTickets(
                tx,
            )))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn set_static_api_addresses(
        &self,
        static_addresses: Option<Vec<SocketAddr>>,
    ) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(
                CommonCommand::SetStaticApiAddresses(tx, static_addresses),
            ))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }
}
