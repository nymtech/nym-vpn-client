// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use nym_http_api_client::UserAgent;
use nym_offline_monitor::{Connectivity, ConnectivityHandle};
use nym_vpn_api_client::{
    response::{NymVpnDevice, NymVpnUsage},
    types::{DeviceStatus, VpnApiAccount},
};
use nym_vpn_lib_types::{
    AccountCommandError, ForgetAccountError, StoreAccountError, VpnApiErrorResponse,
};
use nym_vpn_network_config::Network;
use nym_vpn_store::{mnemonic::Mnemonic, VpnStorage};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinError,
};
use tokio_util::sync::CancellationToken;

use crate::{
    commands::{AccountCommand, AccountCommandHandler, AccountCommandResult},
    connectivity::OfflineWatch,
    error::Error,
    shared_state::{MnemonicState, ReadyToRegisterDevice, ReadyToRequestZkNym, SharedAccountState},
    storage::{AccountStorage, SharedVpnCredentialStorage, VpnCredentialStorage},
    vpn_api_client::AccountControllerVpnApiClient,
    AccountCommandSender, AvailableTicketbooks,
};

// The interval at which we automatically request zk-nyms
const ZK_NYM_AUTOMATIC_REQUEST_INTERVAL: Duration = Duration::from_secs(60);

// The interval at which we update the account state
const ACCOUNT_UPDATE_INTERVAL: Duration = Duration::from_secs(5 * 60);

pub struct AccountControllerConfig {
    // The data directory where we store the account and device keys.
    pub data_dir: PathBuf,

    // User agent used by api client.
    pub user_agent: UserAgent,

    // Credentials mode is a feature flag that determines if we should automatically request
    // zk-nyms.
    pub credentials_mode: Option<bool>,

    // The network environment that the controller is running in.
    pub network_env: Network,
}

impl AccountControllerConfig {
    // Determine if the credentials mode is enabled. This is determined by the credentials_mode
    // field in the config, if it is set. Else the network environment feature flag is used.
    fn background_zk_nym_refresh(&self) -> bool {
        self.credentials_mode.unwrap_or_else(|| {
            self.network_env
                .get_feature_flag_credential_mode()
                .unwrap_or(false)
        })
    }
}

pub struct AccountController<S>
where
    S: VpnStorage,
{
    // The configuration that was used to create the controller.
    config: AccountControllerConfig,

    // The storage used for the account and device keys
    account_storage: AccountStorage<S>,

    // Storage used for credentials.
    credential_storage: SharedVpnCredentialStorage,

    // The current state of the account
    account_state: SharedAccountState,

    // The API client used to interact with the nym-vpn-api
    vpn_api_client: AccountControllerVpnApiClient,

    // Receiver channel used to receive commands from the outside.
    command_channel: (
        UnboundedSender<AccountCommand>,
        UnboundedReceiver<AccountCommand>,
    ),

    // Manage the commands that the controller is currently running
    command_handler: AccountCommandHandler,

    // Keep track of offline state
    offline_watch: OfflineWatch,

    // Listen for cancellation signals
    cancel_token: CancellationToken,
}

