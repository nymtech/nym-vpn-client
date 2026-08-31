// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    commands::{AccountCommand, CommonCommand, ReturnSender},
    deeplink::{CreateDeeplinkParams, DeeplinkMnemonic},
};
use nym_validator_client::nyxd::Coin;
use nym_vpn_api_client::{
    response::{NymVpnDevice, NymVpnUsage},
    types::Platform,
};
use nym_vpn_lib_types::{
    AccountCommandError, AutologinResponse, DeeplinkKind, RegisterAccountResponse, StorableAccount,
    StoredAccountMode, VpnAccountSummary,
};

use tokio::sync::mpsc::UnboundedSender;
use tracing::instrument;
use url::Url;

#[derive(Clone, Debug)]
pub struct AccountCommandSender {
    command_tx: UnboundedSender<AccountCommand>,
}

// Basic set of commands that can be sent to the account controller

impl AccountCommandSender {
    pub fn new(command_tx: UnboundedSender<AccountCommand>) -> Self {
        Self { command_tx }
    }

    #[instrument(skip(self))]
    pub async fn store_account(&self, account: StorableAccount) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::StoreAccount(tx, account))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
    pub async fn link_account(
        &self,
        privy_account: StorableAccount,
    ) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::LinkAccount(tx, privy_account))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn get_stored_account(&self) -> Result<Option<StorableAccount>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetStoredAccount(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn create_account_command(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::CreateAccount(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)??;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_account_id(&self) -> Result<Option<String>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetAccountIdentity(
                tx,
            )))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn get_canonical_account_id(&self) -> Result<Option<String>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(
                CommonCommand::GetCanonicalAccountIdentity(tx),
            ))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn get_account_mode(&self) -> Result<Option<StoredAccountMode>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetAccountMode(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn forget_account(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::ForgetAccount(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn rotate_keys(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RotateKeys(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn refresh_account_state(&self, force: bool) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RefreshAccountState(tx, force))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
    pub async fn get_usage(&self) -> Result<Vec<NymVpnUsage>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetUsage(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn get_device_identity(&self) -> Result<Option<String>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetDeviceIdentity(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn get_devices(&self) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetDevices(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn get_active_devices(&self) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetActiveDevices(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn get_account_summary(
        &self,
    ) -> Result<Option<VpnAccountSummary>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetAccountSummary(tx)))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn get_deeplink(
        &self,
        kind: DeeplinkKind,
        name: String,
        base_url: Url,
    ) -> Result<String, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        let params = CreateDeeplinkParams {
            kind,
            name,
            base_url,
        };
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetDeeplink(
                tx, params,
            )))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn get_autologin_deeplink(
        &self,
        kind: DeeplinkKind,
        name: String,
        base_url: Url,
    ) -> Result<AutologinResponse, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        let params = CreateDeeplinkParams {
            kind,
            name,
            base_url,
        };
        self.command_tx
            .send(AccountCommand::Common(CommonCommand::GetAutologinDeeplink(
                tx, params,
            )))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn derive_deeplink_mnemonic(
        &self,
        deeplink_callback_url: String,
    ) -> Result<DeeplinkMnemonic, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::Common(
                CommonCommand::DeriveDeeplinkMnemonic(tx, deeplink_callback_url),
            ))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn set_vpn_api_firewall_up(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::VpnApiFirewallUp(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn set_vpn_api_firewall_down(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::VpnApiFirewallDown(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn decentralised_balance(&self) -> Result<Vec<Coin>, AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::AccountBalance(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn decentralised_obtain_ticketbooks(&self) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::ObtainTicketbooks(tx))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }

    #[instrument(skip(self))]
    pub async fn handle_subscription_payment(&self) -> Result<(), AccountCommandError> {
        // A payment changes subscription status server-side, so we must re-fetch from the VPN API
        // rather than re-evaluating the stale cached summary.
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::RefreshAccountState(tx, true))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }
}
