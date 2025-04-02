// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_offline_monitor::{Connectivity, ConnectivityHandle};

use crate::AccountCommandSender;

pub(super) struct OfflineWatch {
    // The handle to the offline monitor, used for receiving connectivity changes.
    handle: Option<ConnectivityHandle>,

    // The account controller commander, used for sending commands to the account controller that
    // are triggered by connectivity changes.
    command_sender: AccountCommandSender,

    // The last known connectivity state, used to determine any possible actions that should be
    // taken depending on the connectivity change.
    last_state: Connectivity,
}

impl OfflineWatch {
    pub(super) fn new(command_sender: AccountCommandSender, initial_state: Connectivity) -> Self {
        Self {
            handle: None,
            command_sender,
            last_state: initial_state,
        }
    }

    pub(super) fn is_online(&self) -> bool {
        self.last_state.is_online()
    }

    pub(super) fn is_offline(&self) -> bool {
        !self.is_online()
    }

    pub(super) async fn register_offline_monitor(&mut self, offline_monitor: ConnectivityHandle) {
        self.last_state = offline_monitor.connectivity().await;
        tracing::info!(
            "Registering offline monitor with initial state: {:?}",
            self.last_state
        );

        if self.handle.replace(offline_monitor).is_some() {
            tracing::info!("Registering offline monitor replaced an existing one");
        }
    }

    pub(super) async fn next(&mut self) -> Option<Connectivity> {
        if let Some(handle) = self.handle.as_mut() {
            handle.next().await
        } else {
            std::future::pending().await
        }
    }

    pub(super) async fn handle_changed_connectivity(&mut self, new_state: Connectivity) {
        if new_state != self.last_state {
            let old_state = self.last_state;
            self.last_state = new_state;
            tracing::info!("Connectivity state changed from {old_state:?} to {new_state:?}");

            if new_state.is_online() && old_state.is_offline() {
                self.signal_went_online_to_controller();
            }
        }
    }

    fn signal_went_online_to_controller(&self) {
        self.command_sender.background_sync_account_state();
        self.command_sender.background_sync_device_state();
    }
}
