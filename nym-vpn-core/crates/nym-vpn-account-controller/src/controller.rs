// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_config::defaults::NymNetworkDetails;
use nym_http_api_client::UserAgent;
use nym_vpn_api_client::types::{Device, VpnApiAccount};
use nym_vpn_store::{keys::KeyStore, mnemonic::MnemonicStorage};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{
    error::Error,
    shared_state::{
        DeviceState, MnemonicState, RemoteAccountState, SharedAccountState, SubscriptionState,
    },
};

#[allow(unused)]
#[derive(Clone, Debug)]
pub enum AccountCommand {
    RefreshAccountState,
    RegisterDevice,
}

pub struct AccountController<S>
where
    S: nym_vpn_store::VpnStorage,
{
    // The underlying storage used to store the account and device keys
    storage: Arc<tokio::sync::Mutex<S>>,

    // The API client used to interact with the nym-vpn-api
    api_client: nym_vpn_api_client::VpnApiClient,

    // The current state of the account
    account_state: SharedAccountState,

    // Receiver channel used to receive commands from the consumer
    command_rx: tokio::sync::mpsc::UnboundedReceiver<AccountCommand>,

    // Sender channel only used when the consumer requests a channel to talk to the controller
    command_tx: tokio::sync::mpsc::UnboundedSender<AccountCommand>,

    // Listen for cancellation signals
    cancel_token: CancellationToken,
}

impl<S> AccountController<S>
where
    S: nym_vpn_store::VpnStorage,
{
    pub fn new(
        storage: Arc<tokio::sync::Mutex<S>>,
        user_agent: UserAgent,
        cancel_token: CancellationToken,
    ) -> Self {
        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
        AccountController {
            storage,
            account_state: SharedAccountState::new(),
            api_client: create_api_client(user_agent),
            command_rx,
            command_tx,
            cancel_token,
        }
    }

    async fn load_account(&self) -> Result<VpnApiAccount, <S as MnemonicStorage>::StorageError> {
        self.storage
            .lock()
            .await
            .load_mnemonic()
            .await
            .map(VpnApiAccount::from)
            .inspect(|account| tracing::info!("Loading account id: {}", account.id()))
    }

    async fn load_device_keys(&self) -> Result<Device, <S as KeyStore>::StorageError> {
        self.storage
            .lock()
            .await
            .load_keys()
            .await
            .map(|keys| Device::from(keys.device_keypair()))
            .inspect(|device| {
                tracing::info!("Loading device keys: {}", device.identity_key());
            })
    }

    #[allow(unused)]
    pub(crate) async fn register_device(&self) {
        tracing::info!("Registering device");

        let account = match self.load_account().await {
            Ok(account) => account,
            Err(err) => {
                tracing::error!("Failed to load account: {:?}", err);
                return;
            }
        };

        let device = match self.load_device_keys().await {
            Ok(device) => device,
            Err(err) => {
                tracing::error!("Failed to load device keys: {:?}", err);
                return;
            }
        };

        let result = self.api_client.register_device(&account, &device).await;
        match result {
            Ok(device_result) => {
                tracing::info!("Device registered: {}", device_result.device_identity_key);
            }
            Err(err) => {
                tracing::error!("Failed to register device: {:?}", err);
            }
        }
    }

    async fn update_mnemonic_state(&self) -> Option<VpnApiAccount> {
        match self.load_account().await {
            Ok(account) => {
                tracing::debug!("Our account id: {}", account.id());
                self.account_state.set_mnemonic(MnemonicState::Stored).await;
                Some(account)
            }
            Err(err) => {
                tracing::debug!("No account stored: {}", err);
                self.account_state
                    .set_mnemonic(MnemonicState::NotStored)
                    .await;
                None
            }
        }
    }

    async fn update_remote_account_state(&self, account: &VpnApiAccount) -> Result<(), Error> {
        let account_summary = match self.api_client.get_account_summary(account).await {
            Ok(account_summary) => {
                tracing::info!("Account summary: {:?}", account_summary);
                account_summary
            }
            Err(err) => {
                tracing::warn!("Failed to get account summary: {:?}", err);
                self.account_state
                    .set_account(RemoteAccountState::NotRegistered)
                    .await;
                return Err(Error::FailedToGetAccountSummary);
            }
        };

        self.account_state
            .set_account(RemoteAccountState::from(account_summary.account.status))
            .await;

        self.account_state
            .set_subscription(SubscriptionState::from(account_summary.subscription))
            .await;

        Ok(())
    }

    async fn update_device_state(&self, account: &VpnApiAccount) {
        let our_device = match self.load_device_keys().await {
            Ok(device) => device,
            Err(err) => {
                tracing::error!("Failed to load device keys: {:?}", err);
                return;
            }
        };

        let devices = match self.api_client.get_devices(account).await {
            Ok(devices) => devices,
            Err(err) => {
                tracing::warn!("Failed to get devices: {:?}", err);
                return;
            }
        };

        // TODO: pagination
        let found_device = devices.items.iter().find(|device| {
            device.device_identity_key == our_device.identity_key().to_base58_string()
        });

        let Some(found_device) = found_device else {
            tracing::info!("Our device is not registered");
            self.account_state
                .set_device(DeviceState::NotRegistered)
                .await;
            return;
        };

        self.account_state
            .set_device(DeviceState::from(found_device.status))
            .await;
    }

    pub(crate) async fn refresh_account_state(&self) {
        let Some(account) = self.update_mnemonic_state().await else {
            return;
        };
        if self.update_remote_account_state(&account).await.is_ok() {
            self.update_device_state(&account).await;
        }
    }

    async fn handle_command(&self, command: AccountCommand) {
        tracing::info!("Received command: {:?}", command);
        match command {
            AccountCommand::RefreshAccountState => {
                self.refresh_account_state().await;
            }
            AccountCommand::RegisterDevice => {
                self.register_device().await;
            }
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some(command) = self.command_rx.recv() => {
                    self.handle_command(command).await;
                }
                _ = self.cancel_token.cancelled() => {
                    tracing::trace!("Received cancellation signal");
                    break;
                }
                else => {
                    tracing::debug!("Account controller channel closed");
                    break;
                }
            }
        }
        tracing::debug!("Account controller is exiting");
    }

    pub fn shared_state(&self) -> SharedAccountState {
        self.account_state.clone()
    }

    pub fn command_tx(&self) -> tokio::sync::mpsc::UnboundedSender<AccountCommand> {
        self.command_tx.clone()
    }
}

fn get_nym_vpn_api_url() -> Result<Url, Error> {
    NymNetworkDetails::new_from_env()
        .nym_vpn_api_url
        .ok_or(Error::MissingApiUrl)?
        .parse()
        .map_err(|_| Error::InvalidApiUrl)
        .inspect(|url| tracing::info!("Using nym-vpn-api url: {}", url))
}

fn create_api_client(user_agent: UserAgent) -> nym_vpn_api_client::VpnApiClient {
    let nym_vpn_api_url = get_nym_vpn_api_url().unwrap();
    nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent).unwrap()
}
