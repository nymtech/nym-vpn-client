// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_offline_monitor::ConnectivityHandle;

use nym_vpn_lib_types::{AccountControllerEvent, AccountControllerState};
use nym_vpn_store::VpnStorage;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    watch,
};
use tokio_util::sync::CancellationToken;

use crate::{
    AccountCommandSender, AccountControllerConfig, AccountStateReceiver,
    commands::AccountCommand,
    error::Error,
    shared_state::SharedAccountState,
    state_machine::{
        AccountControllerStateHandler, NextAccountControllerState, OfflineState, SyncingState,
    },
    storage::{AccountStorage, AccountStorageOp, VpnCredentialStorage},
    vpn_api_client::AccountControllerVpnApiClient,
};

pub struct AccountController<S>
where
    S: VpnStorage,
{
    // The storage used for the account and device keys
    account_storage: AccountStorage<S>,

    // The current state of the account
    account_state: SharedAccountState,

    // Receiver channel used to receive commands from the outside.
    command_channel: (
        UnboundedSender<AccountCommand>,
        UnboundedReceiver<AccountCommand>,
    ),

    // State broadcast channels
    state_channel: (
        watch::Sender<AccountControllerState>,
        watch::Receiver<AccountControllerState>,
    ),

    // Channel to transmit event to the outside world
    event_channel: (
        UnboundedSender<AccountControllerEvent>,
        UnboundedReceiver<AccountControllerEvent>,
    ),

    // Channel to received and execute storage operation
    storage_op_receiver: mpsc::UnboundedReceiver<AccountStorageOp>,

    // Current state machine state
    current_state_handler: Box<dyn AccountControllerStateHandler>,

    // Listen for cancellation signals
    cancel_token: CancellationToken,
}

impl<S> AccountController<S>
where
    S: VpnStorage + Send + Sync + 'static,
{
    pub async fn new(
        config: AccountControllerConfig,
        storage: S,
        connectivity_handle: ConnectivityHandle,
        cancel_token: CancellationToken,
    ) -> Result<Self, Error> {
        tracing::info!(
            "Starting account controller: data_dir: {}",
            config.data_dir.display(),
        );

        // Setup up the storage. We have both the account storage as well as the credential storage
        let (account_storage, credential_storage) = init::create_storage(&config, storage).await?;

        // Channels for the account storage
        let (storage_op_sender, storage_op_receiver) = tokio::sync::mpsc::unbounded_channel();

        // Channels to communicate with the account controller
        let event_channel = mpsc::unbounded_channel();
        let command_channel = mpsc::unbounded_channel();

        // Client to query the VPN API
        let vpn_api_client = AccountControllerVpnApiClient::new(&config)?;

        let account_state = init::create_initial_shared_state(
            connectivity_handle,
            config,
            &account_storage,
            credential_storage.clone(),
            vpn_api_client.clone(),
            storage_op_sender,
        )
        .await?;

        let (current_state_handler, initial_state) = if account_state
            .connectivity_handle
            .connectivity()
            .await
            .is_offline()
        {
            OfflineState::enter()
        } else {
            SyncingState::enter(&account_state)
        };

        let public_initial_state = AccountControllerState::from(initial_state);
        tracing::info!("Initial account controller state: {}", public_initial_state);
        let state_channel = watch::channel(public_initial_state);

        Ok(AccountController {
            account_storage,
            account_state,
            command_channel,
            state_channel,
            event_channel,
            storage_op_receiver,
            current_state_handler,
            cancel_token,
        })
    }

    /// Get the command channel used to send commands to the controller.
    pub fn get_command_sender(&self) -> AccountCommandSender {
        AccountCommandSender::new(self.command_channel.0.clone())
    }

    /// Get the command channel used to send commands to the controller.
    pub fn get_state_receiver(&self) -> AccountStateReceiver {
        AccountStateReceiver::new(self.state_channel.1.clone())
    }

    // async fn request_zk_nym_if_ready(&self) {
    //     if self.offline_watch.is_offline() {
    //         tracing::info!("Not requesting zk-nym as we are offline");
    //         return;
    //     }

    //     // if !self.is_background_zk_nym_refresh_active().await {
    //     //     return;
    //     // }

    //     match self.is_all_ticket_types_above_soft_threshold().await {
    //         Ok(false) => (),
    //         Ok(true) => {
    //             tracing::debug!("All ticket types are above soft threshold, not requesting zk-nym");
    //             return;
    //         }
    //         Err(err) => {
    //             // Be conservative, it might be wasteful to request zknyms if we can't store them
    //             // locally anyway.
    //             tracing::error!(
    //                 "Failed to lookup current tickets, not requesting more zk-nyms: {err}"
    //             );
    //             return;
    //         }
    //     }

    //     match self.get_shared_state().ready_to_request_zk_nym().await {
    //         ReadyToRequestZkNym::Ready => {
    //             self.get_command_sender().background_request_zk_nyms();
    //         }
    //         not_ready => {
    //             tracing::debug!("Not ready to try to request zk-nym: {not_ready}");
    //         }
    //     }
    // }

    //SW Figure out a way to make that work without breaking everything?
    fn print_info(&self) {
        let account_id = self
            .account_state
            .vpn_api_account
            .as_ref()
            .map(|account| account.id())
            .unwrap_or_else(|| "(unset)");

        let device_id = self
            .account_state
            .device
            .as_ref()
            .map(|d| d.identity_key().to_base58_string())
            .unwrap_or_else(|| "(unset)".to_string());

        tracing::info!("Account id: {}", account_id);
        tracing::info!("Device id: {}", device_id);
    }

    pub async fn run(mut self) {
        tracing::debug!("Account controller initialized successfully");
        self.print_info();

        let storage = self.account_storage;

        // Loop to handle storage event
        tokio::spawn(async move {
            while let Some(storage_op) = self.storage_op_receiver.recv().await {
                storage.handle_storage_op(storage_op).await
            }
        });

        // ADD, On a timer, refresh sync and request ZK nym
        // SW ADD THAT IN THE STATES

        loop {
            let next_state = self
                .current_state_handler
                .handle_event(
                    &self.cancel_token,
                    &mut self.command_channel.1,
                    &mut self.account_state,
                )
                .await;

            match next_state {
                NextAccountControllerState::NewState((new_state_handler, new_state)) => {
                    self.current_state_handler = new_state_handler;

                    let state = AccountControllerState::from(new_state);
                    tracing::info!("New AccountController state: {}", state);
                    let _ = self
                        .event_channel
                        .0
                        .send(AccountControllerEvent::NewState(state));
                    let _ = self.state_channel.0.send_replace(state);
                }
                NextAccountControllerState::SameState(same_state) => {
                    self.current_state_handler = same_state;
                }
                NextAccountControllerState::Finished => break,
            }
        }
        // SW do that better
        self.account_state.credential_storage.close().await;
        tracing::debug!("Account controller state machine is exiting...");
    }
}

