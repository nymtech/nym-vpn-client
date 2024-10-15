// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use nym_compact_ecash::Base58 as _;
use nym_config::defaults::NymNetworkDetails;
use nym_credentials_interface::TicketType;
use nym_ecash_time::EcashTime as _;
use nym_http_api_client::{HttpClientError, UserAgent};
use nym_sdk::mixnet::CredentialStorage;
use nym_vpn_api_client::{
    response::NymVpnZkNymStatus,
    types::{Device, VpnApiAccount},
};
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
    UpdateSharedAccountState,
    RegisterDevice,
    RequestZkNym,
    GetDeviceZkNym,
}

pub struct AccountController<S>
where
    S: nym_vpn_store::VpnStorage,
{
    // The underlying storage used to store the account and device keys
    storage: Arc<tokio::sync::Mutex<S>>,

    // Storage used for credentials
    credential_storage: nym_credential_storage::persistent_storage::PersistentStorage,

    // The API client used to interact with the nym-vpn-api
    api_client: nym_vpn_api_client::VpnApiClient,

    // The current state of the account
    account_state: SharedAccountState,

    // Pending zk-nym requests that we are waiting for
    pending_zk_nym: HashMap<String, NymVpnZkNymStatus>,

    // Receiver channel used to receive commands from the consumer
    command_rx: tokio::sync::mpsc::UnboundedReceiver<AccountCommand>,

    // Sender channel primarily used when the consumer requests a channel to talk to the
    // controller, but also to queue up commands to itself
    command_tx: tokio::sync::mpsc::UnboundedSender<AccountCommand>,

    // Listen for cancellation signals
    cancel_token: CancellationToken,
}

