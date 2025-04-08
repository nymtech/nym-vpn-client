// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use nym_offline_monitor::{Connectivity, MonitorHandle};

use crate::{AccountCommand, AccountControllerCommander};

#[derive(Debug, thiserror::Error)]
pub enum OfflineMonitorError {
    #[error("offline watch already registered")]
    WatchAlreadyRegistered,
}

pub(super) struct OfflineWatch {
    // The current connectivity state.
    connectivity: Arc<std::sync::Mutex<Connectivity>>,

    // The account controller commander, used for sending commands to the account controller that
    // are triggered by connectivity changes.
    commander: AccountControllerCommander,

    // The task that monitors the connectivity state and updates the `connectivity` field.
    task: Option<tokio::task::JoinHandle<()>>,
}

impl OfflineWatch {
    pub(super) fn new(commander: AccountControllerCommander, initial_state: Connectivity) -> Self {
        let connectivity = Arc::new(std::sync::Mutex::new(initial_state));
        let task = None;

        Self {
            connectivity,
            commander,
            task,
        }
    }

    pub(super) fn is_online(&self) -> bool {
        self.connectivity
            .lock()
            .map(|c| c.is_online())
            .unwrap_or(false)
    }

    pub(super) fn is_offline(&self) -> bool {
        !self.is_online()
    }

    pub(super) fn register_offline_watch(
        &mut self,
        offline_watch: MonitorHandle,
    ) -> Result<(), OfflineMonitorError> {
        if self.task.is_some() {
            return Err(OfflineMonitorError::WatchAlreadyRegistered);
        }

        let offline_watch_task = OfflineWatchTask::new(
            self.connectivity.clone(),
            offline_watch,
            self.commander.clone(),
        );
        let handle = tokio::spawn(offline_watch_task.run());

        self.task = Some(handle);

        Ok(())
    }

    pub(super) async fn wait(&mut self, timeout: Duration) {
        if let Some(mut task) = self.task.take() {
            tokio::select! {
                _ = &mut task => (),
                _ = tokio::time::sleep(timeout) => {
                    tracing::error!("Offline watch task did not finish within the specified timeout");
                },
            }
        } else {
            tracing::error!("Tried to wait for offline watch task, but it was not registered");
        }
    }
}

struct OfflineWatchTask {
    connectivity: Arc<std::sync::Mutex<Connectivity>>,
    offline_watch: MonitorHandle,
    commander: AccountControllerCommander,
}

impl OfflineWatchTask {
    fn new(
        connectivity: Arc<std::sync::Mutex<Connectivity>>,
        offline_watch: MonitorHandle,
        commander: AccountControllerCommander,
    ) -> Self {
        Self {
            connectivity,
            offline_watch,
            commander,
        }
    }

    fn signal_went_online_to_controller(&self) {
        self.commander
            .send(AccountCommand::SyncAccountState(None))
            .inspect_err(|e| tracing::error!("{e}"))
            .ok();
        self.commander
            .send(AccountCommand::SyncDeviceState(None))
            .inspect_err(|e| tracing::error!("{e}"))
            .ok();
    }

    fn update_state(&self, new_state: Connectivity) {
        if let Ok(mut connectivity) = self
            .connectivity
            .lock()
            .inspect_err(|e| tracing::error!("failed to acquire lock: {e:?}"))
        {
            let old_state = *connectivity;
            *connectivity = new_state;
            tracing::info!("Connectivity state changed from {old_state:?} to {new_state:?}",);

            if new_state.is_online() && old_state.is_offline() {
                self.signal_went_online_to_controller();
            }
        }
    }

    async fn run(mut self) {
        tracing::info!("Starting offline watch task");
        let initial_state = self.offline_watch.connectivity().await;
        self.update_state(initial_state);

        while let Some(state) = self.offline_watch.next().await {
            self.update_state(state);
        }
        tracing::info!("Offline watch task has finished");
    }
}