mod init {

    use nym_offline_monitor::ConnectivityHandle;
    use nym_vpn_store::VpnStorage;
    use tokio::sync::mpsc;

    use crate::{
        Error, controller::SharedAccountState, storage::AccountStorageOp,
        vpn_api_client::AccountControllerVpnApiClient,
    };

    use super::{AccountControllerConfig, AccountStorage, VpnCredentialStorage};

    pub(super) async fn create_storage<S>(
        config: &AccountControllerConfig,
        storage: S,
    ) -> Result<(AccountStorage<S>, VpnCredentialStorage), Error>
    where
        S: VpnStorage,
    {
        // Setup the account storage, which is used to store the account and device keys
        let account_storage = AccountStorage::from(storage);

        // Generate the device keys if we don't already have them
        account_storage.init_keys().await?;

        // Setup the credential storage, which is used to store the ticketbooks
        let credential_storage =
            VpnCredentialStorage::setup_from_path(config.data_dir.clone()).await?;

        Ok((account_storage, credential_storage))
    }

    pub(super) async fn create_initial_shared_state<S>(
        connectivity_handle: ConnectivityHandle,
        config: AccountControllerConfig,
        account_storage: &AccountStorage<S>,
        credential_storage: VpnCredentialStorage,
        vpn_api_client: AccountControllerVpnApiClient,
        storage_op_sender: mpsc::UnboundedSender<AccountStorageOp>,
    ) -> Result<SharedAccountState, Error>
    where
        S: VpnStorage,
    {
        // SW maybe handle errors here? What kind of errors are we talking about?
        let vpn_api_account = account_storage.load_account().await.ok();
        let device_keys = account_storage.load_device_keys().await.ok();
        Ok(SharedAccountState::new(
            connectivity_handle,
            config,
            credential_storage,
            vpn_api_client,
            vpn_api_account,
            device_keys,
            storage_op_sender,
        )
        .await)
    }
}
