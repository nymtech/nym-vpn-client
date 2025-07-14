// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::{FutureExt, future::Fuse, pin_mut};
use std::{path::Path, time::Duration};

use crate::{
    api_client::StatisticsControllerApiClient,
    config::StatisticsControllerConfig,
    events::{StatisticsEvent, StatisticsReceiver, StatisticsSender, UsageEvent},
    handler::StatisticsHandler,
    storage::StatsStorage,
};
use rand::{distributions::Uniform, prelude::Distribution};
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

pub struct StatisticsController {
    /// Config for stats reporting (enabled, address, interval)
    config: StatisticsControllerConfig,

    /// Keep store the different types of metrics collectors
    handler: Option<StatisticsHandler>,

    /// Api client to send statistics
    stats_api_client: Option<StatisticsControllerApiClient>,

    /// Incoming packet stats events from other tasks
    stats_rx: StatisticsReceiver,

    stats_tx: UnboundedSender<StatisticsEvent>,

    // Listen for cancellation signals
    cancel_token: CancellationToken,
}

impl StatisticsController {
    pub async fn new<P: AsRef<Path>>(
        config: StatisticsControllerConfig,
        base_storage_path: P,
        cancel_token: CancellationToken,
    ) -> Self {
        let (stats_tx, stats_rx) = tokio::sync::mpsc::unbounded_channel();

        let stats_storage = StatsStorage::init(base_storage_path).await.inspect_err(|e| tracing::error!("Failed to initialize stats storage. Statistics collection will be disabled : {e}")).ok();
        let stats_api_client = StatisticsControllerApiClient::new(&config).inspect_err(|e| tracing::error!(" Failed to build Statistics API client. Statistics collection will be disabled : {e}")).ok().flatten();

        StatisticsController {
            handler: stats_storage.map(|s| StatisticsHandler::new(s, config.clone())),
            stats_api_client,
            stats_rx,
            stats_tx,
            config,
            cancel_token,
        }
    }

    /// Get the command channel used to send commands to the controller.
    pub fn get_statistics_sender(&self) -> StatisticsSender {
        StatisticsSender::new(Some(self.stats_tx.clone()), self.cancel_token.child_token())
    }

    async fn cleanup(&self) {
        if let Some(handler) = &self.handler {
            handler.close().await
        }
    }

    pub async fn run(self) {
        tracing::debug!("StatisticsController initialized successfully");
        if self.config.enabled && self.stats_api_client.is_some() && self.handler.is_some() {
            tracing::debug!("Statistics reporting is enabled");
            self.enabled_loop().await
        } else {
            tracing::debug!("Statistics reporting is disabled");
            self.disabled_loop().await
        }
    }

    // we can't just not run, because StatisticsSender everywhere will expect to be able to send stuff. Hence we will just consume the events and do nothing
    async fn disabled_loop(mut self) {
        loop {
            tokio::select! {
                biased;
                _ = self.cancel_token.cancelled() => {
                    tracing::trace!("StatisticsController : Received cancellation signal");
                    break;
                },
                stats_event = self.stats_rx.recv() => match stats_event {
                        Some(_) => {},
                        None => {
                            tracing::trace!("StatisticsController: shutting down due to closed stats channel");
                            break;
                        }
                },
            }
        }
        self.cleanup().await
    }
    async fn enabled_loop(mut self) {
        if !self.config.enabled || self.stats_api_client.is_none() || self.handler.is_none() {
            tracing::error!(
                "StatisticsController : Enabled loop with disabled collection, missing api client or missing handler. This should never happen."
            );
            self.cleanup().await;
            return;
        }

        // Safety : We just checked that self.stats_api_client wasn't None
        #[allow(clippy::unwrap_used)]
        let stats_api_client = self.stats_api_client.unwrap();

        // Safety : We just checked that self.handler wasn't None
        #[allow(clippy::unwrap_used)]
        let mut stats_handler = self.handler.unwrap();

        let send_timer = Fuse::terminated();
        pin_mut!(send_timer);

        loop {
            tokio::select! {
                biased;
                _ = self.cancel_token.cancelled() => {
                    tracing::trace!("StatisticsController : Received cancellation signal");
                    break;
                },
                stats_event = self.stats_rx.recv() => match stats_event {
                    Some(stats_event) => {
                        tracing::trace!("Received stats event : {stats_event:?}");
                        if matches!(stats_event, StatisticsEvent::Usage(UsageEvent::Connected(_))) {
                            let random_delay_secs = Uniform::new_inclusive(0, self.config.max_reporting_delay).sample(&mut rand::thread_rng());
                            tracing::debug!("StatisticsController : Trying to send report in {random_delay_secs} secs");
                            send_timer.set(tokio::time::sleep(Duration::from_secs(random_delay_secs)).fuse());
                        }
                        stats_handler.handle_event(stats_event).await
                    },
                    None => {
                        tracing::trace!("StatisticsController: shutting down due to closed stats channel");
                        break;
                    }
                },

                // Initial sending strategy, send after a random amount of time, if we're still connected
                _ = &mut send_timer => {
                    if stats_handler.is_connected() {
                        tracing::debug!("Send timer fired and connected, sending report");
                        match stats_handler.get_report().await {
                            Ok(report) => {
                                if let Err(e) = stats_api_client.post_report(report).await {
                                    tracing::warn!("Failed to send statistics report : {e}");
                                } else {
                                    tracing::debug!("Stats sent successfull");
                                }

                            },
                            Err(e) => tracing::warn!("Failed to generate statistics report : {e}"),
                        }
                    } else {
                        tracing::debug!("Not connected, not sending anything")
                    }
                }
            }
        }
        stats_handler.close().await;
        tracing::trace!("StatisticsController: Exiting");
    }
}
