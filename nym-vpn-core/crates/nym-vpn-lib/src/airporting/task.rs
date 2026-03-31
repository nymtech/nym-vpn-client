// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{AirportingError, Result};
use crate::airporting::files::{init_and_load_ip_networks, update_and_load_ip_networks};
use ipnetwork::IpNetwork;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    sync::{mpsc, oneshot},
    time::{sleep, Instant},
};
use tokio_util::sync::CancellationToken;

pub struct AirportingTask {
    data_dir: PathBuf,
    country_codes: Vec<String>,
    user_agent: String,
    shutdown_token: CancellationToken,
    rx: mpsc::UnboundedReceiver<AirportingTaskMessage>,
    tx: mpsc::UnboundedSender<AirportingTaskMessage>,
    ip_networks: Vec<IpNetwork>,
    next_update_due: Instant,
}

impl AirportingTask {
    const WAKE_UP_DEFAULT_DELAY: Duration = Duration::from_secs(60);
    const INITIAL_AIRPORTING_UPDATE_DELAY: Duration = Duration::from_secs(2 * 60);
    const AIRPORTING_UPDATE_DELAY: Duration = Duration::from_secs(60 * 60);

    pub async fn spawn(
        data_dir: &Path,
        country_codes: &[&str],
        user_agent: String,
        shutdown_token: CancellationToken,
    ) -> Result<(AdBlockerTaskHandle, tokio::task::JoinHandle<()>)> {
        let (tx, rx) = mpsc::unbounded_channel();

        let task = Self {
            data_dir: data_dir.to_path_buf(),
            country_codes: country_codes.iter().map(|s| s.to_string()).collect(),
            user_agent,
            shutdown_token,
            rx,
            tx: tx.clone(),
            next_update_due: Instant::now() + Self::AIRPORTING_UPDATE_DELAY,
        };

        let join_handle = tokio::spawn(task.run());

        Ok((AdBlockerTaskHandle::new(tx), join_handle))
    }

    /// Runs the airporting manager as an actor.
    async fn run(mut self) {
        tracing::debug!("Airporting task started");

        let update_fuse = sleep(Self::WAKE_UP_DEFAULT_DELAY);
        tokio::pin!(update_fuse);

        loop {
            tokio::select! {
                msg = self.rx.recv() => {
                    match msg {
                        Some(AirportingTaskMessage::Init { response_tx }) => {
                            self.init(false, 0).await;
                            let _ = response_tx.send(());
                        }
                        #[cfg(test)]
                        Some(AirportingTaskMessage::IsInitted { response_tx }) => {
                            let _ = response_tx.send(self.is_initted().await);
                        }
                        Some(AirportingTaskMessage::InitComplete { result, retry_count }) => {
                            self.handle_init_completed(result, retry_count).await;
                        }
                        Some(AirportingTaskMessage::UpdateComplete { result }) => {
                            self.handle_update_completed(result).await;
                        }
                        Some(AirportingTaskMessage::GetIpNetworks { response_tx }) => {
                            self.handle_get_ip_networks(result).await;
                        }
                        None => {
                            break;
                        }
                    }
                }

                _ = &mut update_fuse => {
                    let now = Instant::now();
                    if self.next_update_due <= now {
                        self.update().await;
                    }
                    update_fuse
                        .as_mut()
                        .reset(Instant::now() + Self::WAKE_UP_DEFAULT_DELAY);
                }

                _ = self.shutdown_token.cancelled() => {
                    break;
                }
            }

            tracing::trace!(
                // Delightful 😒
                "Next Airporting update due at {:?}",
                time::OffsetDateTime::now_utc()
                    + time::Duration::try_from(
                        self.next_update_due
                            .saturating_duration_since(Instant::now())
                    )
                    .unwrap_or(time::Duration::ZERO)
            );
        }

        tracing::debug!("Airporting task stopped");
    }

