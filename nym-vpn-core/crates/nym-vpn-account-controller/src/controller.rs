// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use nym_compact_ecash::Base58 as _;
use nym_config::defaults::NymNetworkDetails;
use nym_credentials::{
    AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures, EpochVerificationKey,
};
use nym_credentials_interface::TicketType;
use nym_ecash_time::EcashTime as _;
use nym_http_api_client::{HttpClientError, UserAgent};
use nym_sdk::mixnet::CredentialStorage;
use nym_vpn_api_client::{
    response::{NymVpnZkNym, NymVpnZkNymStatus},
    types::{Device, VpnApiAccount},
};
use nym_vpn_store::{keys::KeyStore, mnemonic::MnemonicStorage};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{
    ecash_client::VpnEcashApiClient,
    error::Error,
    shared_state::{
        DeviceState, MnemonicState, RemoteAccountState, SharedAccountState, SubscriptionState,
    },
};

// If we go below this threshold, we should request more tickets
// TODO: I picket a random number, check what is shoult be. Or if we can express this in terms of
// data/bandwidth.
const TICKET_THRESHOLD: u32 = 10;

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

    // Remove zk-nym status
    remote_zk_nym: HashMap<String, NymVpnZkNym>,

    // Map of zk-nym types to ticket types, since the remote doesn't returns this info
    // TODO: make the vpn-api return this as part of NymVpnZkNym
    zk_nym_types_map: HashMap<String, TicketType>,

    // Keep track of which zknyms we have imported, so that we don't import the same twice
    // TODO: can we extend the credential store with a name field?
    #[allow(unused)]
    zk_nym_imported: Vec<String>,

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
            remote_zk_nym: Default::default(),
            zk_nym_types_map: Default::default(),
            zk_nym_imported: Default::default(),
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

    async fn request_zk_nym_by_type(
        &mut self,
        account: &VpnApiAccount,
        device: &Device,
        ticketbook_type: TicketType,
    ) -> Result<(), Error> {
        tracing::info!("Requesting zk-nym (inner)");

        let ecash_keypair = device.create_ecash_keypair();
        let expiration_date = nym_ecash_time::ecash_default_expiration_date();

        let (withdrawal_request, _request_info) = nym_compact_ecash::withdrawal_request(
            ecash_keypair.secret_key(),
            expiration_date.ecash_unix_timestamp(),
            ticketbook_type.encode(),
        )
        .map_err(Error::ConstructWithdrawalRequest)?;

        let ecash_pubkey = ecash_keypair.public_key().to_base58_string();

        // Insert pending request into credential storage?
        // Since we the id for the request is a string for the api, and a number in the storage,
        // this would be awkard.

        let response = self
            .api_client
            .request_zk_nym(
                account,
                device,
                withdrawal_request.to_bs58(),
                ecash_pubkey,
                ticketbook_type.to_string(),
            )
            .await
            .map_err(Error::RequestZkNym)?;

        tracing::info!("zk-nym requested: {:#?}", response);
        self.remote_zk_nym
            .insert(response.id.clone(), response.clone());
        self.zk_nym_types_map.insert(response.id, ticketbook_type);
        Ok(())
    }

    async fn request_zk_nym_all(&mut self) -> Result<(), Error> {
        tracing::info!("Requesting zk-nym (inner)");

        let account = self.load_account().await?;
        let device = self.load_device_keys().await?;

        // When requesing zknyms we first update the remote status so that we don't re-request
        // ticketbook types we have already requested.
        self.update_remote_zk_nym_status().await?;

        // Check which ticket types we already have pending
        let remote_zknym_id_pending = self
            .remote_zk_nym
            .iter()
            .filter_map(|(id, zkn)| {
                if matches!(zkn.status, NymVpnZkNymStatus::Pending) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let remote_zknym_types_pending = remote_zknym_id_pending
            .iter()
            .filter_map(|id| self.zk_nym_types_map.get(*id))
            .cloned()
            .collect::<Vec<_>>();

        // Then we check local storage to see what ticket types we already have stored
        let local_remaining_tickets = self.check_local_remaining_tickets().await;
        for (ticket_type, remaining) in &local_remaining_tickets {
            tracing::info!("{}, remaining: {}", ticket_type, remaining);
        }

        let ticket_types_already_stored_with_sufficient_tickets = local_remaining_tickets
            .into_iter()
            .filter(|(_, remaining)| *remaining > TICKET_THRESHOLD)
            .map(|(ticket_type, _)| ticket_type)
            .collect::<Vec<_>>();

        let ticket_types_needed_to_request = ticketbook_types()
            .iter()
            .filter(|ticket_type| {
                !ticket_types_already_stored_with_sufficient_tickets.contains(ticket_type)
                    && !remote_zknym_types_pending.contains(ticket_type)
            })
            .cloned()
            .collect::<Vec<_>>();

        for ticketbook_type in ticket_types_needed_to_request {
            if let Err(err) = self
                .request_zk_nym_by_type(&account, &device, ticketbook_type)
                .await
            {
                tracing::error!("Failed to request zk-nym: {:#?}", err);
            }
        }
        Ok(())
    }

    // Get zk-nyms for the device and store them in the credential storage if they are active and
    // we have not already stored them.
    async fn get_device_zk_nym(&mut self) -> Result<(), Error> {
        tracing::info!("Getting device zk-nym");

        // First we sync our local state with the remote state
        self.update_remote_zk_nym_status().await?;

        // TODO: iterate through the zk-nym status we just updated.
        // For all of them that are marked as Active, and not already stored:
        // 1. Unblind and verify
        // 2. Aggregate partial wallets
        // 3. Store in credential storage

        Ok(())
    }

    async fn check_local_remaining_tickets(&self) -> Vec<(TicketType, u32)> {
        let ticketbooks_info = self
            .credential_storage
            .get_ticketbooks_info()
            .await
            .unwrap();

        // For each ticketbook type, iterate over and check if we have enough tickets stored
        // locally
        let ticketbook_types = ticketbook_types();

        let mut request_zk_nym = Vec::new();
        for ticketbook_type in ticketbook_types.iter() {
            let available_tickets: u32 = ticketbooks_info
                .iter()
                .filter(|ticketbook| ticketbook.ticketbook_type == ticketbook_type.to_string())
                .map(|ticketbook| ticketbook.total_tickets - ticketbook.used_tickets)
                .sum();

            request_zk_nym.push((*ticketbook_type, available_tickets));
        }
        request_zk_nym
    }

    #[allow(unused)]
    async fn is_verification_key_valid(&self) -> Result<bool, Error> {
        let base_url = get_api_url()?;
        let vpn_ecash_api_client = VpnEcashApiClient::new(base_url)?;

        let aggregated_coin_indices_signatures = vpn_ecash_api_client
            .get_aggregated_coin_indices_signatures()
            .await?;
        let current_epoch = aggregated_coin_indices_signatures.epoch_id;

        self.credential_storage
            .get_master_verification_key(current_epoch)
            .await
            .map(|key| key.is_some())
            .map_err(Error::from)
    }

    async fn update_verification_key(&mut self) -> Result<(), Error> {
        tracing::info!("Updating verification key");

        let base_url = get_api_url()?;
        let vpn_ecash_api_client = VpnEcashApiClient::new(base_url)?;

        let master_verification_key = vpn_ecash_api_client.get_master_verification_key().await?;
        let aggregated_coin_indices_signatures = vpn_ecash_api_client
            .get_aggregated_coin_indices_signatures()
            .await?;
        let current_epoch = aggregated_coin_indices_signatures.epoch_id;

        let verification_key = EpochVerificationKey {
            epoch_id: current_epoch,
            key: master_verification_key.key,
        };

        self.credential_storage
            .insert_master_verification_key(&verification_key)
            .await
            .map_err(Error::from)
    }

    #[allow(unused)]
    async fn is_coin_indices_signatures_valid(&self) -> Result<bool, Error> {
        let base_url = get_api_url()?;
        let vpn_ecash_api_client = VpnEcashApiClient::new(base_url)?;

        let aggregated_coin_indices_signatures = vpn_ecash_api_client
            .get_aggregated_coin_indices_signatures()
            .await?;
        let current_epoch = aggregated_coin_indices_signatures.epoch_id;

        self.credential_storage
            .get_coin_index_signatures(current_epoch)
            .await
            .map(|signatures| signatures.is_some())
            .map_err(Error::from)
    }

    async fn update_coin_indices_signatures(&mut self) -> Result<(), Error> {
        tracing::info!("Updating coin indices signatures");

        let base_url = get_api_url()?;
        let vpn_ecash_api_client = VpnEcashApiClient::new(base_url)?;

        let aggregated_coin_indices_signatures = vpn_ecash_api_client
            .get_aggregated_coin_indices_signatures()
            .await?;

        let coin_indices_signatures = AggregatedCoinIndicesSignatures {
            epoch_id: aggregated_coin_indices_signatures.epoch_id,
            signatures: aggregated_coin_indices_signatures.signatures,
        };

        self.credential_storage
            .insert_coin_index_signatures(&coin_indices_signatures)
            .await
            .map_err(Error::from)
    }

    #[allow(unused)]
    async fn is_expiration_date_signatures_valid(&self) -> Result<bool, Error> {
        todo!();
    }

    async fn update_expiration_date_signatures(&mut self) -> Result<(), Error> {
        tracing::info!("Updating expiration date signatures");

        let base_url = get_api_url()?;
        let vpn_ecash_api_client = VpnEcashApiClient::new(base_url)?;

        let aggregated_expiration_data_signatures = vpn_ecash_api_client
            .get_aggregated_expiration_data_signatures()
            .await?;

        let expiration_date_signatures = AggregatedExpirationDateSignatures {
            epoch_id: aggregated_expiration_data_signatures.epoch_id,
            expiration_date: aggregated_expiration_data_signatures.expiration_date,
            signatures: aggregated_expiration_data_signatures.signatures,
        };
        self.credential_storage
            .insert_expiration_date_signatures(&expiration_date_signatures)
            .await
            .map_err(Error::from)
    }

    async fn update_remote_zk_nym_status(&mut self) -> Result<(), Error> {
        tracing::info!("Updating remote zk-nym status");
        let account = self.load_account().await?;
        let device = self.load_device_keys().await?;

        let remote_zknym = self
            .api_client
            .get_device_zk_nyms(&account, &device)
            .await
            .map_err(Error::GetZkNyms)?;

        tracing::info!("Received remote zk-nyms:");
        for remote_zknym in &remote_zknym.items {
            tracing::info!("{:?}", remote_zknym);
        }

        // Update the local pending zk-nym status map
        for remote_zknym in &remote_zknym.items {
            let zknym_new = remote_zknym.clone();
            if let Some(old) = self
                .remote_zk_nym
                .insert(remote_zknym.id.clone(), remote_zknym.clone())
            {
                tracing::info!(
                    "zk-nym status updated for {}: {:?} -> {:?}",
                    zknym_new.id,
                    old.status,
                    zknym_new.status
                );
            } else {
                tracing::info!("zk-nym added (must be from previous run): {}", zknym_new.id);
            }
        }

        // Check if we have record of a pending zknym that is not listed in the response
        let pending_zknym_ids = self.remote_zk_nym.keys().cloned().collect::<Vec<_>>();
        let missing_zknym_ids = pending_zknym_ids
            .iter()
            .filter(|id| !remote_zknym.items.iter().any(|zknym| zknym.id == **id))
            .cloned()
            .collect::<Vec<_>>();

        for missing_id in missing_zknym_ids {
            tracing::info!("zk-nym missing from response: {}", missing_id);
        }

        // Log zk-nym status
        tracing::info!("Local zk-nym state:");
        for zknym in self.remote_zk_nym.values() {
            tracing::info!("{:#?}", zknym);
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

        tracing::info!("Current state: {:#?}", self.shared_state().get().await);

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
            AccountCommand::RequestZkNym => self.request_zk_nym_all().await,
            AccountCommand::GetDeviceZkNym => self.get_device_zk_nym().await,
        }
    }

    async fn print_credential_storage_info(&self) -> Result<(), Error> {
        tracing::info!("Printing credential storage info");
        let ticketbooks_info = self.credential_storage.get_ticketbooks_info().await?;
        tracing::info!("Ticketbooks stored: {}", ticketbooks_info.len());
        for ticketbook in ticketbooks_info {
            tracing::info!("Ticketbook id: {}", ticketbook.id);
        }

        let pending_ticketbooks = self.credential_storage.get_pending_ticketbooks().await?;
        for pending in pending_ticketbooks {
            tracing::info!("Pending ticketbook id: {}", pending.pending_id);
        }
        Ok(())
    }

    pub async fn run(mut self) {
        if let Err(err) = self.print_credential_storage_info().await {
            tracing::error!("Failed to print credential storage info: {:#?}", err);
        }

        self.update_verification_key()
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to update master verification key: {:#?}", err)
            })
            .ok();
        self.update_coin_indices_signatures()
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to update coin indices signatures: {:#?}", err)
            })
            .ok();
        self.update_expiration_date_signatures()
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to update expiration date signatures: {:#?}", err)
            })
            .ok();

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

fn get_api_url() -> Result<Url, Error> {
    NymNetworkDetails::new_from_env()
        .endpoints
        .first()
        .unwrap()
        .api_url()
        .ok_or(Error::MissingApiUrl)
        .inspect(|url| tracing::info!("Using nym-api url: {}", url))
}

fn get_nym_vpn_api_url() -> Result<Url, Error> {
    NymNetworkDetails::new_from_env()
        .nym_vpn_api_url()
        .ok_or(Error::MissingApiUrl)
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

// TODO: add #[derive(EnumIter)] to TicketType so we can iterate over it directly.
fn ticketbook_types() -> [TicketType; 4] {
    [
        TicketType::V1MixnetEntry,
        TicketType::V1MixnetExit,
        TicketType::V1WireguardEntry,
        TicketType::V1WireguardExit,
    ]
}
