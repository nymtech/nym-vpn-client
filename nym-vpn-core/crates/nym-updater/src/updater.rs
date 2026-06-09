// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, pin::Pin, time::Duration};

use tokio::{
    sync::{mpsc, oneshot},
    time::{Instant, Sleep},
};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{UpdateOutcome, download::download_file, error::UpdaterError};

// Used to park the timer when no tasks are registered or all have been removed.
const IDLE_SLEEP: Duration = Duration::from_secs(365 * 24 * 3600);

struct OneShotRequest {
    url: Url,
    dest_path: PathBuf,
    result_tx: oneshot::Sender<Result<UpdateOutcome, UpdaterError>>,
}

struct RegisterRequest {
    url: Url,
    dest_path: PathBuf,
    interval: Duration,
    notify_tx: mpsc::UnboundedSender<Result<UpdateOutcome, UpdaterError>>,
}

enum Message {
    OneShot(OneShotRequest),
    Register(RegisterRequest),
}

struct ScheduledTask {
    url: Url,
    dest_path: PathBuf,
    interval: Duration,
    next_fire: Instant,
    notify_tx: mpsc::UnboundedSender<Result<UpdateOutcome, UpdaterError>>,
}

/// Cloneable handle for submitting file update requests to the [`Updater`] loop.
#[derive(Clone)]
pub struct UpdaterHandle {
    tx: mpsc::Sender<Message>,
}

impl UpdaterHandle {
    /// Request a one-shot download of `url` to `dest_path`.
    ///
    /// Blocks until the updater loop processes the request and returns the outcome.
    pub async fn request_update(
        &self,
        url: Url,
        dest_path: PathBuf,
    ) -> Result<UpdateOutcome, UpdaterError> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .send(Message::OneShot(OneShotRequest {
                url,
                dest_path,
                result_tx,
            }))
            .await
            .map_err(|_| UpdaterError::ChannelClosed)?;
        result_rx.await.map_err(|_| UpdaterError::ChannelClosed)?
    }

    /// Register a URL/file pair for periodic updating at the given `interval`.
    ///
    /// The first check fires after `interval`; subsequent checks fire every `interval` thereafter.
    /// Returns a receiver that yields each [`UpdateOutcome`] as it occurs.
    /// The registration is automatically removed when the returned receiver is dropped.
    pub async fn register(
        &self,
        url: Url,
        dest_path: PathBuf,
        interval: Duration,
    ) -> Result<mpsc::UnboundedReceiver<Result<UpdateOutcome, UpdaterError>>, UpdaterError> {
        let (notify_tx, notify_rx) = mpsc::unbounded_channel();
        self.tx
            .send(Message::Register(RegisterRequest {
                url,
                dest_path,
                interval,
                notify_tx,
            }))
            .await
            .map_err(|_| UpdaterError::ChannelClosed)?;
        Ok(notify_rx)
    }
}

/// File updater service. Runs a single async loop that processes one-shot download
/// requests and fires registered periodic tasks on schedule.
///
/// All work is serialised: only one download runs at a time.
pub struct Updater {
    rx: mpsc::Receiver<Message>,
    http_client: reqwest::Client,
}

impl Updater {
    /// Create a new `Updater` with a default HTTP client, returning the updater and a handle.
    pub fn new() -> Result<(Self, UpdaterHandle), UpdaterError> {
        let http_client = reqwest::Client::builder()
            .build()
            .map_err(|error| UpdaterError::BuildHttpClient { error })?;
        Ok(Self::with_client(http_client))
    }

    /// Create a new `Updater` using a pre-built `reqwest::Client`.
    pub fn with_client(http_client: reqwest::Client) -> (Self, UpdaterHandle) {
        let (tx, rx) = mpsc::channel(32);
        let updater = Self { rx, http_client };
        let handle = UpdaterHandle { tx };
        (updater, handle)
    }

    /// Run the updater loop until `cancel_token` is cancelled or all handles are dropped.
    pub async fn run(mut self, cancel_token: CancellationToken) {
        let mut tasks: Vec<ScheduledTask> = Vec::new();

        // A single pinned Sleep that acts as the fused periodic timer.
        // Parked at IDLE_SLEEP when no tasks are registered; reset to the earliest
        // next_fire whenever the task list changes or a tick is handled.
        let timer = tokio::time::sleep(IDLE_SLEEP);
        tokio::pin!(timer);

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    tracing::debug!("Updater loop cancelled");
                    break;
                }
                msg = self.rx.recv() => {
                    match msg {
                        Some(Message::OneShot(req)) => {
                            let result = download_file(
                                &self.http_client,
                                &req.url,
                                &req.dest_path,
                                cancel_token.child_token(),
                            )
                            .await;
                            let _ = req.result_tx.send(result);
                        }
                        Some(Message::Register(req)) => {
                            tracing::debug!(
                                "Registering periodic update task from {} to {} every {:?}",
                                req.url,
                                req.dest_path.display(),
                                req.interval,
                            );
                            tasks.push(ScheduledTask {
                                url: req.url,
                                dest_path: req.dest_path,
                                interval: req.interval,
                                next_fire: Instant::now() + req.interval,
                                notify_tx: req.notify_tx,
                            });
                            rearm_timer(timer.as_mut(), &tasks);
                        }
                        None => {
                            tracing::debug!("All updater handles dropped, exiting");
                            break;
                        }
                    }
                }
                _ = &mut timer => {
                    let now = Instant::now();
                    let idx = tasks
                        .iter()
                        .enumerate()
                        .filter(|(_, t)| t.next_fire <= now)
                        .min_by_key(|(_, t)| t.next_fire)
                        .map(|(i, _)| i);

                    if let Some(idx) = idx {
                        // Clone to avoid holding a borrow across the await.
                        let url = tasks[idx].url.clone();
                        let dest_path = tasks[idx].dest_path.clone();
                        let interval = tasks[idx].interval;

                        let result = download_file(
                            &self.http_client,
                            &url,
                            &dest_path,
                            cancel_token.child_token(),
                        )
                        .await;

                        if tasks[idx].notify_tx.send(result).is_err() {
                            // Receiver was dropped — unregister the task.
                            tracing::debug!(
                                dest = %dest_path.display(),
                                "Removing unsubscribed periodic update task",
                            );
                            tasks.remove(idx);
                        } else {
                            tasks[idx].next_fire = Instant::now() + interval;
                        }
                    }

                    // Re-arm for the next due task, or park if none remain.
                    rearm_timer(timer.as_mut(), &tasks);
                }
            }
        }
    }
}

/// Reset the timer to the earliest `next_fire` across all tasks, or park it at
/// `IDLE_SLEEP` from now if the task list is empty.
fn rearm_timer(timer: Pin<&mut Sleep>, tasks: &[ScheduledTask]) {
    let deadline = tasks
        .iter()
        .map(|t| t.next_fire)
        .min()
        .unwrap_or_else(|| Instant::now() + IDLE_SLEEP);
    timer.reset(deadline);
}