    async fn init(&self, force_init: bool, retry_count: usize) {
        tracing::debug!("Ad-blocker initializing");

        let data_dir = self.data_dir.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let result = init_and_load_ip_networks(data_dir.clone(), force_init).await;
            let _ = tx.send(AirportingTaskMessage::InitComplete {
                result,
                retry_count,
            });
        });
    }

    async fn handle_disable(&mut self) {
        tracing::debug!("Ad-blocker disabled");

        self.clear_filter_set().await;
    }

    async fn update(&self) {
        if self.is_initted().await {
            tracing::debug!("Ad-blocker updating");

            let data_dir = self.data_dir.clone();
            let user_agent = self.user_agent.clone();
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let result = update_and_load_filter_set(data_dir, user_agent).await;
                let _ = tx.send(AirportingTaskMessage::UpdateComplete { result });
            });
        }
    }

    async fn handle_init_completed(&mut self, result: Result<Box<FilterSet>>, retry_count: usize) {
        match result {
            Ok(filter_set) => {
                self.use_filter_set(filter_set).await;
                tracing::debug!("Ad-blocker was initialized successfully");
                self.next_update_due = Instant::now() + Self::INITIAL_AIRPORTING_UPDATE_DELAY;
            }
            Err(error) => {
                tracing::error!("Failed to initialize or update ad-blocker: {error}");
                if retry_count == 0 {
                    tracing::debug!(
                        "Retrying ad-blocker initialization, forcing data file initialization"
                    );
                    self.init(true, retry_count + 1).await;
                } else {
                    tracing::error!(
                        "Ad-blocker initialization has failed twice, so will remain disabled!"
                    );
                }
            }
        }
    }

    async fn handle_update_completed(
        &mut self,
        result: Result<Option<Box<FilterSet>>, AdBlockerError>,
    ) {
        match result {
            Ok(Some(filter_set)) => {
                self.use_filter_set(filter_set).await;
                tracing::debug!("Ad-blocker was updated successfully");
            }
            Ok(None) => {
                tracing::debug!("Ad-blocker is already up-to-date");
            }
            Err(error) => {
                tracing::error!("Ad-blocker update failed: {error}");
            }
        }

        self.next_update_due = Instant::now() + Self::AIRPORTING_UPDATE_DELAY;
    }

    async fn is_initted(&self) -> bool {
        let mut guard = self.adblocker.lock().await;
        let adblocker = guard
            .as_any_mut()
            .downcast_mut::<AdBlocker>()
            .expect("Failed to downcast to AdBlocker");
        adblocker.is_initted()
    }

    async fn use_filter_set(&self, filter_set: Box<FilterSet>) {
        let mut guard = self.adblocker.lock().await;
        let adblocker = guard
            .as_any_mut()
            .downcast_mut::<AdBlocker>()
            .expect("Failed to downcast to AdBlocker");
        adblocker.use_filter_set(filter_set).await;
        crate::resolver::flush_system_cache();
    }

    async fn clear_filter_set(&self) {
        let mut guard = self.adblocker.lock().await;
        let adblocker = guard
            .as_any_mut()
            .downcast_mut::<AdBlocker>()
            .expect("Failed to downcast to AdBlocker");
        adblocker.clear_filter_set().await;
        crate::resolver::flush_system_cache();
    }
}

enum AirportingTaskMessage {
    /// Initialize Airporting.
    Init { response_tx: oneshot::Sender<()> },

    /// Has the Airporting been initialized yet?
    #[cfg(test)]
    IsInitted { response_tx: oneshot::Sender<bool> },

    /// Get the list of airporting IP addresses
    GetIpNetworks {
        response_tx: oneshot::Sender<Vec<IpNetwork>>,
    },

    /// Airporting initialized in the background.
    InitComplete {
        result: Result<Box<FilterSet>>,
        retry_count: usize,
    },

    /// Ad-blocker updated in the background.
    UpdateComplete {
        result: Result<Option<Box<FilterSet>>>,
    },
}

/// A handle to control the Ad-blocker task.
#[derive(Clone)]
pub struct AdBlockerTaskHandle {
    tx: mpsc::UnboundedSender<AirportingTaskMessage>,
}

impl AdBlockerTaskHandle {
    fn new(tx: mpsc::UnboundedSender<AirportingTaskMessage>) -> Self {
        Self { tx }
    }

    /// Enable Ad-blocker.
    pub async fn enable(&self) {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(AirportingTaskMessage::Init { response_tx })
            .is_ok()
        {
            response_rx.await.ok();
        }
    }

    /// Is the Ad-blocker initted yet? (Only used in testing).
    #[cfg(test)]
    pub async fn is_initted(&self) -> bool {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(AirportingTaskMessage::IsInitted { response_tx })
            .is_ok()
        {
            response_rx.await.ok().unwrap_or(false)
        } else {
            false
        }
    }

    /// Disable Ad-blocker.
    pub async fn disable(&self) {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(AirportingTaskMessage::Disable { response_tx })
            .is_ok()
        {
            response_rx.await.ok();
        }
    }

    /// Get the DNS Filter
    pub async fn get_dns_filter(&self) -> Option<DnsFilter> {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(AirportingTaskMessage::GetIpNetworks { response_tx })
            .is_ok()
        {
            response_rx.await.ok()
        } else {
            None
        }
    }
}
