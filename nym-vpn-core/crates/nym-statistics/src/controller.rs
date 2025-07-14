// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::Path, pin::Pin, time::Duration};

use crate::{
    api_client::StatisticsControllerApiClient,
    config::StatisticsControllerConfig,
    error::Error,
    events::{StatisticsEvent, StatisticsReceiver, StatisticsSender},
    handler::StatisticsHandler,
    storage::StatsStorage,
};

use nym_vpn_lib_types::TunnelState;
use rand::{distributions::Uniform, prelude::Distribution};
use tokio::{
    sync::{mpsc::UnboundedSender, watch},
    time::Sleep,
};
use tokio_util::sync::CancellationToken;

pub struct StatisticsController {
    /// Config for stats reporting (enabled, address, interval)
    config: StatisticsControllerConfig,

    /// Keep store the different types of metrics collectors
    handler: StatisticsHandler,

    /// Api client to send statistics
    stats_api_client: Option<StatisticsControllerApiClient>,

    /// Incoming packet stats events from other tasks
    stats_rx: StatisticsReceiver,

    stats_tx: UnboundedSender<StatisticsEvent>,

    tunnel_state: watch::Receiver<TunnelState>,

    // Listen for cancellation signals
    cancel_token: CancellationToken,
}

impl StatisticsController {
    pub async fn new<P: AsRef<Path>>(
        config: StatisticsControllerConfig,
        base_storage_path: P,
        cancel_token: CancellationToken,
        tunnel_state: watch::Receiver<TunnelState>,
    ) -> Result<Self, Error> {
        let (stats_tx, stats_rx) = tokio::sync::mpsc::unbounded_channel();

        let stats_storage = StatsStorage::init(base_storage_path).await?;
        let stats_api_client = StatisticsControllerApiClient::new(&config)?;

        Ok(StatisticsController {
            handler: StatisticsHandler::new(stats_storage, config.clone()),
            stats_api_client,
            stats_rx,
            stats_tx,
            tunnel_state,
            config,
            cancel_token,
        })
    }

    /// Get the command channel used to send commands to the controller.
    pub fn get_statistics_sender(&self) -> StatisticsSender {
        StatisticsSender::new(Some(self.stats_tx.clone()), self.cancel_token.child_token())
    }

    pub async fn run(self) {
        tracing::debug!("StatisticsController initialized successfully");
        if self.config.enabled && self.config.stats_collector_url.is_some() {
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
                    return;
                },
                stats_event = self.stats_rx.recv() => match stats_event {
                        Some(_) => {},
                        None => {
                            tracing::trace!("StatisticsController: shutting down due to closed stats channel");
                            return;
                        }
                },
            }
        }
    }
    async fn enabled_loop(mut self) {
        if !self.config.enabled || self.stats_api_client.is_none() {
            tracing::error!(
                "StatisticsController : Enabled loop with disabled collection or missing api client. This should never happen."
            );
            return;
        }

        // Safety : We just checked that self.stats_api_client wasn't None
        #[allow(clippy::unwrap_used)]
        let stats_api_client = self.stats_api_client.unwrap();

        let mut send_timer: Option<Pin<Box<Sleep>>> = None;

        loop {
            tokio::select! {
                biased;
                _ = self.cancel_token.cancelled() => {
                    tracing::trace!("StatisticsController : Received cancellation signal");
                    break;
                },
                stats_event = self.stats_rx.recv() => match stats_event {
                    Some(stats_event) => self.handler.handle_event(stats_event).await,
                    None => {
                        tracing::trace!("StatisticsController: shutting down due to closed stats channel");
                        break;
                    }
                },
                Ok(_) = self.tunnel_state.changed() => {
                    match self.tunnel_state.borrow().clone() {
                        TunnelState::Connected {..} => {
                            if let Err(e) = self.stats_tx.send(StatisticsEvent::new_connected()){
                                tracing::warn!("Failed to send stat event : {e}")
                            }
                            let random_delay_secs = Uniform::new_inclusive(0, self.config.max_reporting_delay).sample(&mut rand::thread_rng());
                            tracing::debug!("StatisticsController : Trying to send report in {random_delay_secs} secs");
                            send_timer = Some(Box::pin(tokio::time::sleep(Duration::from_secs(random_delay_secs))));

                        },
                        TunnelState::Disconnecting {..}=>{
                            if let Err(e) = self.stats_tx.send(StatisticsEvent::new_disconnecting()){
                                tracing::warn!("Failed to send stat event : {e}")
                            }
                        },
                        TunnelState::Disconnected => {
                            if let Err(e) = self.stats_tx.send(StatisticsEvent::new_disconnected()){
                                tracing::warn!("Failed to send stat event : {e}")
                            }
                        },
                        TunnelState::Error(client_error) => {
                            if let Err(e) = self.stats_tx.send(StatisticsEvent::new_error(client_error.clone())){
                               tracing::warn!("Failed to send stat event : {e}")
                            }
                        },
                        _ => {},

                    }


                },
                // Initial sending strategy, send after a random amount of time, if we're still connected
                _ = wait_on_maybe_timer(&mut send_timer) => { //SW can't find a way to make that work differently for now
                    if matches!(*self.tunnel_state.borrow(), TunnelState::Connected { .. }) {
                        tracing::debug!("Send timer fired and connected, sending stuff");
                        match self.handler.get_report().await {
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
                        send_timer = None;
                }
            }
        }
        tracing::trace!("StatisticsController: Exiting");
    }
}

async fn wait_on_maybe_timer(timer: &mut Option<Pin<Box<Sleep>>>) {
    if let Some(t) = timer {
        t.as_mut().await
    } else {
        std::future::pending::<()>().await
    }
}
