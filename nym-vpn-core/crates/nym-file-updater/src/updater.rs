// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, pin::Pin, time::Duration};

use tokio::{
    sync::{mpsc, oneshot},
    time::{Instant, Sleep, sleep},
};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{UpdateOutcome, download::download_file, error::FileUpdaterError};

type TaskId = u64;

struct OneShotRequest {
    url: Url,
    dest_path: PathBuf,
    result_tx: oneshot::Sender<Result<UpdateOutcome, FileUpdaterError>>,
}

struct RegisterRequest {
    url: Url,
    dest_path: PathBuf,
    initial_delay: Duration,
    interval: Duration,
    notify_tx: mpsc::UnboundedSender<Result<UpdateOutcome, FileUpdaterError>>,
}

enum Message {
    OneShot(OneShotRequest),
    Register(RegisterRequest),
}

struct ScheduledTask {
    id: TaskId,
    url: Url,
    dest_path: PathBuf,
    interval: Duration,
    next_fire: Instant,
    in_flight: bool,
    notify_tx: mpsc::UnboundedSender<Result<UpdateOutcome, FileUpdaterError>>,
}

struct PeriodicCompletion {
    task_id: TaskId,
    result: Result<UpdateOutcome, FileUpdaterError>,
}

/// Cloneable handle for submitting file update requests to the [`FileUpdater`] loop.
#[derive(Clone)]
pub struct FileUpdaterHandle {
    tx: mpsc::UnboundedSender<Message>,
}

impl FileUpdaterHandle {
    /// Request a one-shot download of `url` to `dest_path`.
    ///
    /// Blocks until the updater loop processes the request and returns the outcome.
    pub async fn request_update(
        &self,
        url: Url,
        dest_path: PathBuf,
    ) -> Result<UpdateOutcome, FileUpdaterError> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .send(Message::OneShot(OneShotRequest {
                url,
                dest_path,
                result_tx,
            }))
            .map_err(|_| FileUpdaterError::ChannelClosed)?;
        result_rx
            .await
            .map_err(|_| FileUpdaterError::ChannelClosed)?
    }

    /// Register a URL/file pair for periodic updating.
    ///
    /// The first check fires after `initial_delay`; subsequent checks fire every
    /// `interval` thereafter.  Returns a receiver that yields each [`UpdateOutcome`]
    /// as it occurs.  The registration is automatically removed when the returned
    /// receiver is dropped.
    pub async fn register(
        &self,
        url: Url,
        dest_path: PathBuf,
        initial_delay: Duration,
        interval: Duration,
    ) -> Result<mpsc::UnboundedReceiver<Result<UpdateOutcome, FileUpdaterError>>, FileUpdaterError>
    {
        let (notify_tx, notify_rx) = mpsc::unbounded_channel();
        self.tx
            .send(Message::Register(RegisterRequest {
                url,
                dest_path,
                initial_delay,
                interval,
                notify_tx,
            }))
            .map_err(|_| FileUpdaterError::ChannelClosed)?;
        Ok(notify_rx)
    }
}

impl FileUpdaterHandle {
    /// Create a disconnected handle for use in tests.
    ///
    /// All `register` and `request_update` calls will return
    /// `Err(FileUpdaterError::ChannelClosed)`, which the adblocker treats as a soft
    /// error (runs without scheduled updates).
    pub fn new_test() -> Self {
        let (tx, _rx) = mpsc::unbounded_channel();
        FileUpdaterHandle { tx }
    }
}

/// File updater service. Runs a single async loop that processes one-shot download
/// requests and fires registered periodic tasks on schedule.
///
/// Downloads are spawned onto separate tasks so the message loop stays responsive
/// while a download is in progress. Each download builds its own
/// [`nym_http_api_client::Client`] bound to its target URL, so no client is held here.
pub struct FileUpdater {
    rx: mpsc::UnboundedReceiver<Message>,
}

impl FileUpdater {
    /// Create a new `FileUpdater`, returning the updater and a handle.
    pub fn new() -> (Self, FileUpdaterHandle) {
        let (tx, rx) = mpsc::unbounded_channel();
        let file_updater = Self { rx };
        let handle = FileUpdaterHandle { tx };
        (file_updater, handle)
    }