impl<S> AccountController<S>
where
    S: nym_vpn_store::VpnStorage,
{
    pub async fn new(
        storage: Arc<tokio::sync::Mutex<S>>,
        data_dir: PathBuf,
        user_agent: UserAgent,
        cancel_token: CancellationToken,
    ) -> Self {
        // TODO: remove unwraps.
        let storage_paths = nym_sdk::mixnet::StoragePaths::new_from_dir(data_dir).unwrap();
        let credential_storage = storage_paths.persistent_credential_storage().await.unwrap();

        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();

        AccountController {
            storage,
            credential_storage,
            account_state: SharedAccountState::new(),
            api_client: create_api_client(user_agent),
            pending_zk_nym: Default::default(),
            command_rx,
            command_tx,
            cancel_token,
        }
    }

    // Load account and keep the error type
    async fn load_account_from_storage(
        &self,
    ) -> Result<VpnApiAccount, <S as MnemonicStorage>::StorageError> {
        self.storage
            .lock()
            .await
            .load_mnemonic()
            .await
            .map(VpnApiAccount::from)
            .inspect(|account| tracing::info!("Loading account id: {}", account.id()))
    }

    // Convenience function to load account and box the error
    async fn load_account(&self) -> Result<VpnApiAccount, Error> {
        self.load_account_from_storage()
            .await
            .map_err(|err| Error::MnemonicStore {
                source: Box::new(err),
            })
    }

    // Load device keys and keep the error type
    async fn load_device_keys_from_storage(&self) -> Result<Device, <S as KeyStore>::StorageError> {
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

    // Convenience function to load device keys and box the error
    async fn load_device_keys(&self) -> Result<Device, Error> {
        self.load_device_keys_from_storage()
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }

    async fn register_device(&self) -> Result<(), Error> {
        tracing::info!("Registering device");

        let account = self.load_account().await?;
        let device = self.load_device_keys().await?;

        self.api_client
            .register_device(&account, &device)
            .await
            .map(|device_result| {
                tracing::info!("Response: {:#?}", device_result);
                tracing::info!("Device registered: {}", device_result.device_identity_key);
            })
            .map_err(Error::RegisterDevice)?;

        self.command_tx
            .send(AccountCommand::UpdateSharedAccountState)?;

        Ok(())
    }

    async fn request_zk_nym(&mut self) -> Result<(), Error> {
        tracing::info!("Requesting zk-nym");

        let account = self.load_account().await?;
        let device = self.load_device_keys().await?;

        let ecash_keypair = device.create_ecash_keypair();
        let ticketbook_type = TicketType::V1MixnetEntry;

        let expiration_date = nym_ecash_time::ecash_default_expiration_date();

        let (withdrawal_request, _request_info) = nym_compact_ecash::withdrawal_request(
            ecash_keypair.secret_key(),
            expiration_date.ecash_unix_timestamp(),
            ticketbook_type.encode(),
        )
        .map_err(Error::ConstructWithdrawalRequest)?;

        let ecash_pubkey = ecash_keypair.public_key().to_base58_string();

        // TODO: insert pending request into credential storage?

        let response = self
            .api_client
            .request_zk_nym(
                &account,
                &device,
                withdrawal_request.to_bs58(),
                ecash_pubkey,
                ticketbook_type.to_string(),
            )
            .await
            .map_err(Error::RequestZkNym)?;

        tracing::info!("zk-nym requested: {:#?}", response);
        self.pending_zk_nym.insert(response.id, response.status);
        Ok(())
    }

    async fn get_device_zk_nym(&self) -> Result<(), Error> {
        tracing::info!("Getting device zk-nym");

        // Current pending zk-nym requests
        tracing::info!("Pending zk-nym requests: {:#?}", self.pending_zk_nym);

        let account = self.load_account().await?;
        let device = self.load_device_keys().await?;

        let zknym = self
            .api_client
            .get_device_zk_nyms(&account, &device)
            .await
            .map_err(Error::GetZkNyms)?;

        // TODO: pagination
        for zknym in zknym.items {
            tracing::info!("zk-nym: {:#?}", zknym);

            let _blinded_shares = match zknym.status {
                NymVpnZkNymStatus::Active => zknym.blinded_shares,
                NymVpnZkNymStatus::Pending
                | NymVpnZkNymStatus::Revoking
                | NymVpnZkNymStatus::Revoked
                | NymVpnZkNymStatus::Error => {
                    break;
                }
            };

            // TODO: unblind and verify
            // TODO: aggregate partial wallets
        }
        Ok(())
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
        tracing::info!("Updating remote account state");

        let response = self.api_client.get_account_summary(account).await;

        // Check if the response indicates that we are not registered
        if let Some(403) = &response.as_ref().err().and_then(extract_status_code) {
            self.account_state
                .set_account(RemoteAccountState::NotRegistered)
                .await;
        }

        let account_summary = response.map_err(Error::GetAccountSummary)?;
        tracing::info!("Account summary: {:#?}", account_summary);

        self.account_state
            .set_account(RemoteAccountState::from(account_summary.account.status))
            .await;

        self.account_state
            .set_subscription(SubscriptionState::from(account_summary.subscription))
            .await;

        Ok(())
    }

    async fn update_device_state(&self, account: &VpnApiAccount) -> Result<(), Error> {
        tracing::info!("Updating device state");
        let our_device = self.load_device_keys().await?;

        let devices = self
            .api_client
            .get_devices(account)
            .await
            .map_err(Error::GetDevices)?;

        tracing::info!("Registered devices: {:#?}", devices);

        // TODO: pagination
        let found_device = devices.items.iter().find(|device| {
            device.device_identity_key == our_device.identity_key().to_base58_string()
        });

        let Some(found_device) = found_device else {
            tracing::info!("Our device is not registered");
            self.account_state
                .set_device(DeviceState::NotRegistered)
                .await;
            return Ok(());
        };

        self.account_state
            .set_device(DeviceState::from(found_device.status))
            .await;

        Ok(())
    }

    pub(crate) async fn update_shared_account_state(&self) -> Result<(), Error> {
        let Some(account) = self.update_mnemonic_state().await else {
            return Ok(());
        };

        self.update_remote_account_state(&account).await?;
        self.update_device_state(&account).await?;

        if self.shared_state().is_ready_to_register_device().await {
            self.command_tx.send(AccountCommand::RegisterDevice)?;
        }
        Ok(())
    }

    async fn handle_command(&mut self, command: AccountCommand) -> Result<(), Error> {
        tracing::info!("Received command: {:?}", command);
        match command {
            AccountCommand::UpdateSharedAccountState => self.update_shared_account_state().await,
            AccountCommand::RegisterDevice => self.register_device().await,
            AccountCommand::RequestZkNym => self.request_zk_nym().await,
            AccountCommand::GetDeviceZkNym => self.get_device_zk_nym().await,
        }
    }

    async fn print_credential_storage_info(&self) -> Result<(), Error> {
        tracing::info!("Printing credential storage info");
        let ticketbooks_info = self.credential_storage.get_ticketbooks_info().await?;
        for ticketbook in ticketbooks_info {
            tracing::info!("Ticketbook id: {}", ticketbook.id);
        }

        let pending_ticketbooks = self.credential_storage.get_pending_ticketbooks().await?;
        for a in pending_ticketbooks {
            tracing::info!("Pending ticketbook id: {}", a.pending_id);
        }
        Ok(())
    }

    pub async fn run(mut self) {
        if let Err(err) = self.print_credential_storage_info().await {
            tracing::error!("Failed to print credential storage info: {:#?}", err);
        }

        loop {
            tokio::select! {
                Some(command) = self.command_rx.recv() => {
                    if let Err(err) = self.handle_command(command).await {
                        tracing::error!("{:#?}", err);
                    }
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

fn extract_status_code<E>(err: &E) -> Option<u16>
where
    E: std::error::Error + 'static,
{
    let mut source = err.source();
    while let Some(err) = source {
        if let Some(status) = err
            .downcast_ref::<HttpClientError>()
            .and_then(extract_status_code_inner)
        {
            return Some(status);
        }
        source = err.source();
    }
    None
}

fn extract_status_code_inner(err: &HttpClientError) -> Option<u16> {
    match err {
        HttpClientError::EndpointFailure { status, .. } => Some((*status).into()),
        _ => None,
    }
}
