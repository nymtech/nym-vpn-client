// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, sync::Arc, time::Duration};

use futures::StreamExt;
use nym_compact_ecash::Base58;
use nym_config::defaults::NymNetworkDetails;
use nym_credentials::EpochVerificationKey;
use nym_credentials_interface::{RequestInfo, TicketType, VerificationKeyAuth};
use nym_ecash_time::EcashTime;
use nym_http_api_client::UserAgent;
use nym_vpn_api_client::{
    response::{NymVpnZkNym, NymVpnZkNymPost, NymVpnZkNymStatus},
    types::{Device, VpnApiAccount},
};
use nym_vpn_store::VpnStorage;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::{JoinError, JoinSet},
};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{
    commands::{
        register_device::RegisterDeviceCommandHandler,
        update_account::WaitingUpdateAccountCommandHandler,
        update_device::WaitingUpdateDeviceCommandHandler,
        zknym::{
            construct_zk_nym_request_data, poll_zk_nym, request_zk_nym, PollingResult,
            ZkNymRequestData,
        },
        AccountCommand, AccountCommandError, AccountCommandResult, Command, RunningCommands,
    },
    error::Error,
    shared_state::{MnemonicState, ReadyToRegisterDevice, ReadyToRequestZkNym, SharedAccountState},
    storage::{AccountStorage, VpnCredentialStorage},
    AccountControllerCommander, AvailableTicketbooks, ReadyToConnect,
};

// The interval at which we automatically request zk-nyms
const ZK_NYM_AUTOMATIC_REQUEST_INTERVAL: Duration = Duration::from_secs(10 * 60);

// The maximum number of zk-nym requests that can fail in a row before we disable background
// refresh
const ZK_NYM_MAX_FAILS: u32 = 10;

// The interval at which we update the account state
const ACCOUNT_UPDATE_INTERVAL: Duration = Duration::from_secs(5 * 60);

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

    // Receiver channel used to receive commands from the consumer
    command_rx: UnboundedReceiver<AccountCommand>,

    // Sender channel primarily used when the consumer requests a channel to talk to the
    // controller, but also to queue up commands to itself
    command_tx: UnboundedSender<AccountCommand>,

    // List of currently running command tasks and their type
    running_commands: RunningCommands,

    // Command tasks that are currently running
    // running_command_tasks: JoinSet<Result<AccountCommandResult, Error>>,
    running_command_tasks: JoinSet<AccountCommandResult>,

    // Account update command handler state reused between runs
    waiting_update_account_command_handler: WaitingUpdateAccountCommandHandler,

    // Device update command handler state reused between runs
    waiting_update_device_command_handler: WaitingUpdateDeviceCommandHandler,

    // Tasks used to poll the status of zk-nyms
    zk_nym_polling_tasks: JoinSet<PollingResult>,

    // If we have multiple fails in a row, disable background refresh
    zk_nym_fails_in_row: u32,

    // When credential mode is disabled we don't automatically request zk-nyms. We can still do
    // so manually, but we don't want to do it automatically
    background_zk_nym_refresh: bool,

    // Listen for cancellation signals
    cancel_token: CancellationToken,
}

