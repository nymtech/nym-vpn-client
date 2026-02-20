// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{AdBlocker, AdBlockerError, Result};
use crate::{
    adblocker::files::{init_and_load_filter_set, update_and_load_filter_set},
    resolver::DnsFilter,
};
use adblock::FilterSet;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{Mutex, mpsc, oneshot},
    time::{Instant, sleep},
};
use tokio_util::sync::CancellationToken;

pub struct AdBlockerTask {
    data_dir: PathBuf,
    rx: mpsc::UnboundedReceiver<AdBlockerTaskMessage>,
    tx: mpsc::UnboundedSender<AdBlockerTaskMessage>,
    adblocker: DnsFilter,
    next_update_due: Instant,
    user_agent: String,
    shutdown_token: CancellationToken,
}

impl AdBlockerTask {
    const WAKE_UP_DEFAULT_DELAY: Duration = Duration::from_secs(60);
    const INITIAL_ADBLOCK_UPDATE_DELAY: Duration = Duration::from_secs(2 * 60);
    const ADBLOCK_UPDATE_DELAY: Duration = Duration::from_secs(60 * 60);

    pub async fn spawn(
        data_dir: &Path,
        user_agent: String,
        shutdown_token: CancellationToken,
    ) -> Result<(AdBlockerTaskHandle, tokio::task::JoinHandle<()>)> {
        let (tx, rx) = mpsc::unbounded_channel();
        let adblocker: DnsFilter = Arc::new(Mutex::new(Box::new(AdBlocker::default())));

        let task = Self {
            data_dir: data_dir.to_path_buf(),
            rx,
            tx: tx.clone(),
            adblocker,
            next_update_due: Instant::now(),
            user_agent,
            shutdown_token,
        };

        // Spawn onto the multi-thread runtime.
        let join_handle = tokio::spawn(task.run());

        Ok((AdBlockerTaskHandle::new(tx), join_handle))
    }

    /// Runs the ad-blocker manager as an actor.
    async fn run(mut self) {
        tracing::debug!("Ad-blocker task started");

        let update_fuse = sleep(Self::WAKE_UP_DEFAULT_DELAY);
        tokio::pin!(update_fuse);

        loop {
            tokio::select! {
                msg = self.rx.recv() => {
                    match msg {
                        Some(AdBlockerTaskMessage::Init { response_tx }) => {
                            self.init(false, 0).await;
                            let _ = response_tx.send(());
                        }
                        #[cfg(test)]
                        Some(AdBlockerTaskMessage::IsInitted { response_tx }) => {
                            let _ = response_tx.send(self.is_initted().await);
                        }
                        Some(AdBlockerTaskMessage::Disable { response_tx }) => {
                            self.handle_disable().await;
                            let _ = response_tx.send(());
                        }
                        Some(AdBlockerTaskMessage::InitComplete { result, retry_count }) => {
                            self.handle_init_completed(result, retry_count).await;
                        }
                        Some(AdBlockerTaskMessage::UpdateComplete { result }) => {
                            self.handle_update_completed(result).await;
                        }
                        Some(AdBlockerTaskMessage::GetDnsFilter { response_tx }) => {
                            self.handle_get_dns_filter(response_tx).await;
                        }
                        None => {
                            break;
                        }
                    }
                }

                _ = &mut update_fuse => {
                    let now = Instant::now();
                    if self.next_update_due <= now {
                        self.next_update_due = now + Self::ADBLOCK_UPDATE_DELAY;
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
        }

        tracing::debug!("Ad-blocker task stopped");
    }

    async fn init(&self, force_init: bool, retry_count: usize) {
        tracing::debug!("Ad-blocker initializing");

        let data_dir = self.data_dir.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let result = init_and_load_filter_set(data_dir.clone(), force_init).await;
            let _ = tx.send(AdBlockerTaskMessage::InitComplete {
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
                let _ = tx.send(AdBlockerTaskMessage::UpdateComplete { result });
            });
        }
    }

    async fn handle_get_dns_filter(&self, response_tx: oneshot::Sender<DnsFilter>) {
        let dns_filter: DnsFilter = self.adblocker.clone() as _;
        let _ = response_tx.send(dns_filter);
    }

    async fn handle_init_completed(&mut self, result: Result<Box<FilterSet>>, retry_count: usize) {
        match result {
            Ok(filter_set) => {
                self.use_filter_set(filter_set).await;
                self.next_update_due = Instant::now() + Self::INITIAL_ADBLOCK_UPDATE_DELAY;
                tracing::debug!("Ad-blocker was initialized successfully");
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
    }

    async fn is_initted(&self) -> bool {
        let mut guard = self.adblocker.lock().await;
        if let Some(adblocker) = guard.as_any_mut().downcast_mut::<AdBlocker>() {
            adblocker.is_initted()
        } else {
            tracing::error!("AdBlocker downcast failed!"); // Should never happen
            false
        }
    }

    async fn use_filter_set(&self, filter_set: Box<FilterSet>) {
        let mut guard = self.adblocker.lock().await;
        if let Some(adblocker) = guard.as_any_mut().downcast_mut::<AdBlocker>() {
            adblocker.use_filter_set(filter_set).await;
        } else {
            tracing::error!("AdBlocker downcast failed!"); // Should never happen
        }
    }

    async fn clear_filter_set(&self) {
        let mut guard = self.adblocker.lock().await;
        if let Some(adblocker) = guard.as_any_mut().downcast_mut::<AdBlocker>() {
            adblocker.clear_filter_set().await;
        } else {
            tracing::error!("AdBlocker downcast failed!"); // Should never happen
        }
    }
}

enum AdBlockerTaskMessage {
    /// Initialize Ad-blocker.
    Init { response_tx: oneshot::Sender<()> },

    /// Has the Ad-blocker been initialized yet?
    #[cfg(test)]
    IsInitted { response_tx: oneshot::Sender<bool> },

    /// Disable the ad-blocker, by removing the filter-set, allowing all domains to pass.
    Disable { response_tx: oneshot::Sender<()> },

    /// Get the DNS filter
    GetDnsFilter {
        response_tx: oneshot::Sender<DnsFilter>,
    },

    /// Ad-blocker initialized in the background.
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
    tx: mpsc::UnboundedSender<AdBlockerTaskMessage>,
}

impl AdBlockerTaskHandle {
    fn new(tx: mpsc::UnboundedSender<AdBlockerTaskMessage>) -> Self {
        Self { tx }
    }

    /// Enable Ad-blocker.
    pub async fn enable(&self) {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(AdBlockerTaskMessage::Init { response_tx })
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
            .send(AdBlockerTaskMessage::IsInitted { response_tx })
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
            .send(AdBlockerTaskMessage::Disable { response_tx })
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
            .send(AdBlockerTaskMessage::GetDnsFilter { response_tx })
            .is_ok()
        {
            response_rx.await.ok()
        } else {
            None
        }
    }
}
