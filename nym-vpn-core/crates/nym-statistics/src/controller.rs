// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_statistics_common::generate_vpn_client_stats_id;
use nym_vpn_lib_types::{NetworkStatisticsConfig, NetworkStatisticsIdentity};
use std::{path::Path, time::Duration};

use crate::{
    commands::{
        ConfigCommand, ControllerCommand, ControllerCommandsReceiver, SeedCommand,
        StatisticsCommandsSender,
    },
    error::Error,
    events::{StatisticsEvent, StatisticsReceiver, StatisticsSender},
    handler::StatisticsHandler,
    storage::StatsStorage,
};
use nym_statistics_api_client::StatisticsApiClient;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

pub struct StatisticsController {
    /// Config for stats reporting
    config: NetworkStatisticsConfig,

    /// Incoming stats events from other tasks
    stats_rx: StatisticsReceiver,
    stats_tx: UnboundedSender<StatisticsEvent>,

    /// Incoming commands from other tasks
    commands_rx: ControllerCommandsReceiver,
    commands_tx: UnboundedSender<ControllerCommand>,

    /// Keep store the different types of metrics collectors
    handler: Option<StatisticsHandler>,

    /// Sqlite storage
    storage: Option<StatsStorage>,

    // Listen for cancellation signals
    cancel_token: CancellationToken,
}

impl StatisticsController {
    pub async fn new<P: AsRef<Path>>(
        config: NetworkStatisticsConfig,
        stats_api_client: Option<StatisticsApiClient>,
        base_storage_path: P,
        cancel_token: CancellationToken,
    ) -> Self {
        let (stats_tx, stats_rx) = tokio::sync::mpsc::unbounded_channel();
        let (commands_tx, commands_rx) = tokio::sync::mpsc::unbounded_channel();

        let stats_storage = StatsStorage::init(base_storage_path).await.inspect_err(|e| tracing::error!("Failed to initialize stats storage. Statistics collection will be disabled : {e}")).ok();
        if stats_api_client.is_none() {
            tracing::debug!(
                "StatisticsController : No stats API client availabe, statisticis collection will be disabled"
            );
        }

        // iOS stats are disabled because they currently don't go through the tunnel
        #[cfg(target_os = "ios")]
        let mut config = config; // we need the config outside of the scope

        #[cfg(target_os = "ios")]
        {
            config.enabled = false;
        }

        let statistics_handler = if let Some(storage) = stats_storage.clone()
            && let Some(api_client) = stats_api_client
        {
            Some(StatisticsHandler::new(storage, api_client, config))
        } else {
            None
        };

        StatisticsController {
            config,
            stats_rx,
            stats_tx,
            commands_rx,
            commands_tx,
            handler: statistics_handler,
            storage: stats_storage,
            cancel_token,
        }
    }

    /// Get the stats channel used to send stats event to the controller.
    pub fn get_statistics_sender(&self) -> StatisticsSender {
        StatisticsSender::new(self.stats_tx.clone(), self.cancel_token.child_token())
    }

    /// Get the command channel used to send commands to the controller.
    pub fn get_commands_sender(&self) -> StatisticsCommandsSender {
        StatisticsCommandsSender::new(self.commands_tx.clone(), self.cancel_token.child_token())
    }

    async fn handle_command(&mut self, command: ControllerCommand) {
        match command {
            ControllerCommand::Config(ConfigCommand::DisableCollection) => {
                self.config.enabled = false;
                tracing::info!("StatisticsController : Collection disabled");
            }
            ControllerCommand::Config(ConfigCommand::EnableCollection) => {
                // Impossible to enable stats on ios while the tunnel issue isn't fixed
                #[cfg(not(target_os = "ios"))]
                {
                    self.config.enabled = true;
                    tracing::info!("StatisticsController : Collection enabled");
                }
            }
            ControllerCommand::Config(ConfigCommand::AllowDirectSending(status)) => {
                self.config.allow_disconnected = status;
            }
            ControllerCommand::Seed(SeedCommand::ResetSeed(tx, seed)) => {
                let res = match &self.storage {
                    Some(storage) => storage.reset_seed(seed).await.map_err(Into::into),
                    None => Err(Error::NoStorage),
                };
                let _ = tx.send(res);
            }
            ControllerCommand::Seed(SeedCommand::GetSeed(tx)) => {
                let res = match &self.storage {
                    Some(storage) => storage
                        .maybe_init_and_load_seed(None)
                        .await
                        .map_err(Into::into),
                    None => Err(Error::NoStorage),
                };
                let identity = res.map(|seed| NetworkStatisticsIdentity {
                    id: generate_vpn_client_stats_id(seed.clone()),
                    seed,
                });
                let _ = tx.send(identity);
            }
        }
    }

    // Main loop. We're listening to statistics events and controller commands
    pub async fn run(mut self) {
        if self.handler.is_none() {
            tracing::error!(
                "StatisticsController : either storage or API client was missing, collection disabled"
            );
        } else {
            tracing::debug!(
                "StatisticsController initialized successfully : Reporting enabled? {}",
                self.config.enabled
            );
        }

        loop {
            tokio::select! {
                biased;
                _ = self.cancel_token.cancelled() => {
                    tracing::debug!("StatisticsController : Received cancellation signal");
                    // Signal shutdown with some time to wrap up tasks
                    if let Some(ref mut stats_handler) = self.handler {
                        let _ = tokio::time::timeout(Duration::from_secs(2), stats_handler.on_shutdown()).await;
                    }
                    break;
                },
                command = self.commands_rx.recv() => match command {
                    Some(command) => {
                        match command {
                            ControllerCommand::Config(config_command) => {
                                if let Some(ref mut stats_handler) = self.handler {
                                    stats_handler.handle_command(config_command).await;
                                }
                                self.handle_command(command).await;
                            }
                            ControllerCommand::Seed(_) => {
                                self.handle_command(command).await;

                            }
                        }
                    },
                    None => {
                        tracing::error!("StatisticsController: shutting down due to closed command channel. This should never happen as we are holding a sender.");
                        break;
                    }
                },
                stats_event = self.stats_rx.recv() => match stats_event {
                    Some(stats_event) => {
                        if let Some(ref mut stats_handler) = self.handler && self.config.enabled {
                            tracing::trace!("Received stats event : {stats_event:?}");
                            stats_handler.handle_event(stats_event).await;
                        }
                    },
                    None => {
                        tracing::error!("StatisticsController: shutting down due to closed stats channel. This should never happen as we are holding a sender.");
                        break;
                    }
                },
            }
        }
        if let Some(ref stats_handler) = self.handler {
            stats_handler.close().await;
        }
        tracing::debug!("StatisticsController: Exiting");
    }
}