impl<S> AccountController<S>
where
    S: VpnStorage,
{
    pub async fn new(
        config: AccountControllerConfig,
        storage: Arc<tokio::sync::Mutex<S>>,
        initial_connectivity: Option<Connectivity>,
        cancel_token: CancellationToken,
    ) -> Result<Self, Error> {
        tracing::info!(
            "Starting account controller: data_dir: {}",
            config.data_dir.display(),
        );

        // Setup up the storage. We have both the account storage as well as the credential storage
        let (account_storage, credential_storage) = init::create_storage(&config, storage).await?;

        // Client to query the VPN API
        let vpn_api_client = AccountControllerVpnApiClient::new(&config)?;

        // We expose the account state as a shared object that can be queried without having to ask
        // the controller
        let account_state = init::create_initial_shared_state(&account_storage).await;

        // The channels used to communicate with the controller
        let command_channel = tokio::sync::mpsc::unbounded_channel();

        // Keep track of the commands that are currently running
        let command_handler = AccountCommandHandler::new(
            account_state.clone(),
            vpn_api_client.inner().clone(),
            credential_storage.clone(),
        );

        // The offline watch is used to keep track of the current connectivity state, since we
        // don't want to do certain operations when we are offline
        let offline_watch = OfflineWatch::new(
            AccountCommandSender::new(command_channel.0.clone(), account_state.clone()),
            initial_connectivity.unwrap_or(Connectivity::new_presume_offline()),
        );

        Ok(AccountController {
            config,
            account_storage,
            credential_storage,
            vpn_api_client,
            account_state,
            command_channel,
            command_handler,
            cancel_token,
            offline_watch,
        })
    }

    pub fn get_shared_state(&self) -> SharedAccountState {
        self.account_state.clone()
    }

    pub fn get_command_sender(&self) -> AccountCommandSender {
        AccountCommandSender::new(self.command_channel.0.clone(), self.account_state.clone())
    }

    async fn is_background_zk_nym_refresh_active(&self) -> bool {
        self.config.background_zk_nym_refresh()
            && !self.command_handler.max_zknym_request_fails_reached().await
    }

    async fn is_all_ticket_types_above_soft_threshold(&self) -> Result<bool, AccountCommandError> {
        self.credential_storage
            .lock()
            .await
            .is_all_ticket_types_above_soft_threshold()
            .await
            .map_err(AccountCommandError::internal)
    }

    async fn update_mnemonic_state(&self) -> Result<VpnApiAccount, Error> {
        let account = self.account_storage.load_account().await;
        match account {
            Ok(ref account) => {
                tracing::debug!("Our account id: {}", account.id());
                self.account_state
                    .set_mnemonic(MnemonicState::Stored { id: account.id() })
                    .await;
            }
            Err(ref err) => {
                tracing::debug!("No account stored: {err}");
                self.account_state.reset_to(MnemonicState::NotStored).await;
            }
        }
        account
    }

    async fn register_device_if_ready(&self) {
        if self.offline_watch.is_offline() {
            tracing::info!("Not registering device as we are offline");
            return;
        }

        match self.get_shared_state().ready_to_register_device().await {
            ReadyToRegisterDevice::Ready => {
                self.get_command_sender().background_register_device();
            }
            not_ready => {
                tracing::debug!("Not trying to register device: {not_ready}");
            }
        }
    }

    async fn request_zk_nym_if_ready(&self) {
        if self.offline_watch.is_offline() {
            tracing::info!("Not requesting zk-nym as we are offline");
            return;
        }

        if !self.is_background_zk_nym_refresh_active().await {
            return;
        }

        match self.is_all_ticket_types_above_soft_threshold().await {
            Ok(false) => (),
            Ok(true) => {
                tracing::debug!("All ticket types are above soft threshold, not requesting zk-nym");
                return;
            }
            Err(err) => {
                // Be conservative, it might be wasteful to request zknyms if we can't store them
                // locally anyway.
                tracing::error!(
                    "Failed to lookup current tickets, not requesting more zk-nyms: {err}"
                );
                return;
            }
        }

        match self.get_shared_state().ready_to_request_zk_nym().await {
            ReadyToRequestZkNym::Ready => {
                self.get_command_sender().background_request_zk_nyms();
            }
            not_ready => {
                tracing::debug!("Not ready to try to request zk-nym: {not_ready}");
            }
        }
    }

    async fn unregister_device_from_api(&self) -> Result<NymVpnDevice, AccountCommandError> {
        tracing::info!("Unregistering device from API");
        if self.get_shared_state().ready_to_register_device().await
            == ReadyToRegisterDevice::InProgress
        {
            return Err(ForgetAccountError::RegistrationInProgress.into());
        }

        let device = self
            .account_storage
            .load_device_keys()
            .await
            .map_err(|_err| AccountCommandError::NoDeviceStored)?;

        let account = self
            .account_storage
            .load_account()
            .await
            .map_err(|_err| AccountCommandError::NoAccountStored)?;

        self.vpn_api_client
            .update_device(&account, &device, DeviceStatus::DeleteMe)
            .await
            .map_err(|err| {
                VpnApiErrorResponse::try_from(err)
                    .map(ForgetAccountError::UpdateDeviceErrorResponse)
                    .unwrap_or_else(|err| ForgetAccountError::UnexpectedResponse(err.to_string()))
                    .into()
            })
    }

    async fn handle_store_account(&self, mnemonic: Mnemonic) -> Result<(), AccountCommandError> {
        if self.offline_watch.is_online() {
            self.vpn_api_client
                .check_account_exists_on_api(&VpnApiAccount::from(mnemonic.clone()))
                .await?;
        } else {
            tracing::info!("Not checking if account exists on vpn-api as we are offline");
        }

        self.account_storage
            .store_account(mnemonic)
            .await
            .map_err(|err| StoreAccountError::Storage(err.to_string()))?;

        self.update_mnemonic_state()
            .await
            .map_err(AccountCommandError::internal)?;

        if self.offline_watch.is_online() {
            // We don't need to wait for the sync to finish, so queue it up and return
            self.get_command_sender().background_sync_account_state();
            self.get_command_sender().background_sync_device_state();
        }

        Ok(())
    }

    async fn handle_forget_account(&mut self) -> Result<(), AccountCommandError> {
        tracing::info!("REMOVING ACCOUNT AND ALL ASSOCIATED DATA");

        // TODO: here we should put the controller in some sort of idle state, and wait for all
        // currently running operations to finish before proceeding with the reset

        if self.offline_watch.is_online() {
            if let Err(err) = self.unregister_device_from_api().await {
                tracing::error!("Failed to unregister device: {err}");
            } else {
                tracing::info!("Device has been unregistered");
            }
        } else {
            tracing::info!("Not unregistering device as we are offline");
        }

        self.account_storage
            .remove_account()
            .await
            .map_err(|source| {
                tracing::error!("Failed to remove account: {source:?}");
                ForgetAccountError::RemoveAccount(source.to_string())
            })?;

        self.account_storage
            .remove_device_keys()
            .await
            .map_err(|source| {
                tracing::error!("Failed to remove device identity: {source:?}");
                ForgetAccountError::RemoveDeviceKeys(source.to_string())
            })?;

        self.credential_storage
            .lock()
            .await
            .reset()
            .await
            .map_err(|source| {
                tracing::error!("Failed to reset credential storage: {source:?}");
                ForgetAccountError::ResetCredentialStorage(source.to_string())
            })?;

        // Purge all files in the data directory that we are not explicitly deleting through it's
        // owner. Ideally we should strive for this to be removed.
        // If this fails, we still need to continue with the remaining steps
        let remove_files_result = crate::storage::remove_files_for_account(&self.config.data_dir)
            .inspect_err(|err| {
                tracing::error!("Failed to remove files for account: {err:?}");
            });

        // Once we have removed or reset all storage, we need to reset the account state
        self.command_handler.reset();
        self.account_state.reset().await;

        // And now we are ready to start reconstructing
        let reinit_keys_result = self
            .account_storage
            .init_keys()
            .await
            .inspect_err(|source| {
                tracing::error!("Failed to reinitialize device keys: {source:?}");
            });

        // And conclude by syncing with the remote state
        if self.offline_watch.is_online() {
            self.handle_sync_account_state(AccountCommand::SyncAccountState(None))
                .await;
        }

        if let Err(err) = remove_files_result {
            return Err(ForgetAccountError::RemoveAccountFiles(format!(
                "Failed to remove files for account: {err}"
            ))
            .into());
        }

        if let Err(err) = reinit_keys_result {
            return Err(ForgetAccountError::InitDeviceKeys(format!(
                "Failed to reinitialize device keys: {err}"
            ))
            .into());
        }

        Ok(())
    }

    async fn handle_sync_account_state(&mut self, command: AccountCommand) {
        let account = match self.update_mnemonic_state().await {
            Ok(account) => account,
            Err(err) => {
                command.return_no_account(err);
                return;
            }
        };

        if self.offline_watch.is_offline() {
            tracing::info!("Not syncing account state as we are offline");
            command.return_no_connectivity();
            return;
        }

        self.command_handler
            .sync_account_state(command, account)
            .await;
    }

    async fn handle_sync_device_state(&mut self, command: AccountCommand) {
        let account = match self.update_mnemonic_state().await {
            Ok(account) => account,
            Err(err) => {
                command.return_no_account(err);
                return;
            }
        };

        let device = match self.account_storage.load_device_keys().await {
            Ok(device) => device,
            Err(err) => {
                command.return_no_device(err);
                return;
            }
        };

        if self.offline_watch.is_offline() {
            tracing::info!("Not syncing device state as we are offline");
            command.return_no_connectivity();
            return;
        }

        self.command_handler
            .sync_device_state(command, account, device)
            .await;
    }

    async fn handle_get_usage(&self) -> Result<Vec<NymVpnUsage>, AccountCommandError> {
        let account = self
            .account_storage
            .load_account()
            .await
            .map_err(|err| AccountCommandError::Storage(err.to_string()))?;
        if self.offline_watch.is_offline() {
            tracing::error!("Unable to get usage as we are offline");
            return Err(AccountCommandError::Offline);
        }

        let usage = self
            .vpn_api_client
            .get_usage(&account)
            .await
            .map_err(|err| {
                VpnApiErrorResponse::try_from(err)
                    .map(AccountCommandError::from)
                    .unwrap_or_else(AccountCommandError::internal)
            })?;
        tracing::info!("Usage: {:#?}", usage);
        Ok(usage.items)
    }

    async fn handle_get_device_identity(&self) -> Result<String, AccountCommandError> {
        let device = self
            .account_storage
            .load_device_id()
            .await
            .map_err(|_err| AccountCommandError::NoDeviceStored)?;

        tracing::info!("Device identity: {device:?}");
        Ok(device)
    }

    async fn handle_register_device(&mut self, command: AccountCommand) {
        let account = match self.update_mnemonic_state().await {
            Ok(account) => account,
            Err(err) => {
                command.return_no_account(err);
                return;
            }
        };

        let device = match self.account_storage.load_device_keys().await {
            Ok(device) => device,
            Err(err) => {
                command.return_no_device(err);
                return;
            }
        };

        if self.offline_watch.is_offline() {
            tracing::info!("Not registering device as we are offline");
            command.return_no_connectivity();
            return;
        }

        self.command_handler
            .register_device(
                command,
                account,
                device,
                self.account_state.clone(),
                self.vpn_api_client.inner().clone(),
            )
            .await;
    }

    async fn handle_get_devices(&mut self) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        tracing::info!("Getting devices from API");

        let account = self
            .account_storage
            .load_account()
            .await
            .map_err(|err| AccountCommandError::Storage(err.to_string()))?;

        if self.offline_watch.is_offline() {
            tracing::error!("Unable to get devices as we are offline");
            return Err(AccountCommandError::Offline);
        }

        let devices = self
            .vpn_api_client
            .get_devices(&account)
            .await
            .map_err(|err| {
                VpnApiErrorResponse::try_from(err)
                    .map(AccountCommandError::from)
                    .unwrap_or_else(AccountCommandError::internal)
            })?;

        tracing::info!("The account has the following devices associated to it:");
        // TODO: pagination
        for device in &devices.items {
            tracing::info!("{:?}", device);
        }
        Ok(devices.items)
    }

    async fn handle_get_active_devices(
        &mut self,
    ) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        tracing::info!("Getting active devices from API");

        let account = self
            .account_storage
            .load_account()
            .await
            .map_err(AccountCommandError::storage)?;

        if self.offline_watch.is_offline() {
            tracing::error!("Unable to get active devices as we are offline");
            return Err(AccountCommandError::Offline);
        }

        let devices = self
            .vpn_api_client
            .get_active_devices(&account)
            .await
            .map_err(|err| {
                VpnApiErrorResponse::try_from(err)
                    .map(AccountCommandError::from)
                    .unwrap_or_else(AccountCommandError::internal)
            })?;

        tracing::info!("The account has the following active devices associated to it:");
        // TODO: pagination
        for device in &devices.items {
            tracing::info!("{:?}", device);
        }
        Ok(devices.items)
    }

    async fn handle_request_zk_nym(&mut self, command: AccountCommand) {
        let account = match self.update_mnemonic_state().await {
            Ok(account) => account,
            Err(err) => {
                command.return_no_account(err);
                return;
            }
        };

        let device = match self.account_storage.load_device_keys().await {
            Ok(device) => device,
            Err(err) => {
                command.return_no_device(err);
                return;
            }
        };

        if self.offline_watch.is_offline() {
            tracing::info!("Not requesting zknyms as we are offline");
            command.return_no_connectivity();
            return;
        }

        self.command_handler
            .request_zk_nym(command, account, device)
            .await;
    }

    async fn handle_get_device_zk_nym(&mut self) -> Result<(), AccountCommandError> {
        tracing::info!("Getting device zk-nym from API");

        let account = self
            .account_storage
            .load_account()
            .await
            .map_err(AccountCommandError::storage)?;

        let device = self
            .account_storage
            .load_device_keys()
            .await
            .map_err(AccountCommandError::storage)?;

        if self.offline_watch.is_offline() {
            tracing::error!("Unable to get device zknyms as we are offline");
            return Err(AccountCommandError::Offline);
        }

        let reported_device_zk_nyms = self
            .vpn_api_client
            .get_device_zk_nyms(&account, &device)
            .await
            .map_err(|err| {
                VpnApiErrorResponse::try_from(err)
                    .map(AccountCommandError::from)
                    .unwrap_or_else(AccountCommandError::internal)
            })?;

        tracing::info!("The device as the following zk-nyms associated to it on the account:");
        // TODO: pagination
        for zk_nym in &reported_device_zk_nyms.items {
            tracing::info!("{:?}", zk_nym);
        }
        Ok(())
    }

    async fn handle_get_zk_nyms_available_for_download(&self) -> Result<(), AccountCommandError> {
        tracing::info!("Getting zk-nyms available for download from API");

        let account = self
            .account_storage
            .load_account()
            .await
            .map_err(AccountCommandError::storage)?;

        let device = self
            .account_storage
            .load_device_keys()
            .await
            .map_err(AccountCommandError::storage)?;

        if self.offline_watch.is_offline() {
            tracing::error!("Unable to get zknyms available for download as we are offline");
            return Err(AccountCommandError::Offline);
        }

        let reported_device_zk_nyms = self
            .vpn_api_client
            .get_zk_nyms_available_for_download(&account, &device)
            .await
            .map_err(|err| {
                VpnApiErrorResponse::try_from(err)
                    .map(AccountCommandError::from)
                    .unwrap_or_else(AccountCommandError::internal)
            })?;

        tracing::info!("The device as the following zk-nyms available to download:");
        // TODO: pagination
        for zk_nym in &reported_device_zk_nyms.items {
            tracing::info!("{:?}", zk_nym);
        }

        Ok(())
    }

    async fn handle_get_zk_nym_by_id(&self, id: &str) -> Result<(), AccountCommandError> {
        tracing::info!("Getting zk-nym by id from API");

        let account = self
            .account_storage
            .load_account()
            .await
            .map_err(AccountCommandError::storage)?;

        let device = self
            .account_storage
            .load_device_keys()
            .await
            .map_err(AccountCommandError::storage)?;

        if self.offline_watch.is_offline() {
            tracing::error!("Unable to get zknym by id as we are offline");
            return Err(AccountCommandError::Offline);
        }

        let reported_device_zk_nyms = self
            .vpn_api_client
            .get_zk_nym_by_id(&account, &device, id)
            .await
            .map_err(|err| {
                VpnApiErrorResponse::try_from(err)
                    .map(AccountCommandError::from)
                    .unwrap_or_else(AccountCommandError::internal)
            })?;

        tracing::info!(
            "The device as the following zk-nym available to download: {:#?}",
            reported_device_zk_nyms
        );

        Ok(())
    }

    async fn handle_confirm_zk_nym_downloaded(
        &self,
        id: String,
    ) -> Result<(), AccountCommandError> {
        tracing::info!("Confirming zk-nym downloaded: {}", id);

        let account = self
            .account_storage
            .load_account()
            .await
            .map_err(AccountCommandError::storage)?;

        let device = self
            .account_storage
            .load_device_keys()
            .await
            .map_err(AccountCommandError::storage)?;

        if self.offline_watch.is_offline() {
            tracing::error!("Unable to confirm zknym downloaded as we are offline");
            return Err(AccountCommandError::Offline);
        }

        let response = self
            .vpn_api_client
            .confirm_zk_nym_download_by_id(&account, &device, &id)
            .await
            .map_err(|err| {
                VpnApiErrorResponse::try_from(err)
                    .map(AccountCommandError::from)
                    .unwrap_or_else(AccountCommandError::internal)
            })?;

        tracing::info!("Confirmed zk-nym downloaded: {response:?}");

        Ok(())
    }

    async fn handle_get_available_tickets(
        &self,
    ) -> Result<AvailableTicketbooks, AccountCommandError> {
        tracing::debug!("Getting available tickets from local credential storage");
        let guard = self.credential_storage.lock().await;
        guard
            .print_info()
            .await
            .map_err(|err| AccountCommandError::Storage(err.to_string()))?;
        guard
            .get_available_ticketbooks()
            .await
            .map_err(|err| AccountCommandError::Storage(err.to_string()))
    }

    fn handle_set_static_api_addresses(
        &mut self,
        static_api_addresses: Option<Vec<SocketAddr>>,
    ) -> Result<(), AccountCommandError> {
        nym_vpn_api_client::VpnApiClient::new_with_resolver_overrides(
            self.vpn_api_client.current_url().clone(),
            self.config.user_agent.clone(),
            static_api_addresses.as_deref(),
        )
        .map(|new_vpn_api_client| {
            self.vpn_api_client
                .swap_inner_client(new_vpn_api_client.clone());
            self.command_handler
                .update_vpn_api_client(new_vpn_api_client);
        })
        .map_err(|e| AccountCommandError::internal(format!("Failed to set static addresses: {e}")))
    }

    async fn handle_register_offline_monitor(
        &mut self,
        offline_monitor: ConnectivityHandle,
    ) -> Result<(), AccountCommandError> {
        self.offline_watch
            .register_offline_monitor(offline_monitor)
            .await;
        Ok(())
    }

    async fn handle_command(&mut self, command: AccountCommand) {
        tracing::info!("← {}", command);
        match command {
            AccountCommand::StoreAccount(result_tx, mnemonic) => {
                result_tx.send(self.handle_store_account(mnemonic).await);
            }
            AccountCommand::ForgetAccount(result_tx) => {
                result_tx.send(self.handle_forget_account().await);
            }
            AccountCommand::SyncAccountState(_) => {
                self.handle_sync_account_state(command).await;
            }
            AccountCommand::SyncDeviceState(_) => {
                self.handle_sync_device_state(command).await;
            }
            AccountCommand::GetUsage(result_tx) => {
                result_tx.send(self.handle_get_usage().await);
            }
            AccountCommand::GetDeviceIdentity(result_tx) => {
                result_tx.send(self.handle_get_device_identity().await);
            }
            AccountCommand::RegisterDevice(_) => {
                self.handle_register_device(command).await;
            }
            AccountCommand::GetDevices(result_tx) => {
                result_tx.send(self.handle_get_devices().await);
            }
            AccountCommand::GetActiveDevices(result_tx) => {
                result_tx.send(self.handle_get_active_devices().await);
            }
            AccountCommand::RequestZkNym(_) => {
                self.handle_request_zk_nym(command).await;
            }
            AccountCommand::GetDeviceZkNym => {
                self.handle_get_device_zk_nym()
                    .await
                    .inspect_err(|err| tracing::error!("Failed to get device zk-nym: {err:#?}"))
                    .ok();
            }
            AccountCommand::GetZkNymsAvailableForDownload => {
                self.handle_get_zk_nyms_available_for_download()
                    .await
                    .inspect_err(|err| {
                        tracing::error!("Failed to get zk-nyms available for download: {err:#?}")
                    })
                    .ok();
            }
            AccountCommand::GetZkNymById(id) => {
                self.handle_get_zk_nym_by_id(&id)
                    .await
                    .inspect_err(|err| tracing::error!("Failed to get zk-nym by id: {err:#?}"))
                    .ok();
            }
            AccountCommand::ConfirmZkNymIdDownloaded(id) => {
                self.handle_confirm_zk_nym_downloaded(id)
                    .await
                    .inspect_err(|err| {
                        tracing::error!("Failed to confirm zk-nym downloaded: {err:#?}")
                    })
                    .ok();
            }
            AccountCommand::GetAvailableTickets(result_tx) => {
                result_tx.send(self.handle_get_available_tickets().await);
            }
            AccountCommand::SetStaticApiAddresses(result_tx, static_api_addresses) => {
                result_tx.send(self.handle_set_static_api_addresses(static_api_addresses));
            }
            AccountCommand::RegisterOfflineMonitor(result_tx, offline_monitor) => {
                result_tx.send(self.handle_register_offline_monitor(offline_monitor).await);
            }
        };
    }

    async fn handle_command_result(&self, result: Result<AccountCommandResult, JoinError>) {
        // WIP: this can be a problem. We need to remove the commands from the running_commands for
        // this error case
        let Ok(result) = result else {
            tracing::error!("Joining task failed: {result:?}");
            return;
        };

        match result {
            AccountCommandResult::SyncAccountState(r) => {
                tracing::debug!("Account sync task: {r:?}");
                self.command_handler.finish_sync_account_state(&r).await;
                if r.is_ok() {
                    self.register_device_if_ready().await;
                    self.request_zk_nym_if_ready().await;
                }
            }
            AccountCommandResult::SyncDeviceState(r) => {
                tracing::debug!("Device sync task: {r:?}");
                self.command_handler.finish_sync_device_state(&r).await;
                if r.is_ok() {
                    self.register_device_if_ready().await;
                    self.request_zk_nym_if_ready().await;
                }
            }
            AccountCommandResult::RegisterDevice(r) => {
                tracing::debug!("Device register task: {r:?}");
                self.command_handler.finish_register_device(&r).await;
                if r.is_ok() {
                    self.get_command_sender().background_sync_account_state();
                    self.request_zk_nym_if_ready().await;
                }
            }
            AccountCommandResult::RequestZkNym(r) => {
                tracing::debug!("Request zk-nym task: {r:?}");
                self.command_handler.finish_request_zk_nym(r).await;
            }
        }
    }

    async fn cleanup(mut self) {
        let timeout = tokio::time::sleep(Duration::from_secs(5));
        tokio::pin!(timeout);
        while self.command_handler.is_command_running() {
            tokio::select! {
                _ = &mut timeout => {
                    tracing::warn!("Timeout waiting for polling tasks to finish, pending zk-nym's not imported into local credential store!");
                    break;
                },
                Some(result) = self.command_handler.join_next() => {
                    self.handle_command_result(result).await
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

        if let Err(err) = self.credential_storage.lock().await.print_info().await {
            tracing::error!("Failed to print credential storage info: {:#?}", err);
        }
    }

    pub async fn run(mut self) {
        tracing::debug!("Account controller initialized successfully");
        self.print_info().await;

        // Timer to check if any command tasks have finished. This just needs to be something small
        // so that we periodically check the results without interfering with other tasks
        let mut command_finish_timer = tokio::time::interval(Duration::from_millis(500));

        // Timer to periodically sync the remote account state.
        // Call tick() once to start the timer immediately. We don't want the first sync to happen
        // immediately, so we wait for the first tick to happen.
        let mut sync_account_state_timer = tokio::time::interval(ACCOUNT_UPDATE_INTERVAL);
        sync_account_state_timer.tick().await;

        // Timer to periodically check if we need to request more zk-nyms
        let mut update_zk_nym_timer = tokio::time::interval(ZK_NYM_AUTOMATIC_REQUEST_INTERVAL);

        loop {
            tokio::select! {
                // Handle incoming commands
                Some(command) = self.command_channel.1.recv() => {
                    self.handle_command(command).await;
                }
                // Check the results of finished tasks
                _ = command_finish_timer.tick() => {
                    while let Some(result) = self.command_handler.try_join_next() {
                        self.handle_command_result(result).await;
                    }
                }
                // On a timer we want to sync the account and device state
                _ = sync_account_state_timer.tick() => {
                    if self.offline_watch.is_online() {
                        tracing::info!("Timed sync of account and device state");
                        self.get_command_sender().background_sync_account_state();
                        self.get_command_sender().background_sync_device_state();
                    }
                }
                // On a timer to check if we need to request more zk-nyms
                _ = update_zk_nym_timer.tick() => {
                    self.request_zk_nym_if_ready().await;
                }
                _ = self.cancel_token.cancelled() => {
                    tracing::trace!("Received cancellation signal");
                    break;
                }
                Some(connectivity) = self.offline_watch.next() => {
                    self.offline_watch.handle_changed_connectivity(connectivity).await;
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

mod init {
    use std::sync::Arc;

    use nym_vpn_store::VpnStorage;

    use crate::{shared_state::MnemonicState, Error, SharedAccountState};

    use super::{
        AccountControllerConfig, AccountStorage, SharedVpnCredentialStorage, VpnCredentialStorage,
    };

    pub(super) async fn create_storage<S>(
        config: &AccountControllerConfig,
        storage: Arc<tokio::sync::Mutex<S>>,
    ) -> Result<(AccountStorage<S>, SharedVpnCredentialStorage), Error>
    where
        S: VpnStorage,
    {
        // Setup the account storage, which is used to store the account and device keys
        let account_storage = AccountStorage::from(storage);

        // Generate the device keys if we don't already have them
        account_storage.init_keys().await?;

        // Setup the credential storage, which is used to store the ticketbooks
        let credential_storage = Arc::new(tokio::sync::Mutex::new(
            VpnCredentialStorage::setup_from_path(config.data_dir.clone()).await?,
        ));

        Ok((account_storage, credential_storage))
    }

    pub(super) async fn create_initial_shared_state<S>(
        account_storage: &AccountStorage<S>,
    ) -> SharedAccountState
    where
        S: VpnStorage,
    {
        // Load the account id if we have one stored
        let mnemonic_state = account_storage
            .load_account_id()
            .await
            .map(|id| MnemonicState::Stored { id })
            .unwrap_or(MnemonicState::NotStored);

        SharedAccountState::new(mnemonic_state)
    }
}
