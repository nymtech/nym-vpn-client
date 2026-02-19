// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{AdBlocker, AdBlockerError, Result};
use crate::resolver::DnsFilter;
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
    adblocker_initted: bool,
    next_update_due: Instant,
    user_agent: String,
    shutdown_token: CancellationToken,
}

impl AdBlockerTask {
    const WAKE_UP_DEFAULT_DELAY: Duration = Duration::from_secs(60);
    const INITIAL_ADBLOCK_UPDATE_DELAY: Duration = Duration::from_secs(2 * 60);
    const ADBLOCK_UPDATE_DELAY: Duration = Duration::from_secs(60 * 60);

    /// Spawn the AdBlocker task.
    ///
    /// Returns:
    /// - a handle to control the task
    /// - a `DnsFilter` used by the resolver to check domains
    /// - the join handle for the background task
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
            adblocker: adblocker.clone(),
            adblocker_initted: false,
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
        let adblock_update_fuse = sleep(Self::WAKE_UP_DEFAULT_DELAY);
        tokio::pin!(adblock_update_fuse);

        loop {
            tokio::select! {
                msg = self.rx.recv() => {
                    match msg {
                        Some(AdBlockerTaskMessage::InitAdBlocker { response_tx }) => {
                            self.init_ad_blocker(false, 0).await;
                            let _ = response_tx.send(());
                        }
                        Some(AdBlockerTaskMessage::AdBlockerInitted { result, retry_count }) => {
                            self.handle_ad_blocker_initted(result, retry_count).await;
                        }
                        Some(AdBlockerTaskMessage::AdBlockerUpdated { result }) => {
                            self.handle_ad_blocker_updated(result).await;
                        }
                        Some(AdBlockerTaskMessage::GetDnsFilter { response_tx }) => {
                            let _ = response_tx.send(self.adblocker.clone());
                        }
                        Some(AdBlockerTaskMessage::StoppedUsingDnsFilter { response_tx }) => {
                            self.handle_stopped_using_dns_filter().await;
                            let _ = response_tx.send(());
                        }
                        None => {
                            self.shutdown_token.cancel();
                            break;
                        }
                    }
                }

                _ = &mut adblock_update_fuse => {
                    let now = Instant::now();
                    if self.next_update_due <= now {
                        self.update_ad_blocker().await;
                    }
                    adblock_update_fuse
                        .as_mut()
                        .reset(Instant::now() + Self::WAKE_UP_DEFAULT_DELAY);
                }

                _ = self.shutdown_token.cancelled() => {
                    break;
                }
            }
        }
    }

    /// Initialize Ad-blocker. This is expensive, so we spawn a new task to perform
    /// initialization in the background, and update once it is done.
    async fn init_ad_blocker(&self, force_init: bool, retry_count: usize) {
        let data_dir = self.data_dir.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let result = AdBlocker::with_files(data_dir, force_init).await;
            let _ = tx.send(AdBlockerTaskMessage::AdBlockerInitted {
                result,
                retry_count,
            });
        });
    }

    /// Update filters. Potentially expensive, so perform work in the background.
    async fn update_ad_blocker(&self) -> Duration {
        if self.adblocker_initted {
            let data_dir = self.data_dir.clone();
            let user_agent = self.user_agent.clone();
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let result = AdBlocker::with_updated_files(data_dir, user_agent).await;
                let _ = tx.send(AdBlockerTaskMessage::AdBlockerUpdated { result });
            });
        }
        Self::ADBLOCK_UPDATE_DELAY
    }

    async fn handle_ad_blocker_initted(
        &mut self,
        result: Result<Box<AdBlocker>, AdBlockerError>,
        retry_count: usize,
    ) {
        match result {
            Ok(adblocker) => {
                let mut guard = self.adblocker.lock().await;
                *guard = adblocker;
                self.adblocker_initted = true;
                self.next_update_due = Instant::now() + Self::INITIAL_ADBLOCK_UPDATE_DELAY;
                tracing::debug!("Ad-blocker was initialized successfully");
            }
            Err(error) => {
                tracing::error!("Failed to initialize or update ad-blocker: {error}");
                self.adblocker_initted = false;
                self.next_update_due = Instant::now() + Self::ADBLOCK_UPDATE_DELAY;
                if retry_count == 0 {
                    tracing::debug!(
                        "Retrying ad-blocker initialization, forcing data file initialization"
                    );
                    self.init_ad_blocker(true, retry_count + 1).await;
                } else {
                    tracing::error!(
                        "Ad-blocker initialization has failed twice, so will remain disabled!"
                    );
                }
            }
        }
    }

    async fn handle_ad_blocker_updated(
        &mut self,
        result: Result<Option<Box<AdBlocker>>, AdBlockerError>,
    ) {
        match result {
            Ok(Some(adblocker)) => {
                let mut guard = self.adblocker.lock().await;
                *guard = adblocker;
                tracing::debug!("Ad-blocker was updated successfully");
            }
            Ok(None) => {
                tracing::debug!("Ad-blocker is already up-to-date");
            }
            Err(error) => {
                tracing::error!("Failed to initialize or update ad-blocker: {error}");
            }
        }

        self.next_update_due = Instant::now() + Self::ADBLOCK_UPDATE_DELAY;
    }

    async fn handle_stopped_using_dns_filter(&mut self) {
        let mut guard = self.adblocker.lock().await;
        *guard = Box::new(AdBlocker::default());
        self.adblocker_initted = false;
        self.next_update_due = Instant::now() + Self::ADBLOCK_UPDATE_DELAY;
    }
}

enum AdBlockerTaskMessage {
    /// Initialize Ad-blocker.
    InitAdBlocker {
        /// Response channel when the task has accepted the command.
        response_tx: oneshot::Sender<()>,
    },

    /// Ad-blocker initialized in the background.
    AdBlockerInitted {
        result: Result<Box<AdBlocker>, AdBlockerError>,
        retry_count: usize,
    },

    /// Ad-blocker updated in the background.
    /// (it may not have actually updated if the data files didn't change)
    AdBlockerUpdated {
        result: Result<Option<Box<AdBlocker>>, AdBlockerError>,
    },

    /// Get the DNS filter
    GetDnsFilter {
        response_tx: oneshot::Sender<DnsFilter>,
    },

    /// Signal that we've stopped using the DNS filter, so we can free up memory used by the Ad-blocker filters
    StoppedUsingDnsFilter { response_tx: oneshot::Sender<()> },
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

    /// Initialize Ad-blocker.
    pub async fn init_ad_blocker(&self) {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(AdBlockerTaskMessage::InitAdBlocker { response_tx })
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

    /// Signal that we've stopped using the DNS filter, so we can free up memory
    /// used by the Ad-blocker filters
    pub async fn stopped_using_dns_filter(&self) {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(AdBlockerTaskMessage::StoppedUsingDnsFilter { response_tx })
            .is_ok()
        {
            response_rx.await.ok();
        }
    }
}
