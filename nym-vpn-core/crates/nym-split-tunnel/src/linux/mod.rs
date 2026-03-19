// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Linux split-tunneling implementation using cgroups.
//!
//! It's recommended to read the kernel docs before delving into this module:
//! <https://docs.kernel.org/admin-guide/cgroup-v2.html>

pub mod pid_manager;
pub mod process;

mod bindings;

use std::{collections::HashSet, path::PathBuf, sync::Arc};

use nym_common::trace_err_chain;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use pid_manager::PidManager;
use process::{ProcessEvent, ProcessMonitor};

use crate::SplitTunnelErrorCause;

pub enum Message {
    SetExcludePaths {
        result_tx: oneshot::Sender<Result<(), Error>>,
        paths: HashSet<PathBuf>,
    },
    GetExcludedCGroup {
        result_tx: oneshot::Sender<Option<nym_cgroup::v2::CGroup2>>,
    },
    GetNetClsClassId {
        result_tx: oneshot::Sender<Option<u32>>,
    },
}

#[derive(Debug, Clone)]
pub struct SplitTunnelHandle {
    message_tx: mpsc::UnboundedSender<Message>,
}

pub struct SplitTunnel {
    message_rx: mpsc::UnboundedReceiver<Message>,
    pid_manager: Option<PidManager>,
    process_event_rx: mpsc::UnboundedReceiver<ProcessEvent>,
    process_monitor_handle: Option<JoinHandle<()>>,
    excluded_paths: HashSet<PathBuf>,
    shutdown_token: CancellationToken,
}

impl SplitTunnel {
    pub async fn spawn(shutdown_token: CancellationToken) -> (SplitTunnelHandle, JoinHandle<()>) {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (process_event_tx, process_event_rx) = mpsc::unbounded_channel();

        let pid_manager = PidManager::new()
            .inspect_err(|err| trace_err_chain!(err, "failed to initialize split tunnel"))
            .ok();

        let process_monitor_handle = if pid_manager.is_some() {
            ProcessMonitor::spawn(process_event_tx, shutdown_token.child_token())
                .await
                .inspect_err(|err| trace_err_chain!(err, "failed to initialize split tunnel"))
                .ok()
        } else {
            None
        };

        let st = Self {
            message_rx,
            pid_manager,
            process_event_rx,
            process_monitor_handle,
            excluded_paths: HashSet::new(),
            shutdown_token,
        };
        let join_handle = tokio::spawn(st.run());

        (
            SplitTunnelHandle {
                message_tx,
            },
            join_handle,
        )
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.message_rx.recv() => {
                    match msg {
                        Message::GetNetClsClassId { result_tx } =>{
                            result_tx.send(self.pid_manager.as_ref().and_then(|pid_manager| pid_manager.net_cls_classid())).ok();
                        }
                        Message::GetExcludedCGroup { result_tx } => {
                            result_tx.send(self.pid_manager.as_ref().and_then(|pid_manager| pid_manager.excluded_cgroup())).ok();
                        }
                        Message::SetExcludePaths { result_tx, paths } => {
                            result_tx.send(self.set_exclude_paths(paths).await).ok();
                        }
                    }
                }
                Some(event) = self.process_event_rx.recv() => {
                    self.handle_process_event(event).await;
                }
                _  = self.shutdown_token.cancelled() => {
                    break;
                }
            }
        }

        if let Some(process_monitor_handle) = self.process_monitor_handle.take()
            && let Err(err) = process_monitor_handle.await
        {
            tracing::error!("Failed to join on process monitor handle: {err}");
        }
    }

    async fn set_exclude_paths(&mut self, paths: HashSet<PathBuf>) -> Result<(), Error> {
        let pid_manager = self.pid_manager.as_ref().ok_or(Error::unavailable())?;

        self.

        todo!()
    }

    async fn handle_process_event(&mut self, event: ProcessEvent) {
        match event {
            ProcessEvent::Fork {
                parent_pid,
                child_pid,
                path,
            } => {
                todo!()
            }
            ProcessEvent::Exec { pid, path } => {
                todo!()
            }
            ProcessEvent::Exit {
                pid,
                parent_pid,
                path,
            } => {
                todo!()
            }
        }
    }
}

/// Errors caused by split tunneling
#[derive(Debug, Clone)]
pub struct Error {
    inner: Arc<InnerError>,
}

impl Error {
    fn unavailable() -> Self {
        Self {
            inner: Arc::new(InnerError::Unavailable),
        }
    }
}

impl From<&Error> for SplitTunnelErrorCause {
    fn from(value: &Error) -> Self {
        Self::Other
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&*self.inner, f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl<T: Into<InnerError>> From<T> for Error {
    fn from(inner: T) -> Self {
        Self {
            inner: Arc::new(inner.into()),
        }
    }
}

/// Errors caused by split tunneling
#[derive(thiserror::Error, Debug)]
enum InnerError {
    /// Process monitor error
    #[error("failed to initialize pid manager")]
    PidManager(#[source] pid_manager::Error),

    /// Process manager error
    #[error("failed to initialize process monitor")]
    ProcessMonitor(#[source] std::io::Error),

    /// Split tunnel is unavailable
    #[error("split tunnel is unavailable")]
    Unavailable,
}