    /// Run the updater loop until `cancel_token` is cancelled or all handles are dropped.
    pub async fn run(mut self, cancel_token: CancellationToken) {
        let mut tasks: Vec<ScheduledTask> = Vec::new();
        let mut next_id: TaskId = 0;
        let (completion_tx, mut completion_rx) = mpsc::unbounded_channel::<PeriodicCompletion>();

        // A single pinned Sleep used as the periodic timer. The timer branch in
        // select! is guarded so it is never polled when all tasks are idle or in-flight.
        // `rearm_timer` keeps it pointed at the earliest non-in-flight next_fire.
        let timer = sleep(Duration::ZERO);
        tokio::pin!(timer);

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    tracing::debug!("File updater loop cancelled");
                    break;
                }
                msg = self.rx.recv() => {
                    match msg {
                        Some(Message::OneShot(req)) => {
                            let cancel = cancel_token.child_token();
                            tokio::spawn(async move {
                                let result =
                                    download_file(&req.url, &req.dest_path, cancel).await;
                                let _ = req.result_tx.send(result);
                            });
                        }
                        Some(Message::Register(req)) => {
                            tracing::debug!(
                                "Registering periodic update task from {} to {} (initial delay {:?}, interval {:?})",
                                req.url,
                                req.dest_path.display(),
                                req.initial_delay,
                                req.interval,
                            );
                            tasks.push(ScheduledTask {
                                id: next_id,
                                url: req.url,
                                dest_path: req.dest_path,
                                interval: req.interval,
                                next_fire: Instant::now() + req.initial_delay,
                                in_flight: false,
                                notify_tx: req.notify_tx,
                            });
                            next_id += 1;
                            rearm_timer(timer.as_mut(), &tasks);
                        }
                        None => {
                            tracing::debug!("All file updater handles dropped, exiting");
                            break;
                        }
                    }
                }
                _ = &mut timer, if tasks.iter().any(|t| !t.in_flight) => {
                    let now = Instant::now();
                    let idx = tasks
                        .iter()
                        .enumerate()
                        .filter(|(_, t)| !t.in_flight && t.next_fire <= now)
                        .min_by_key(|(_, t)| t.next_fire)
                        .map(|(i, _)| i);

                    if let Some(idx) = idx {
                        let task_id = tasks[idx].id;
                        let url = tasks[idx].url.clone();
                        let dest_path = tasks[idx].dest_path.clone();
                        let cancel = cancel_token.child_token();
                        let completion_tx = completion_tx.clone();

                        tasks[idx].in_flight = true;

                        tokio::spawn(async move {
                            let result = download_file(&url, &dest_path, cancel).await;
                            let _ = completion_tx.send(PeriodicCompletion { task_id, result });
                        });
                    }

                    rearm_timer(timer.as_mut(), &tasks);
                }
                Some(completion) = completion_rx.recv() => {
                    if let Some(idx) = tasks.iter().position(|t| t.id == completion.task_id) {
                        tasks[idx].in_flight = false;
                        let interval = tasks[idx].interval;
                        if tasks[idx].notify_tx.send(completion.result).is_err() {
                            // Receiver was dropped — unregister the task.
                            tracing::debug!(
                                dest = %tasks[idx].dest_path.display(),
                                "Removing unsubscribed periodic update task",
                            );
                            tasks.remove(idx);
                        } else {
                            tasks[idx].next_fire = Instant::now() + interval;
                        }
                        rearm_timer(timer.as_mut(), &tasks);
                    }
                }
            }
        }
    }
}

/// Reset the timer to the earliest `next_fire` among non-in-flight tasks. No-op
/// when all tasks are in-flight or the list is empty; the guard in `select!`
/// prevents the timer branch from being polled in those cases.
fn rearm_timer(timer: Pin<&mut Sleep>, tasks: &[ScheduledTask]) {
    if let Some(deadline) = tasks
        .iter()
        .filter(|t| !t.in_flight)
        .map(|t| t.next_fire)
        .min()
    {
        timer.reset(deadline);
    }
}
