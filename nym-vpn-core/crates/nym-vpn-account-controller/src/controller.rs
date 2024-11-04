// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use futures::StreamExt;
use nym_compact_ecash::Base58;
use nym_config::defaults::NymNetworkDetails;
use nym_credentials::EpochVerificationKey;
use nym_credentials_interface::{RequestInfo, TicketType, VerificationKeyAuth};
use nym_ecash_time::EcashTime;
use nym_http_api_client::UserAgent;
use nym_vpn_api_client::{
    response::{
        NymVpnAccountSummaryResponse, NymVpnDevicesResponse, NymVpnZkNym, NymVpnZkNym2,
        NymVpnZkNymStatus,
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
    error::Error,
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

    // The current state of the account
    account_state: SharedAccountState,

    // The last account summary we received from the API. We use this to check if the account state
    // has changed.
    last_account_summary: AccountSummaryResponse,

    // The last devices we received from the API. We use this to check if the device state has
    // changed.
    last_devices: DevicesResponse,

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
            account_state: SharedAccountState::new(),
            last_account_summary: Arc::new(tokio::sync::Mutex::new(None)),
            last_devices: Arc::new(tokio::sync::Mutex::new(None)),
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

    async fn new_command_handler(&self, command: AccountCommand) -> Result<CommandHandler, Error> {
        Ok(CommandHandler::new(
            command,
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
        let command_handler = self.new_command_handler(command).await?;
        self.command_tasks.spawn(command_handler.run());
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
        response: NymVpnZkNym2,
        ticketbook_type: TicketType,
        request_info: RequestInfo,
        request: ZkNymRequestData,
    ) -> Result<(), Error> {
        tracing::info!("Importing zk-nym: {}", response.id);

        let account = self.account_storage.load_account().await?;

        let Some(ref shares) = response.blinded_shares else {
            return Err(Error::MissingBlindedShares);
        };

        let issuers = self
            .vpn_api_client
            .get_directory_zk_nyms_ticketbookt_partial_verification_keys()
            .await
            .map_err(Error::GetZkNyms)?;

        if shares.epoch_id != issuers.epoch_id {
            return Err(Error::InconsistentEpochId);
        }

        tracing::info!("epoch_id: {}", shares.epoch_id);

        let master_vk_bs58 = shares
            .master_verification_key
            .clone()
            .ok_or(Error::MissingMasterVerificationKey)?
            .bs58_encoded_key;

        let master_vk = VerificationKeyAuth::try_from_bs58(&master_vk_bs58)
            .map_err(Error::InvalidMasterVerificationKey)?;

        // dbg!(&response.valid_until_utc);
        //let format = "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]Z";
        //let expiration_date = OffsetDateTime::parse(
        //    &response.valid_until_utc,
        //    &time::format_description::parse(format).unwrap(),
        //)
        //.map_err(Error::InvalidExpirationDate)?;
        //let expiration_date = OffsetDateTime::parse(&response.valid_until_utc, &Rfc3339)
        //    .map_err(Error::InvalidExpirationDate)?;
        //let expiration_date = time::Date::parse(&response.valid_until_utc, &Rfc3339)
        //    .map_err(Error::InvalidExpirationDate)?;
        //let format = time::format_description::parse(
        //    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]Z",
        //)
        //.unwrap();
        //let expiration_date = time::Date::parse(&response.valid_until_utc, &format).unwrap();
        let expiration_date = request.expiration_date;

        let issued_ticketbook = crate::commands::zknym::unblind_and_aggregate(
            shares.clone(),
            issuers,
            master_vk.clone(),
            ticketbook_type,
            expiration_date.ecash_date(),
            request_info,
            account,
        )
        .await?;

        // Insert master verification key
        tracing::info!("Inserting master verification key");
        let epoch_vk = EpochVerificationKey {
            epoch_id: shares.epoch_id,
            key: master_vk,
        };
        self.credential_storage
            .insert_master_verification_key(&epoch_vk)
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to insert master verification key: {:#?}", err);
            })
            .ok();

        // Insert aggregated coin index signatures
        tracing::info!("Inserting coin index signatures");
        self.credential_storage
            .insert_coin_index_signatures(
                &shares
                    .aggregated_coin_index_signatures
                    .clone()
                    .unwrap()
                    .signatures,
            )
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to insert coin index signatures: {:#?}", err);
            })
            .ok();

        // Insert aggregated expiration date signatures
        tracing::info!("Inserting expiration date signatures");
        self.credential_storage
            .insert_expiration_date_signatures(
                &shares
                    .aggregated_expiration_date_signatures
                    .clone()
                    .unwrap()
                    .signatures,
            )
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to insert expiration date signatures: {:#?}", err);
            })
            .ok();

        tracing::info!("Inserting issued ticketbook");
        self.credential_storage
            .insert_issued_ticketbook(&issued_ticketbook)
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
        let _ticket_types_needed_to_request = local_remaining_tickets
            .into_iter()
            .filter(|(_, remaining)| *remaining < TICKET_THRESHOLD)
            .map(|(ticket_type, _)| ticket_type)
            .collect::<Vec<_>>();

        // For testing: uncomment to only request zk-nyms for a specific ticket type
        let ticket_types_needed_to_request = vec![TicketType::V1MixnetEntry];

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

    async fn handle_get_zk_nyms_available_for_download(&self) -> Result<(), Error> {
        tracing::info!("Getting zk-nyms available for download from API");

        let account = self.account_storage.load_account().await?;
        let device = self.account_storage.load_device_keys().await?;

        let reported_device_zk_nyms = self
            .vpn_api_client
            .get_zk_nyms_available_for_download(&account, &device)
            .await
            .map_err(Error::GetZkNyms)?;

        tracing::info!("The device as the following zk-nyms available to download:");
        // TODO: pagination
        for zk_nym in &reported_device_zk_nyms.items {
            tracing::info!("{:?}", zk_nym);
        }

        Ok(())
    }

    async fn handle_get_zk_nym_by_id(&self, id: &str) -> Result<(), Error> {
        tracing::info!("Getting zk-nym by id from API");

        let account = self.account_storage.load_account().await?;
        let device = self.account_storage.load_device_keys().await?;

        let reported_device_zk_nyms = self
            .vpn_api_client
            .get_zk_nym_by_id(&account, &device, id)
            .await
            .map_err(Error::GetZkNyms)?;

        tracing::info!(
            "The device as the following zk-nym available to download: {:#?}",
            reported_device_zk_nyms
        );

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
            PollingResult::Finished(response, ticketbook_type, request_info, request)
                if response.status == NymVpnZkNymStatus::Active =>
            {
                tracing::info!("Polling finished succesfully, importing ticketbook");
                self.import_zk_nym(response, ticketbook_type, *request_info, request)
                    .await
                    .inspect_err(|err| {
                        tracing::error!("Failed to import zk-nym: {:#?}", err);
                    })
                    .ok();
            }
            PollingResult::Finished(response, _, _, _) => {
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
            AccountCommand::GetZkNymsAvailableForDownload => {
                self.handle_get_zk_nyms_available_for_download().await
            }
            AccountCommand::GetZkNymById(id) => self.handle_get_zk_nym_by_id(&id).await,
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

        // Timer to check if any command tasks have finished
        let mut command_finish_timer = tokio::time::interval(Duration::from_millis(500));

        // Timer to check if any zk-nym polling tasks have finished
        let mut polling_timer = tokio::time::interval(Duration::from_millis(500));

        // Timer to periodically refresh the remote account state
        let mut update_account_state_timer = tokio::time::interval(Duration::from_secs(5 * 60));

        tracing::info!("Account controller starting loop");
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
