// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use futures::StreamExt;
use nym_compact_ecash::{Base58 as _, BlindedSignature, VerificationKeyAuth};
use nym_config::defaults::NymNetworkDetails;
use nym_credentials::IssuedTicketBook;
use nym_credentials_interface::{RequestInfo, TicketType};
use nym_ecash_time::EcashTime;
use nym_http_api_client::UserAgent;
use nym_vpn_api_client::{
    response::{
        NymVpnAccountSummaryResponse, NymVpnDevicesResponse, NymVpnZkNym, NymVpnZkNymStatus,
    },
    types::{Device, VpnApiAccount},
};
use nym_vpn_store::VpnStorage;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::{JoinError, JoinSet},
};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{
    commands::{
        zknym::{
            construct_zk_nym_request_data, poll_zk_nym, request_zk_nym, PollingResult,
            ZkNymRequestData,
        },
        AccountCommand, AccountCommandResult, CommandHandler,
    },
    ecash_client::VpnEcashApiClient,
    error::Error,
    models::WalletShare,
    shared_state::{MnemonicState, ReadyToRegisterDevice, SharedAccountState},
    storage::{AccountStorage, VpnCredentialStorage},
};

// If we go below this threshold, we should request more tickets
// TODO: I picked a random number, check what is should be. Or if we can express this in terms of
// data/bandwidth.
const TICKET_THRESHOLD: u32 = 10;

pub(crate) type PendingCommands = Arc<std::sync::Mutex<HashMap<uuid::Uuid, AccountCommand>>>;
pub(crate) type DevicesResponse = Arc<tokio::sync::Mutex<Option<NymVpnDevicesResponse>>>;
pub(crate) type AccountSummaryResponse =
    Arc<tokio::sync::Mutex<Option<NymVpnAccountSummaryResponse>>>;

pub struct AccountController<S>
where
    S: VpnStorage,
{
    // The storage used for the account and device keys
    account_storage: AccountStorage<S>,

    // Storage used for credentials
    credential_storage: VpnCredentialStorage,

    // The API client used to interact with the nym-vpn-api
    vpn_api_client: nym_vpn_api_client::VpnApiClient,

    // The API client used to interact with cash endpoints
    // NOTE: this is a temporary solution until the data is available on the vpn-api
    vpn_ecash_api_client: VpnEcashApiClient,

    // The current state of the account
    account_state: SharedAccountState,

    // The last account summary we received from the API. We use this to check if the account state
    // has changed.
    last_account_summary: AccountSummaryResponse,

    // The last devices we received from the API. We use this to check if the device state has
    // changed.
    last_devices: DevicesResponse,

    // Keep track of the current ecash epoch
    current_epoch: Option<u64>,

    // Tasks used to poll the status of zk-nyms
    polling_tasks: JoinSet<PollingResult>,

    // Receiver channel used to receive commands from the consumer
    command_rx: UnboundedReceiver<AccountCommand>,

    // Sender channel primarily used when the consumer requests a channel to talk to the
    // controller, but also to queue up commands to itself
    command_tx: UnboundedSender<AccountCommand>,

    // Listen for cancellation signals
    cancel_token: CancellationToken,

    // Command tasks that are currently running
    command_tasks: JoinSet<Result<AccountCommandResult, Error>>,

    // List of currently running command tasks and their type
    pending_commands: PendingCommands,
}

