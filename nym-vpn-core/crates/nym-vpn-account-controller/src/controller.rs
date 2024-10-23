// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use nym_compact_ecash::{Base58 as _, BlindedSignature, VerificationKeyAuth};
use nym_config::defaults::NymNetworkDetails;
use nym_credentials::{
    AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures, EpochVerificationKey,
    IssuedTicketBook,
};
use nym_credentials_interface::{RequestInfo, TicketType};
use nym_ecash_time::EcashTime;
use nym_http_api_client::{HttpClientError, UserAgent};
use nym_sdk::mixnet::CredentialStorage;
use nym_vpn_api_client::{
    response::{NymVpnZkNym, NymVpnZkNymStatus},
    types::{Device, VpnApiAccount},
    VpnApiClientError,
};
use nym_vpn_store::{keys::KeyStore, mnemonic::MnemonicStorage};
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::{task::JoinError, time::timeout};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{
    ecash_client::VpnEcashApiClient,
    error::Error,
    shared_state::{
        AccountState, DeviceState, MnemonicState, ReadyToRegisterDevice, SharedAccountState,
        SubscriptionState,
    },
};

// If we go below this threshold, we should request more tickets
// TODO: I picked a random number, check what is should be. Or if we can express this in terms of
// data/bandwidth.
const TICKET_THRESHOLD: u32 = 10;

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
    vpn_api_client: nym_vpn_api_client::VpnApiClient,

    // The API client used to interact with cash endpoints
    // NOTE: this is a temporary solution until the data is available on the vpn-api
    vpn_ecash_api_client: VpnEcashApiClient,

    // The current state of the account
    account_state: SharedAccountState,

    // Keep track of the current ecash epoch
    current_epoch: Option<u64>,

    // Tasks used to poll the status of zk-nyms
    polling_tasks: tokio::task::JoinSet<PollingResult>,

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
            vpn_api_client: create_api_client(user_agent),
            vpn_ecash_api_client: create_ecash_api_client(),
            account_state: SharedAccountState::new(),
            current_epoch: None,
            zk_nym_imported: Default::default(),
            polling_tasks: tokio::task::JoinSet::new(),
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

        self.vpn_api_client
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
        tracing::info!("Requesting zk-nym by type: {}", ticketbook_type);

        let ecash_keypair = account
            .create_ecash_keypair()
            .map_err(Error::CreateEcashKeyPair)?;
        let expiration_date = nym_ecash_time::ecash_default_expiration_date();

        let (withdrawal_request, request_info) = nym_compact_ecash::withdrawal_request(
            ecash_keypair.secret_key(),
            expiration_date.ecash_unix_timestamp(),
            ticketbook_type.encode(),
        )
        .map_err(Error::ConstructWithdrawalRequest)?;

        let ecash_pubkey = ecash_keypair.public_key().to_base58_string();

        // Insert pending request into credential storage?
        // Since the id for the request is a string for the api, and a number in the storage,
        // this would be awkard.

        let response = self
            .vpn_api_client
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

        // Spawn a task to poll the status of the zk-nym
        self.spawn_polling_task(
            response.id,
            ticketbook_type,
            request_info,
            account.clone(),
            device.clone(),
        )
        .await;

        Ok(())
    }

    async fn update_pending_zk_nym_tasks(&self) {
        self.account_state
            .set_pending_zk_nym(self.is_pending_zk_nym_tasks().await)
            .await
    }

    async fn is_pending_zk_nym_tasks(&self) -> bool {
        !self.polling_tasks.is_empty()
    }

    async fn spawn_polling_task(
        &mut self,
        id: String,
        ticketbook_type: TicketType,
        request_info: RequestInfo,
        account: VpnApiAccount,
        device: Device,
    ) {
        self.account_state.set_pending_zk_nym(true).await;

        let api_client = self.vpn_api_client.clone();
        self.polling_tasks.spawn(async move {
            let start_time = Instant::now();
            loop {
                tracing::info!("polling zk-nym status: {}", id);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                match api_client.get_zk_nym_by_id(&account, &device, &id).await {
                    Ok(response) if response.status != NymVpnZkNymStatus::Pending => {
                        tracing::info!("zk-nym polling finished: {:#?}", response);
                        return PollingResult::Finished(
                            response,
                            ticketbook_type,
                            Box::new(request_info),
                        );
                    }
                    Ok(response) => {
                        tracing::info!("zk-nym polling not finished: {:#?}", response);
                        if start_time.elapsed() > Duration::from_secs(60) {
                            tracing::error!("zk-nym polling timed out: {}", id);
                            return PollingResult::Timeout(response);
                        }
                    }
                    Err(error) => {
                        tracing::error!("Failed to poll zk-nym ({}) status: {:#?}", id, error);
                        return PollingResult::Error(PollingError { id, error });
                    }
                }
            }
        });
    }

    // Check the local credential storage to see if we need to request more zk-nyms, the proceed to
    // request zk-nyms for each ticket type that we need.
    async fn request_zk_nym_all(&mut self) -> Result<(), Error> {
        tracing::info!("Requesting zk-nym (inner)");

        let account = self.load_account().await?;
        let device = self.load_device_keys().await?;

        // Then we check local storage to see what ticket types we already have stored
        let local_remaining_tickets = self.check_local_remaining_tickets().await;
        for (ticket_type, remaining) in &local_remaining_tickets {
            tracing::info!("{}, remaining: {}", ticket_type, remaining);
        }

        let ticket_types_needed_to_request = local_remaining_tickets
            .into_iter()
            .filter_map(|(ticket_type, remaining)| {
                if remaining < TICKET_THRESHOLD {
                    Some(ticket_type)
                } else {
                    None
                }
            })
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

    // Get and list zk-nyms for the device
    async fn get_device_zk_nym(&mut self) -> Result<(), Error> {
        tracing::info!("Getting device zk-nym from API");

        let account = self.load_account().await?;
        let device = self.load_device_keys().await?;

        let reported_device_zk_nyms = self
            .vpn_api_client
            .get_device_zk_nyms(&account, &device)
            .await
            .map_err(Error::GetZkNyms)?;

        tracing::info!("The device as the following zk-nyms associated to it on the account:");
        // TODO: pagination
        for zk_nym in &reported_device_zk_nyms.items {
            tracing::info!("{:?}", zk_nym);
        }
        Ok(())
    }

    async fn check_local_remaining_tickets(&self) -> Vec<(TicketType, u32)> {
        // TODO: remove unwrap
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
    async fn update_ecash_epoch(&mut self) -> Result<(), Error> {
        let aggregated_coin_indices_signatures = self
            .vpn_ecash_api_client
            .get_aggregated_coin_indices_signatures()
            .await?;
        let current_epoch = aggregated_coin_indices_signatures.epoch_id;

        self.current_epoch = Some(current_epoch);
        Ok(())
    }

    async fn update_verification_key(&mut self) -> Result<(), Error> {
        tracing::info!("Updating verification key");

        let master_verification_key = self
            .vpn_ecash_api_client
            .get_master_verification_key()
            .await?;
        let aggregated_coin_indices_signatures = self
            .vpn_ecash_api_client
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

    async fn get_current_verification_key(&self) -> Result<Option<VerificationKeyAuth>, Error> {
        let current_epoch = self.current_epoch.ok_or(Error::NoEpoch)?;
        self.credential_storage
            .get_master_verification_key(current_epoch)
            .await
            .map_err(Error::from)
    }

    async fn update_coin_indices_signatures(&mut self) -> Result<(), Error> {
        tracing::info!("Updating coin indices signatures");

        let aggregated_coin_indices_signatures = self
            .vpn_ecash_api_client
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

    async fn update_expiration_date_signatures(&mut self) -> Result<(), Error> {
        tracing::info!("Updating expiration date signatures");

        let aggregated_expiration_data_signatures = self
            .vpn_ecash_api_client
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

    async fn update_account_state(&self, account: &VpnApiAccount) -> Result<(), Error> {
        tracing::info!("Updating account state");

        let response = self.vpn_api_client.get_account_summary(account).await;

        // Check if the response indicates that we are not registered
        if let Some(403) = &response.as_ref().err().and_then(extract_status_code) {
            tracing::info!("Account is not found: access denied");
            self.account_state
                .set_account(AccountState::NotRegistered)
                .await;
        }

        let account_summary = response.map_err(|source| {
            tracing::error!("Failed to get account summary: {:#?}", source);
            Error::GetAccountSummary {
                base_url: self.vpn_api_client.current_url().clone(),
                source: Box::new(source),
            }
        })?;
        tracing::info!("Account summary: {:#?}", account_summary);

        self.account_state
            .set_account(AccountState::from(account_summary.account.status))
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
            .vpn_api_client
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

    pub(crate) async fn update_shared_account_state(&mut self) -> Result<(), Error> {
        let Some(account) = self.update_mnemonic_state().await else {
            return Ok(());
        };

        self.update_account_state(&account).await?;
        self.update_device_state(&account).await?;

        tracing::info!("Current state: {}", self.shared_state().lock().await);

        self.register_device_if_ready().await?;

        Ok(())
    }

    async fn register_device_if_ready(&self) -> Result<(), Error> {
        match self.shared_state().is_ready_to_register_device().await {
            ReadyToRegisterDevice::Ready => {
                self.command_tx.send(AccountCommand::RegisterDevice)?;
            }
            device_register_state => {
                tracing::info!("Not ready to register device: {:?}", device_register_state);
            }
        }

        Ok(())
    }

    async fn import_zk_nym(
        &mut self,
        response: NymVpnZkNym,
        ticketbook_type: TicketType,
        request_info: RequestInfo,
    ) -> Result<(), Error> {
        let account = self.load_account().await?;
        let ecash_keypair = account
            .create_ecash_keypair()
            .map_err(Error::CreateEcashKeyPair)?;
        // TODO: use explicit epoch id, that we include together with the request_info
        let current_epoch = self.current_epoch.ok_or(Error::NoEpoch)?;
        // TODO: remove unwrap
        let vk_auth = self.get_current_verification_key().await?.unwrap();

        let mut partial_wallets = Vec::new();
        for blinded_share in response.blinded_shares {
            // TODO: remove unwrap
            let blinded_share: WalletShare = serde_json::from_str(&blinded_share).unwrap();

            // TODO: remove unwrap
            let blinded_sig =
                BlindedSignature::try_from_bs58(&blinded_share.bs58_encoded_share).unwrap();

            match nym_compact_ecash::issue_verify(
                &vk_auth,
                ecash_keypair.secret_key(),
                &blinded_sig,
                &request_info,
                blinded_share.node_index,
            ) {
                Ok(partial_wallet) => partial_wallets.push(partial_wallet),
                Err(err) => {
                    tracing::error!("Failed to issue verify: {:#?}", err);
                    return Err(Error::ImportZkNym(err));
                }
            }
        }

        // TODO: remove unwrap
        let aggregated_wallets = nym_compact_ecash::aggregate_wallets(
            &vk_auth,
            ecash_keypair.secret_key(),
            &partial_wallets,
            &request_info,
        )
        .unwrap();

        // TODO: remove unwrap
        let expiration_date = OffsetDateTime::parse(&response.valid_until_utc, &Rfc3339).unwrap();

        let ticketbook = IssuedTicketBook::new(
            aggregated_wallets.into_wallet_signatures(),
            current_epoch,
            ecash_keypair.into(),
            ticketbook_type,
            expiration_date.ecash_date(),
        );

        self.credential_storage
            .insert_issued_ticketbook(&ticketbook)
            .await?;

        self.zk_nym_imported.push(response.id);

        Ok(())
    }

    // Once we finish polling the result of the zk-nym request, we now import the zk-nym into the
    // local credential store
    async fn handle_polling_result(&mut self, result: Result<PollingResult, JoinError>) {
        let result = match result {
            Ok(result) => result,
            Err(err) => {
                tracing::error!("Polling task failed: {:#?}", err);
                return;
            }
        };
        match result {
            PollingResult::Finished(response, ticketbook_type, request_info) => {
                tracing::info!("Polling task finished: {:#?}", response);
                if response.status == NymVpnZkNymStatus::Active {
                    if let Err(err) = self
                        .import_zk_nym(response, ticketbook_type, *request_info)
                        .await
                    {
                        tracing::error!("Failed to import zk-nym: {:#?}", err);
                    }
                } else {
                    tracing::warn!(
                        "Polling finished with status: {:?}, not importing!",
                        response.status
                    );
                }
            }
            PollingResult::Timeout(response) => {
                tracing::info!("Polling task timed out: {:#?}", response);
            }
            PollingResult::Error(error) => {
                tracing::error!("Polling task failed for {}: {:#?}", error.id, error.error);
            }
        }
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
                tracing::debug!("Failed to update master verification key: {:?}", err)
            })
            .ok();
        self.update_coin_indices_signatures()
            .await
            .inspect_err(|err| {
                tracing::debug!("Failed to update coin indices signatures: {:?}", err)
            })
            .ok();
        self.update_expiration_date_signatures()
            .await
            .inspect_err(|err| {
                tracing::debug!("Failed to update expiration date signatures: {:?}", err)
            })
            .ok();

        // Timer to check if any zk-nym polling tasks have finished
        let mut polling_timer = tokio::time::interval(Duration::from_millis(500));

        // Timer to periodically refresh the remote account state
        let mut update_shared_account_state_timer = tokio::time::interval(Duration::from_secs(60));

        loop {
            tokio::select! {
                Some(command) = self.command_rx.recv() => {
                    if let Err(err) = self.handle_command(command).await {
                        tracing::error!("{err}");
                        tracing::debug!("{err:#?}");
                    }
                }
                _ = update_shared_account_state_timer.tick() => {
                    self.queue_command(AccountCommand::UpdateSharedAccountState);
                }
                _ = polling_timer.tick() => {
                    while let Some(result) = self.polling_tasks.try_join_next() {
                        self.handle_polling_result(result).await;
                    }
                    self.update_pending_zk_nym_tasks().await;
                }
                _ = self.cancel_token.cancelled() => {
                    tracing::trace!("Received cancellation signal");
                    if !self.polling_tasks.is_empty() {
                        tracing::info!("Waiting for polling remaining zknyms (5 secs) ...");
                        match timeout(Duration::from_secs(5), self.polling_tasks.join_all()).await {
                            Ok(_) => tracing::info!("All polling tasks finished"),
                            Err(err) => tracing::warn!("Failed to wait for polling tasks, pending zknym's not imported into local credential store! {:#?}", err),
                        }
                    }
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

    fn queue_command(&self, command: AccountCommand) {
        if let Err(err) = self.command_tx.send(command) {
            tracing::error!("Failed to queue command: {:#?}", err);
        }
    }
}

fn get_api_url() -> Result<Url, Error> {
    // TODO: remove unwrap
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
    // TODO: remove unwrap
    let nym_vpn_api_url = get_nym_vpn_api_url().unwrap();
    // TODO: remove unwrap
    nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent).unwrap()
}

fn create_ecash_api_client() -> VpnEcashApiClient {
    // TODO: remove unwrap
    let base_url = get_api_url().unwrap();
    // TODO: remove unwrap
    VpnEcashApiClient::new(base_url).unwrap()
}

fn extract_status_code<E>(err: &E) -> Option<u16>
where
    E: std::error::Error + 'static,
{
    let mut source = err.source();
    while let Some(err) = source {
        if let Some(status) = err
            .downcast_ref::<nym_vpn_api_client::HttpClientError<nym_vpn_api_client::response::NymErrorResponse>>()
            .and_then(extract_status_code_inner)
        {
            return Some(status);
        }
        source = err.source();
    }
    None
}

fn extract_status_code_inner(
    err: &nym_vpn_api_client::HttpClientError<nym_vpn_api_client::response::NymErrorResponse>,
) -> Option<u16> {
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

#[derive(Debug)]
enum PollingResult {
    Finished(NymVpnZkNym, TicketType, Box<RequestInfo>),
    Timeout(NymVpnZkNym),
    Error(PollingError),
}

#[derive(Debug)]
struct PollingError {
    id: String,
    error: VpnApiClientError,
}

// These are temporarily copy pasted here from the nym-credential-proxy. They will eventually make
// their way into the crates we use through the nym repo.

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WalletShare {
    pub node_index: u64,
    pub bs58_encoded_share: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct TicketbookWalletSharesResponse {
    epoch_id: u64,
    shares: Vec<WalletShare>,
    master_verification_key: Option<MasterVerificationKeyResponse>,
    aggregated_coin_index_signatures: Option<AggregatedCoinIndicesSignaturesResponse>,
    aggregated_expiration_date_signatures: Option<AggregatedExpirationDateSignaturesResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct MasterVerificationKeyResponse {
    epoch_id: u64,
    bs58_encoded_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AggregatedCoinIndicesSignaturesResponse {
    signatures: AggregatedCoinIndicesSignatures,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AggregatedExpirationDateSignaturesResponse {
    signatures: AggregatedExpirationDateSignatures,
}