impl<S> AccountController<S>
where
    S: VpnStorage,
{
    pub async fn new(
        storage: Arc<tokio::sync::Mutex<S>>,
        data_dir: PathBuf,
        user_agent: UserAgent,
        credentials_mode: bool,
        cancel_token: CancellationToken,
    ) -> Result<Self, Error> {
        tracing::info!("Starting account controller");
        tracing::info!("Account controller: data directory: {:?}", data_dir);
        tracing::info!("Account controller: credential mode: {}", credentials_mode);

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

        let account_state = SharedAccountState::new();
        let vpn_api_client = create_api_client(user_agent.clone());

        let waiting_update_state_command_handler =
            WaitingUpdateAccountCommandHandler::new(account_state.clone(), vpn_api_client.clone());
        let waiting_update_device_state_command_handler =
            WaitingUpdateDeviceCommandHandler::new(account_state.clone(), vpn_api_client.clone());

        Ok(AccountController {
            account_storage,
            credential_storage,
            vpn_api_client,
            account_state,
            command_rx,
            command_tx,
            running_commands: Default::default(),
            running_command_tasks: JoinSet::new(),
            waiting_update_account_command_handler: waiting_update_state_command_handler,
            waiting_update_device_command_handler: waiting_update_device_state_command_handler,
            zk_nym_polling_tasks: JoinSet::new(),
            zk_nym_fails_in_row: 0,
            background_zk_nym_refresh: credentials_mode,
            cancel_token,
        })
    }

    pub fn shared_state(&self) -> SharedAccountState {
        self.account_state.clone()
    }

    pub fn commander(&self) -> AccountControllerCommander {
        AccountControllerCommander {
            command_tx: self.command_tx.clone(),
            shared_state: self.account_state.clone(),
        }
    }

    async fn update_pending_zk_nym_tasks(&self) {
        self.account_state
            .set_pending_zk_nym(self.is_pending_zk_nym_tasks().await)
            .await
    }

    async fn is_pending_zk_nym_tasks(&self) -> bool {
        !self.zk_nym_polling_tasks.is_empty()
    }

    async fn spawn_polling_task(
        &mut self,
        request: ZkNymRequestData,
        response: NymVpnZkNymPost,
        account: VpnApiAccount,
        device: Device,
    ) {
        let api_client = self.vpn_api_client.clone();
        self.account_state.set_pending_zk_nym(true).await;
        self.zk_nym_polling_tasks
            .spawn(poll_zk_nym(request, response, account, device, api_client));
    }

    async fn import_zk_nym(
        &mut self,
        response: NymVpnZkNym,
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

        self.confirm_zk_nym_downloaded(&response.id).await?;

        Ok(())
    }

    async fn confirm_zk_nym_downloaded(&self, id: &str) -> Result<(), Error> {
        let account = self.account_storage.load_account().await?;
        let device = self.account_storage.load_device_keys().await?;

        let response = self
            .vpn_api_client
            .confirm_zk_nym_download_by_id(&account, &device, id)
            .await
            .map_err(Error::ConfirmZkNymDownloaded)?;

        tracing::info!("Confirmed zk-nym downloaded: {:?}", response);
        Ok(())
    }

    // Check the local credential storage to see if we need to request more zk-nyms, the proceed
    // to request zk-nyms for each ticket type that we need.
    async fn handle_request_zk_nym(&mut self) -> Result<(), Error> {
        let account_state = self.shared_state().lock().await.clone();

        // Don't attempt to request zk-nyms if the account is not ready to connect
        match account_state.is_ready_to_connect(false) {
            ReadyToConnect::Ready => {}
            not_ready => {
                tracing::info!("Account not ready to request zk-nyms, skipping: {not_ready}");
                return Ok(());
            }
        }

        // Check if we are already in the process of requesting zk-nyms
        if account_state.pending_zk_nym {
            tracing::info!("zk-nym request already in progress, skipping");
            return Ok(());
        }

        tracing::info!("Checking which ticket types are running low");
        let ticket_types_needed_to_request = self
            .credential_storage
            .check_ticket_types_running_low()
            .await?;

        if ticket_types_needed_to_request.is_empty() {
            tracing::info!("No ticket types running low, skipping zk-nym request");
            return Ok(());
        }

        tracing::info!(
            "Ticket types running low: {:?}",
            ticket_types_needed_to_request
        );

        let account = self.account_storage.load_account().await?;
        let device = self.account_storage.load_device_keys().await?;

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

    async fn update_mnemonic_state(&self) -> Result<VpnApiAccount, Error> {
        match self.account_storage.load_account().await {
            Ok(account) => {
                tracing::debug!("Our account id: {}", account.id());
                self.account_state
                    .set_mnemonic(MnemonicState::Stored { id: account.id() })
                    .await;
                Ok(account)
            }
            Err(err) => {
                tracing::debug!("No account stored: {}", err);
                self.account_state
                    .set_mnemonic(MnemonicState::NotStored)
                    .await;
                Err(err)
            }
        }
    }

    async fn register_device_if_ready(&self) -> Result<(), Error> {
        match self.shared_state().is_ready_to_register_device().await {
            ReadyToRegisterDevice::Ready => {
                self.queue_command(AccountCommand::RegisterDevice(None));
            }
            not_ready => {
                tracing::info!("Not trying to register device: {not_ready}");
            }
        }
        Ok(())
    }

    fn is_background_zk_nym_refresh(&self) -> bool {
        self.zk_nym_fails_in_row < ZK_NYM_MAX_FAILS && self.background_zk_nym_refresh
    }

    async fn request_zk_nym_if_ready(&self) -> Result<(), Error> {
        if !self.is_background_zk_nym_refresh() {
            return Ok(());
        }
        match self.shared_state().is_ready_to_request_zk_nym().await {
            ReadyToRequestZkNym::Ready => {
                self.queue_command(AccountCommand::RequestZkNym);
            }
            not_ready => {
                tracing::info!("Not trying to request zk-nym: {not_ready}");
            }
        }
        Ok(())
    }

    async fn handle_update_account_state(&mut self, command: AccountCommand) {
        let account = self
            .update_mnemonic_state()
            .await
            .map_err(|_err| AccountCommandError::NoAccountStored);

        let account = match account {
            Ok(account) => account,
            Err(err) => {
                command.return_error(err);
                return;
            }
        };

        let command_handler = self.waiting_update_account_command_handler.build(account);

        if self.running_commands.add(command).await == Command::IsFirst {
            self.running_command_tasks.spawn(command_handler.run());
        }
    }

    async fn handle_update_device_state(&mut self, command: AccountCommand) {
        let account = self
            .update_mnemonic_state()
            .await
            .map_err(|_err| AccountCommandError::NoAccountStored);

        let account = match account {
            Ok(account) => account,
            Err(err) => {
                command.return_error(err);
                return;
            }
        };

        let device = self
            .account_storage
            .load_device_keys()
            .await
            .map_err(|_err| AccountCommandError::NoDeviceStored);

        let device = match device {
            Ok(device) => device,
            Err(err) => {
                command.return_error(err);
                return;
            }
        };

        let command_handler = self
            .waiting_update_device_command_handler
            .build(account, device);

        if self.running_commands.add(command).await == Command::IsFirst {
            self.running_command_tasks.spawn(command_handler.run());
        }
    }

    async fn handle_register_device(&mut self, command: AccountCommand) {
        let account = self
            .update_mnemonic_state()
            .await
            .map_err(|_err| AccountCommandError::NoAccountStored);

        let account = match account {
            Ok(account) => account,
            Err(err) => {
                command.return_error(err);
                return;
            }
        };

        let device = self
            .account_storage
            .load_device_keys()
            .await
            .map_err(|_err| AccountCommandError::NoDeviceStored);

        let device = match device {
            Ok(device) => device,
            Err(err) => {
                command.return_error(err);
                return;
            }
        };

        let command_handler = RegisterDeviceCommandHandler::new(
            account,
            device,
            self.account_state.clone(),
            self.vpn_api_client.clone(),
        );
        if self.running_commands.add(command).await == Command::IsFirst {
            self.running_command_tasks.spawn(command_handler.run());
        }
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

    async fn handle_get_available_tickets(&self) -> Result<AvailableTicketbooks, Error> {
        tracing::info!("Getting available tickets from local credential storage");
        self.credential_storage.print_info().await?;
        self.credential_storage.get_available_ticketbooks().await
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
                let id = response.id.clone();
                tracing::info!("Polling finished succesfully, importing ticketbook: {id}",);
                match self
                    .import_zk_nym(response, ticketbook_type, *request_info, *request)
                    .await
                {
                    Ok(_) => {
                        tracing::info!("Successfully imported zk-nym: {}", id);
                        self.zk_nym_fails_in_row = 0;
                    }
                    Err(err) => {
                        tracing::error!("Failed to import zk-nym: {:#?}", err);
                        self.zk_nym_fails_in_row += 1;
                    }
                };
            }
            PollingResult::Finished(response, _, _, _) => {
                tracing::warn!(
                    "Polling for {} finished with status: {:?}",
                    response.id,
                    response.status,
                );
                tracing::warn!("Not importing zk-nym: {}", response.id);
                self.zk_nym_fails_in_row += 1;
            }
            PollingResult::Timeout(response) => {
                tracing::info!("Polling task timed out: {:#?}", response);
                self.zk_nym_fails_in_row += 1;
            }
            PollingResult::Error(error) => {
                tracing::error!("Polling task failed for {}: {:#?}", error.id, error.error);
                self.zk_nym_fails_in_row += 1;
            }
        }
    }

    fn queue_command(&self, command: AccountCommand) {
        if let Err(err) = self.command_tx.send(command) {
            tracing::error!("Failed to queue command: {:#?}", err);
        }
    }

    async fn handle_command(&mut self, command: AccountCommand) {
        tracing::info!("Received command: {}", command);
        match command {
            AccountCommand::ResetAccount => {
                self.account_state.reset().await;
            }
            AccountCommand::UpdateAccountState(_) => {
                self.handle_update_account_state(command).await;
            }
            AccountCommand::UpdateDeviceState(_) => {
                self.handle_update_device_state(command).await;
            }
            AccountCommand::RegisterDevice(_) => {
                self.handle_register_device(command).await;
            }
            AccountCommand::RequestZkNym => {
                self.handle_request_zk_nym()
                    .await
                    .inspect_err(|err| {
                        tracing::error!("Failed to request zk-nym: {:#?}", err);
                    })
                    .ok();
            }
            AccountCommand::GetDeviceZkNym => {
                self.handle_get_device_zk_nym()
                    .await
                    .inspect_err(|err| {
                        tracing::error!("Failed to get device zk-nym: {:#?}", err);
                    })
                    .ok();
            }
            AccountCommand::GetZkNymsAvailableForDownload => {
                self.handle_get_zk_nyms_available_for_download()
                    .await
                    .inspect_err(|err| {
                        tracing::error!("Failed to get zk-nyms available for download: {:#?}", err);
                    })
                    .ok();
            }
            AccountCommand::GetZkNymById(id) => {
                self.handle_get_zk_nym_by_id(&id)
                    .await
                    .inspect_err(|err| {
                        tracing::error!("Failed to get zk-nym by id: {:#?}", err);
                    })
                    .ok();
            }
            AccountCommand::ConfirmZkNymIdDownloaded(id) => {
                self.confirm_zk_nym_downloaded(&id)
                    .await
                    .inspect_err(|err| {
                        tracing::error!("Failed to confirm zk-nym downloaded: {:#?}", err);
                    })
                    .ok();
            }
            AccountCommand::GetAvailableTickets(result_tx) => {
                let result = self.handle_get_available_tickets().await;
                result_tx
                    .send(result)
                    .inspect_err(|err| {
                        tracing::error!("Failed to send available tickets response: {:#?}", err);
                    })
                    .ok();
            }
        };
    }

    async fn handle_command_result(&self, result: Result<AccountCommandResult, JoinError>) {
        let Ok(result) = result else {
            tracing::error!("Joining task failed: {:#?}", result);
            return;
        };

        match result {
            AccountCommandResult::UpdateAccountState(r) => {
                tracing::debug!("Account state completed: {:?}", r);
                let commands = self
                    .running_commands
                    .remove(&AccountCommand::UpdateAccountState(None))
                    .await;
                for command in commands {
                    if let AccountCommand::UpdateAccountState(Some(tx)) = command {
                        tx.send(r.clone());
                    }
                }
                if r.is_ok() {
                    self.register_device_if_ready().await.ok();
                    self.request_zk_nym_if_ready().await.ok();
                }
            }
            AccountCommandResult::UpdateDeviceState(r) => {
                tracing::debug!("Device state updated: {:?}", r);
                let commands = self
                    .running_commands
                    .remove(&AccountCommand::UpdateDeviceState(None))
                    .await;
                for command in commands {
                    if let AccountCommand::UpdateDeviceState(Some(tx)) = command {
                        tx.send(r.clone());
                    }
                }
                if r.is_ok() {
                    self.register_device_if_ready().await.ok();
                    self.request_zk_nym_if_ready().await.ok();
                }
            }
            AccountCommandResult::RegisterDevice(r) => {
                tracing::info!("Device register task: {:#?}", r);
                let commands = self
                    .running_commands
                    .remove(&AccountCommand::RegisterDevice(None))
                    .await;
                for command in commands {
                    if let AccountCommand::RegisterDevice(Some(tx)) = command {
                        tx.send(r.clone());
                    }
                }
                if r.is_ok() {
                    self.queue_command(AccountCommand::UpdateAccountState(None));
                    self.request_zk_nym_if_ready().await.ok();
                }
            }
        }
    }

    async fn cleanup(mut self) {
        let timeout = tokio::time::sleep(Duration::from_secs(5));
        tokio::pin!(timeout);
        while !self.running_command_tasks.is_empty() && !self.zk_nym_polling_tasks.is_empty() {
            tokio::select! {
                _ = &mut timeout => {
                    tracing::warn!("Timeout waiting for polling tasks to finish, pending zk-nym's not imported into local credential store!");
                    break;
                },
                Some(result) = self.running_command_tasks.join_next() => {
                    self.handle_command_result(result).await
                },
                Some(result) = self.zk_nym_polling_tasks.join_next() =>  {
                    self.handle_polling_result(result).await
                },
            }
        }
    }

    async fn print_info(&self) {
        let account_id = self
            .account_storage
            .load_account_id()
            .await
            .ok()
            .unwrap_or_else(|| "(unset)".to_string());
        let device_id = self
            .account_storage
            .load_device_id()
            .await
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

        // Timer to check if any command tasks have finished. This just needs to be something small
        // so that we periodically check the results without interfering with other tasks
        let mut command_finish_timer = tokio::time::interval(Duration::from_millis(500));

        // Timer to check if any zk-nym polling tasks have finished. This just needs to be
        // something small so that we periodically check the results without interfering with other
        // tasks
        let mut polling_timer = tokio::time::interval(Duration::from_millis(500));

        // Timer to periodically refresh the remote account state
        let mut update_account_state_timer = tokio::time::interval(ACCOUNT_UPDATE_INTERVAL);

        // Timer to periodically check if we need to request more zk-nyms
        let mut update_zk_nym_timer = tokio::time::interval(ZK_NYM_AUTOMATIC_REQUEST_INTERVAL);

        loop {
            tokio::select! {
                // Handle incoming commands
                Some(command) = self.command_rx.recv() => {
                    self.handle_command(command).await;
                }
                // Check the results of finished tasks
                _ = command_finish_timer.tick() => {
                    while let Some(result) = self.running_command_tasks.try_join_next() {
                        self.handle_command_result(result).await;
                    }
                }
                // Check the results of finished zk nym polling tasks
                _ = polling_timer.tick() => {
                    while let Some(result) = self.zk_nym_polling_tasks.try_join_next() {
                        self.handle_polling_result(result).await;
                    }
                    self.update_pending_zk_nym_tasks().await;
                }
                // On a timer we want to refresh the account and device state
                _ = update_account_state_timer.tick() => {
                    self.queue_command(AccountCommand::UpdateAccountState(None));
                    self.queue_command(AccountCommand::UpdateDeviceState(None));
                }
                // On a timer to check if we need to request more zk-nyms
                _ = update_zk_nym_timer.tick(), if self.is_background_zk_nym_refresh() => {
                    self.queue_command(AccountCommand::RequestZkNym);
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
