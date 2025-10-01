// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::SocketAddr;

use crate::{
    AvailableTicketbooks,
    commands::{AccountCommand, CommonCommand, ReturnSender},
};
use nym_vpn_api_client::{
    response::{NymVpnDevice, NymVpnUsage},
    types::Platform,
};
use nym_vpn_lib_types::{AccountCommandError, RegisterAccountResponse};
use nym_vpn_store::types::StorableAccount;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
pub struct AccountCommandSender {
    command_tx: UnboundedSender<AccountCommand>,
}

// Basic set of commands that can be sent to the account controller

impl AccountCommandSender {
    pub fn new(command_tx: UnboundedSender<AccountCommand>) -> Self {
        Self { command_tx }
    }

    pub async fn store_account(&self, account: StorableAccount) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::StoreAccount(tx, account))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn register_account(
        &self,
        account: StorableAccount,
        platform: Platform,
    ) -> Result<RegisterAccountResponse, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RegisterAccount(tx, account, platform))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn get_stored_account(&self) -> Result<Option<StorableAccount>, AccountCommandError> {
        // let (tx, rx) = ReturnSender::new();
        // self.command_tx
        //     .send(AccountCommand::Common(CommonCommand::GetStoredMnemonic(tx)))
        //     .map_err(AccountCommandError::internal)?;
        // rx.await.map_err(AccountCommandError::internal)?
        todo!()
    }

    pub async fn create_account_command(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::CreateAccount(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)??;
        Ok(())
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

    pub async fn forget_account(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::ForgetAccount(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn background_refresh_account_state(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RefreshAccountState(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
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

    pub async fn get_device_identity(&self) -> Result<Option<String>, AccountCommandError> {
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

    pub async fn set_vpn_api_firewall_up(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::VpnApiFirewallUp(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    pub async fn set_vpn_api_firewall_down(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::VpnApiFirewallDown(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }
}
