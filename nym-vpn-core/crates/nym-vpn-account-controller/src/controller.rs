// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_bandwidth_controller::requests::BandwidthControllerRequestSender;
use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::VpnApiClient;
use nym_vpn_lib_types::{AccountControllerEvent, AccountControllerState};
use nym_vpn_store::{VpnStorage, keys::wireguard::WireguardKeysDb};
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    watch,
};
use tokio_util::sync::CancellationToken;

use crate::{
    AccountCommandSender, AccountControllerConfig, AccountControllerEventSender,
    AccountStateReceiver, NyxdClient,
    commands::AccountCommand,
    error::Error,
    event_sender::AccountControllerEventReceiver,
    shared_state::SharedAccountState,
    state_machine::{
        AccountControllerStateHandler, NextAccountControllerState, OfflineState, SyncMode,
        SyncingNetworkState,
    },
    storage::{AccountStorage, AccountStorageOp},
};

pub struct AccountController<C, S>
where
    S: VpnStorage,
    C: ConnectivityMonitor,
{
    // The storage used for the account and device keys
    account_storage: AccountStorage<S>,

    // The current state of the account
    shared_state: SharedAccountState<C>,

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

    // Channel to received and execute storage operation
    storage_op_receiver: UnboundedReceiver<AccountStorageOp>,

    // Current state machine state
    current_state_handler: Box<dyn AccountControllerStateHandler<C>>,

    // Listen for cancellation signals
    cancel_token: CancellationToken,
}

impl<C, S> AccountController<C, S>
where
    S: VpnStorage + Send + Sync + 'static,
    C: ConnectivityMonitor,
{
    pub async fn new(
        nym_vpn_api_client: VpnApiClient,
        nyxd_client: NyxdClient,
        config: AccountControllerConfig,
        storage: S,
        connectivity_handle: C,
        bandwidth_control_command_tx: BandwidthControllerRequestSender,
        cancel_token: CancellationToken,
    ) -> Result<Self, Error> {
        tracing::info!(
            "Initializing account controller: data_dir: {}",
            config.data_dir.display(),
        );

        // Channels for the account storage
        let (storage_op_sender, storage_op_receiver) = mpsc::unbounded_channel();

        // Channels to communicate with the account controller
        let event_channel = AccountControllerEventSender::new();
        let command_channel = mpsc::unbounded_channel();

        // Setup the account storage, which is used to store the account and device keys
        let account_storage = AccountStorage::from(storage);

        // Load remaining state that may fail
        let vpn_api_account = account_storage.load_vpn_account().await?;
        let device_keys = account_storage.load_device_keys().await?;
        let wireguard_keys_storage = WireguardKeysDb::init(Some(config.data_dir.clone())).await?;

        // Load the cached account summary (if any) so it is available before the first network
        // sync completes. A read failure is non-critical - treat it as no cache.
        let cached_account_summary = if vpn_api_account.is_some() {
            account_storage
                .load_account_summary()
                .await
                .unwrap_or_else(|err| {
                    tracing::warn!("Failed to load cached account summary: {err}");
                    None
                })
        } else {
            // If we don't have an account stored, make sure we don't have a summary either
            let _ = account_storage.remove_account_summary().await;
            None
        };

        // Shared_state
        let mut shared_state = SharedAccountState::new(
            connectivity_handle,
            bandwidth_control_command_tx,
            config,
            wireguard_keys_storage,
            nym_vpn_api_client,
            nyxd_client,
            vpn_api_account,
            cached_account_summary,
            device_keys,
            storage_op_sender,
            event_channel,
            cancel_token.clone(),
        );

        let is_offline = shared_state
            .connectivity_handle
            .connectivity()
            .await
            .is_offline();

        let (current_state_handler, initial_state) = if is_offline {
            OfflineState::enter()
        } else {
            SyncingNetworkState::enter(&mut shared_state, SyncMode::Optimistic).await
        };

        let public_initial_state = AccountControllerState::from(initial_state);
        tracing::info!("Initial account controller state: {}", public_initial_state);
        let state_channel = watch::channel(public_initial_state);

        Ok(AccountController {
            account_storage,
            shared_state,
            command_channel,
            state_channel,
            storage_op_receiver,
            current_state_handler,
            cancel_token,
        })
    }

    /// Get the channel for receiving broadcasted controller events.
    pub fn subscribe_to_events(&self) -> AccountControllerEventReceiver {
        self.shared_state.event_sender.subscribe()
    }

    /// Get the command channel used to send commands to the controller.
    pub fn get_command_sender(&self) -> AccountCommandSender {
        AccountCommandSender::new(self.command_channel.0.clone())
    }

    /// Get the channel used to keep track of the controller state.
    pub fn get_state_receiver(&self) -> AccountStateReceiver {
        AccountStateReceiver::new(self.state_channel.1.clone())
    }

    /// Get the wireguard keys database storage
    pub fn get_wireguard_keys_storage(&self) -> WireguardKeysDb {
        self.shared_state.wireguard_keys_storage.clone()
    }

    fn print_info(&self) {
        let account_id = self
            .shared_state
            .vpn_api_account
            .as_ref()
            .map(|account| account.id())
            .unwrap_or_else(|| "(unset)".to_string());

        let device_id = self
            .shared_state
            .device
            .as_ref()
            .map(|d| d.identity_key().to_base58_string())
            .unwrap_or_else(|| "(unset)".to_string());

        tracing::info!("Account id: {account_id}");
        tracing::info!("Device id: {device_id}");
    }

    pub async fn run(mut self) {
        tracing::debug!("Account controller initialized successfully");
        self.print_info();

        let storage = self.account_storage;

        // Loop to handle storage event. It will stop automatically when the storage_op_sender will be dropped
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
                    self.shared_state
                        .event_sender
                        .broadcast(AccountControllerEvent::NewState(state.clone()));
                    let _ = self.state_channel.0.send_replace(state);
                }
                NextAccountControllerState::SameState(same_state) => {
                    self.current_state_handler = same_state;
                }
                NextAccountControllerState::Finished => break,
            }
        }

        self.shared_state.wireguard_keys_storage.close().await;
        tracing::debug!("Account controller state machine is exiting...");
    }
}