impl<S> AccountController<S>
where
    S: VpnStorage,
{
    pub async fn new(
        storage: Arc<tokio::sync::Mutex<S>>,
        data_dir: PathBuf,
        user_agent: UserAgent,
        cancel_token: CancellationToken,
    ) -> Result<Self, Error> {
        let account_storage = AccountStorage::from(storage);

        // Generate the device keys if we don't already have them
        account_storage.init_keys().await?;

        let storage_paths =
            nym_sdk::mixnet::StoragePaths::new_from_dir(data_dir).map_err(Error::StoragePaths)?;
        let credential_storage = VpnCredentialStorage {
            storage: storage_paths
                .persistent_credential_storage()
                .await
                .map_err(Error::SetupCredentialStorage)?,
        };

        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();

        Ok(AccountController {
            account_storage,
            credential_storage,
            vpn_api_client: create_api_client(user_agent),
            vpn_ecash_api_client: create_ecash_api_client(),
            account_state: SharedAccountState::new(),
            last_account_summary: Arc::new(tokio::sync::Mutex::new(None)),
            last_devices: Arc::new(tokio::sync::Mutex::new(None)),
            current_epoch: None,
            polling_tasks: JoinSet::new(),
            command_rx,
            command_tx,
            cancel_token,
            pending_commands: Default::default(),
            command_tasks: JoinSet::new(),
        })
    }

    pub fn shared_state(&self) -> SharedAccountState {
        self.account_state.clone()
    }

    pub fn command_tx(&self) -> UnboundedSender<AccountCommand> {
        self.command_tx.clone()
    }

    async fn new_command_handler(&self) -> Result<CommandHandler, Error> {
        Ok(CommandHandler::new(
            self.account_storage.load_account().await?,
            self.account_storage.load_device_keys().await?,
            self.pending_commands.clone(),
            self.account_state.clone(),
            self.vpn_api_client.clone(),
            self.last_account_summary.clone(),
            self.last_devices.clone(),
        ))
    }

    async fn spawn_command_task(&mut self, command: AccountCommand) -> Result<(), Error> {
        tracing::debug!("Spawning command: {:?}", command);
        let command_handler = self.new_command_handler().await?;
        self.command_tasks.spawn(command_handler.run(command));
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
        request: ZkNymRequestData,
        response: NymVpnZkNym,
        account: VpnApiAccount,
        device: Device,
    ) {
        let api_client = self.vpn_api_client.clone();
        self.account_state.set_pending_zk_nym(true).await;
        self.polling_tasks
            .spawn(poll_zk_nym(request, response, account, device, api_client));
    }

    async fn import_zk_nym(
        &mut self,
        response: NymVpnZkNym,
        ticketbook_type: TicketType,
        request_info: RequestInfo,
    ) -> Result<(), Error> {
        let account = self.account_storage.load_account().await?;
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

        Ok(())
    }

    // Check the local credential storage to see if we need to request more zk-nyms, the proceed to
    // request zk-nyms for each ticket type that we need.
    async fn handle_request_zk_nym(&mut self) -> Result<(), Error> {
        let account = self.account_storage.load_account().await?;
        let device = self.account_storage.load_device_keys().await?;

        // Then we check local storage to see what ticket types we already have stored
        let local_remaining_tickets = self
            .credential_storage
            .check_local_remaining_tickets()
            .await;
        for (ticket_type, remaining) in &local_remaining_tickets {
            tracing::info!("{}, remaining: {}", ticket_type, remaining);
        }

        // Get the ticket types that are below the threshold
        let ticket_types_needed_to_request = local_remaining_tickets
            .into_iter()
            .filter(|(_, remaining)| *remaining < TICKET_THRESHOLD)
            .map(|(ticket_type, _)| ticket_type)
            .collect::<Vec<_>>();

        // Request zk-nyms for each ticket type that we need
        let responses = futures::stream::iter(ticket_types_needed_to_request)
            .filter_map(|ticket_type| {
                let account = account.clone();
                async move { construct_zk_nym_request_data(&account, ticket_type).ok() }
            })
            .map(|request| {
                let account = account.clone();
                let device = device.clone();
                let vpn_api_client = self.vpn_api_client.clone();
                async move { request_zk_nym(request, &account, &device, &vpn_api_client).await }
            })
            .buffer_unordered(4)
            .collect::<Vec<_>>()
            .await;

        // Spawn polling tasks for each zk-nym request to monitor the outcome
        for (request, response) in responses {
            match response {
                Ok(response) => {
                    self.spawn_polling_task(request, response, account.clone(), device.clone())
                        .await;
                }
                Err(err) => {
                    tracing::error!("Failed to request zk-nym: {:#?}", err);
                }
            }
        }

        Ok(())
    }

    #[allow(unused)]
    async fn update_ecash_epoch(&mut self) -> Result<(), Error> {
        self.current_epoch = self
            .vpn_ecash_api_client
            .get_aggregated_coin_indices_signatures()
            .await
            .ok()
            .map(|response| response.epoch_id);
        Ok(())
    }

    async fn update_verification_key(&mut self) -> Result<(), Error> {
        tracing::debug!("Updating verification key");
        let verification_key = self
            .vpn_ecash_api_client
            .get_master_verification_key()
            .await?;
        self.credential_storage
            .insert_master_verification_key(&verification_key)
            .await
    }

    async fn get_current_verification_key(&self) -> Result<Option<VerificationKeyAuth>, Error> {
        let current_epoch = self.current_epoch.ok_or(Error::NoEpoch)?;
        self.credential_storage
            .get_master_verification_key(current_epoch)
            .await
    }

    async fn update_coin_indices_signatures(&mut self) -> Result<(), Error> {
        tracing::debug!("Updating coin indices signatures");
        let coin_indices_signatures = self
            .vpn_ecash_api_client
            .get_aggregated_coin_indices_signatures()
            .await?;
        self.credential_storage
            .insert_coin_index_signatures(&coin_indices_signatures)
            .await
    }

    async fn update_expiration_date_signatures(&mut self) -> Result<(), Error> {
        tracing::debug!("Updating expiration date signatures");
        let expiration_date_signatures = self
            .vpn_ecash_api_client
            .get_aggregated_expiration_data_signatures()
            .await?;
        self.credential_storage
            .insert_expiration_date_signatures(&expiration_date_signatures)
            .await
    }

    async fn update_mnemonic_state(&self) -> Option<VpnApiAccount> {
        match self.account_storage.load_account().await {
            Ok(account) => {
                tracing::debug!("Our account id: {}", account.id());
                self.account_state
                    .set_mnemonic(MnemonicState::Stored { id: account.id() })
                    .await;
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

    async fn register_device_if_ready(&self) -> Result<(), Error> {
        match self.shared_state().is_ready_to_register_device().await {
            ReadyToRegisterDevice::Ready => {
                self.queue_command(AccountCommand::RegisterDevice);
            }
            ReadyToRegisterDevice::DeviceAlreadyRegistered => {
                tracing::debug!("Skipping device registration, already registered");
            }
            s @ ReadyToRegisterDevice::NoMnemonicStored
            | s @ ReadyToRegisterDevice::AccountNotActive
            | s @ ReadyToRegisterDevice::NoActiveSubscription => {
                tracing::info!("Not ready to register device: {}", s);
            }
            s @ ReadyToRegisterDevice::DeviceInactive
            | s @ ReadyToRegisterDevice::DeviceDeleted => {
                tracing::info!("Skipping registering device: {}", s);
            }
        }

        Ok(())
    }

    async fn handle_update_account_state(&mut self) -> Result<(), Error> {
        let Some(_account) = self.update_mnemonic_state().await else {
            return Ok(());
        };
        self.spawn_command_task(AccountCommand::UpdateAccountState)
            .await
    }

    async fn handle_register_device(&mut self) -> Result<(), Error> {
        tracing::debug!("Registering device");
        self.spawn_command_task(AccountCommand::RegisterDevice)
            .await
            .ok();
        Ok(())
    }

    // Get and list zk-nyms for the device
    async fn handle_get_device_zk_nym(&mut self) -> Result<(), Error> {
        tracing::info!("Getting device zk-nym from API");

        let account = self.account_storage.load_account().await?;
        let device = self.account_storage.load_device_keys().await?;

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

    // Once we finish polling the result of the zk-nym request, we now import the zk-nym into the
    // local credential store
    async fn handle_polling_result(&mut self, result: Result<PollingResult, JoinError>) {
        let Ok(result) = result else {
            tracing::error!("Polling task failed: {:#?}", result);
            return;
        };

        match result {
            PollingResult::Finished(response, ticketbook_type, request_info)
                if response.status == NymVpnZkNymStatus::Active =>
            {
                tracing::info!("Polling finished succesfully, importing ticketbook");
                self.import_zk_nym(response, ticketbook_type, *request_info)
                    .await
                    .inspect_err(|err| {
                        tracing::error!("Failed to import zk-nym: {:#?}", err);
                    })
                    .ok();
            }
            PollingResult::Finished(response, _, _) => {
                tracing::warn!(
                    "Polling finished with status: {:?}, not importing!",
                    response.status
                );
            }
            PollingResult::Timeout(response) => {
                tracing::info!("Polling task timed out: {:#?}", response);
            }
            PollingResult::Error(error) => {
                tracing::error!("Polling task failed for {}: {:#?}", error.id, error.error);
            }
        }
    }

    async fn is_command_running(&self, command: &AccountCommand) -> Result<bool, Error> {
        self.pending_commands
            .lock()
            .map(|guard| {
                guard
                    .values()
                    .any(|running_command| running_command == command)
            })
            .map_err(Error::internal)
    }

    fn queue_command(&self, command: AccountCommand) {
        if let Err(err) = self.command_tx.send(command) {
            tracing::error!("Failed to queue command: {:#?}", err);
        }
    }

    async fn handle_command(&mut self, command: AccountCommand) -> Result<(), Error> {
        tracing::info!("Received command: {:?}", command);

        if self.is_command_running(&command).await? {
            tracing::info!("Command already running, skipping: {:?}", command);
            return Ok(());
        }

        match command {
            AccountCommand::UpdateAccountState => self.handle_update_account_state().await,
            AccountCommand::RegisterDevice => self.handle_register_device().await,
            AccountCommand::RequestZkNym => self.handle_request_zk_nym().await,
            AccountCommand::GetDeviceZkNym => self.handle_get_device_zk_nym().await,
        }
    }

    async fn handle_command_result(
        &self,
        result: Result<Result<AccountCommandResult, Error>, JoinError>,
    ) {
        let Ok(result) = result else {
            tracing::error!("Polling task failed: {:#?}", result);
            return;
        };

        let result = match result {
            Ok(result) => result,
            Err(err) => {
                tracing::warn!("Command failed: {:#?}", err);
                return;
            }
        };

        match result {
            AccountCommandResult::UpdatedAccountState => {
                tracing::debug!("Account state updated");
                self.register_device_if_ready().await.ok();
            }
            AccountCommandResult::RegisteredDevice(registered_device) => {
                tracing::info!("Device registered: {:#?}", registered_device);
                self.queue_command(AccountCommand::UpdateAccountState);
            }
        }
    }

    async fn cleanup(mut self) {
        let timeout = tokio::time::sleep(Duration::from_secs(5));
        tokio::pin!(timeout);
        while !self.command_tasks.is_empty() && !self.polling_tasks.is_empty() {
            tokio::select! {
                _ = &mut timeout => {
                    tracing::warn!("Timeout waiting for polling tasks to finish, pending zk-nym's not imported into local credential store!");
                    break;
                },
                Some(result) = self.command_tasks.join_next() => {
                    self.handle_command_result(result).await
                },
                Some(result) = self.polling_tasks.join_next() =>  {
                    self.handle_polling_result(result).await
                },
            }
        }
    }

    async fn print_info(&self) {
        let account_id = self
            .account_storage
            .load_account()
            .await
            .map(|account| account.id())
            .ok()
            .unwrap_or_else(|| "(unset)".to_string());
        let device_id = self
            .account_storage
            .load_device_keys()
            .await
            .map(|device| device.identity_key().to_string())
            .ok()
            .unwrap_or_else(|| "(unset)".to_string());

        tracing::info!("Account id: {}", account_id);
        tracing::info!("Device id: {}", device_id);

        if let Err(err) = self.credential_storage.print_info().await {
            tracing::error!("Failed to print credential storage info: {:#?}", err);
        }
    }

    pub async fn run(mut self) {
        self.print_info().await;

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

        // Timer to check if any command tasks have finished
        let mut command_finish_timer = tokio::time::interval(Duration::from_millis(500));

        // Timer to check if any zk-nym polling tasks have finished
        let mut polling_timer = tokio::time::interval(Duration::from_millis(500));

        // Timer to periodically refresh the remote account state
        let mut update_account_state_timer = tokio::time::interval(Duration::from_secs(5 * 60));

        loop {
            tokio::select! {
                // Handle incoming commands
                Some(command) = self.command_rx.recv() => {
                    if let Err(err) = self.handle_command(command).await {
                        tracing::error!("{err}");
                        tracing::debug!("{err:#?}");
                    }
                }
                // Check the results of finished tasks
                _ = command_finish_timer.tick() => {
                    while let Some(result) = self.command_tasks.try_join_next() {
                        self.handle_command_result(result).await;
                    }
                }
                // Check the results of finished zk nym polling tasks
                _ = polling_timer.tick() => {
                    while let Some(result) = self.polling_tasks.try_join_next() {
                        self.handle_polling_result(result).await;
                    }
                    self.update_pending_zk_nym_tasks().await;
                }
                // On a timer we want to refresh the account state
                _ = update_account_state_timer.tick() => {
                    self.queue_command(AccountCommand::UpdateAccountState);
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

        self.cleanup().await;
        tracing::debug!("Account controller is exiting");
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
