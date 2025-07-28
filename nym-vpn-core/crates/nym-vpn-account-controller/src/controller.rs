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
    shared_state: SharedAccountState,

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
            "Initializing account controller: data_dir: {}",
            config.data_dir.display(),
        );

        // Client to query the VPN API
        let vpn_api_client = AccountControllerVpnApiClient::new(&config)?;

        // Channels for the account storage
        let (storage_op_sender, storage_op_receiver) = tokio::sync::mpsc::unbounded_channel();

        // Channels to communicate with the account controller
        let event_channel = mpsc::unbounded_channel();
        let command_channel = mpsc::unbounded_channel();

        // Setup the account storage, which is used to store the account and device keys
        let account_storage = AccountStorage::from(storage);

        // Setup the credential storage, which is used to store the ticketbooks
        let credential_storage =
            VpnCredentialStorage::setup_from_path(config.data_dir.clone()).await?;

        // SW maybe handle errors here? What kind of errors are we talking about?
        let vpn_api_account = account_storage.load_account().await.ok();
        let device_keys = Some(account_storage.load_device_keys().await?);

        // Shared_state
        let shared_state = SharedAccountState::new(
            connectivity_handle,
            config,
            credential_storage,
            vpn_api_client,
            vpn_api_account,
            device_keys,
            storage_op_sender,
        );

        let (current_state_handler, initial_state) = if shared_state
            .connectivity_handle
            .connectivity()
            .await
            .is_offline()
        {
            OfflineState::enter()
        } else {
            SyncingState::enter(&shared_state, 0)
        };

        let public_initial_state = AccountControllerState::from(initial_state);
        tracing::info!("Initial account controller state: {}", public_initial_state);
        let state_channel = watch::channel(public_initial_state);

        Ok(AccountController {
            account_storage,
            shared_state,
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

    // SW Figure out a way to print tickets without breaking everything?
    fn print_info(&self) {
        let account_id = self
            .shared_state
            .vpn_api_account
            .as_ref()
            .map(|account| account.id())
            .unwrap_or_else(|| "(unset)");

        let device_id = self
            .shared_state
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

        loop {
            let next_state = self
                .current_state_handler
                .handle_event(
                    &self.cancel_token,
                    &mut self.command_channel.1,
                    &mut self.shared_state,
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
        // SW can we do that better
        self.shared_state.credential_storage.close().await;
        tracing::debug!("Account controller state machine is exiting...");
    }
}
