// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Linux split-tunneling implementation using cgroups.
//!
//! It's recommended to read the kernel docs before delving into this module:
//! <https://docs.kernel.org/admin-guide/cgroup-v2.html>

pub mod pid_manager;
pub mod process;

mod bindings;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};

use libc::pid_t;
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
pub struct ProcessInfo {
    pub exec_path: PathBuf,
    pub excluded_by_paths: HashSet<PathBuf>,
}

impl ProcessInfo {
    fn included(exec_path: PathBuf) -> Self {
        ProcessInfo {
            exec_path,
            excluded_by_paths: HashSet::new(),
        }
    }

    fn is_excluded(&self) -> bool {
        !self.excluded_by_paths.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct SplitTunnelHandle {
    message_tx: mpsc::UnboundedSender<Message>,
}

impl SplitTunnelHandle {
    pub async fn excluded_cgroup(&self) -> Result<Option<nym_cgroup::v2::CGroup2>, Error> {
        let (tx, rx) = oneshot::channel();
        self.message_tx
            .send(Message::GetExcludedCGroup { result_tx: tx })
            .map_err(|_| Error::unavailable())?;

        rx.await.map_err(|_| Error::unavailable())
    }

    pub async fn net_cls_classid(&self) -> Result<Option<u32>, Error> {
        let (tx, rx) = oneshot::channel();
        self.message_tx
            .send(Message::GetNetClsClassId { result_tx: tx })
            .map_err(|_| Error::unavailable())?;

        rx.await.map_err(|_| Error::unavailable())
    }

    pub async fn set_exclude_paths(&self, paths: HashSet<PathBuf>) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.message_tx
            .send(Message::SetExcludePaths {
                result_tx: tx,
                paths,
            })
            .map_err(|_| Error::unavailable())?;

        rx.await.map_err(|_| Error::unavailable())?
    }
}

pub struct SplitTunnel {
    message_rx: mpsc::UnboundedReceiver<Message>,
    pid_manager: Option<PidManager>,
    process_event_rx: mpsc::UnboundedReceiver<ProcessEvent>,
    process_monitor_handle: Option<JoinHandle<()>>,
    exclude_paths: HashSet<PathBuf>,
    processes: HashMap<pid_t, ProcessInfo>,
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
            exclude_paths: HashSet::new(),
            processes: HashMap::new(),
            shutdown_token,
        };
        let join_handle = tokio::spawn(st.run());

        (SplitTunnelHandle { message_tx }, join_handle)
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

        for (pid, info) in self.processes.iter_mut() {
            // Remove no-longer excluded paths from exclusion list
            let mut new_exclude_paths: HashSet<_> = info
                .excluded_by_paths
                .intersection(&paths)
                .cloned()
                .collect();

            match (
                paths.contains(&info.exec_path),
                new_exclude_paths.contains(&info.exec_path),
            ) {
                (true, false) => {
                    // Check if own path is excluded
                    new_exclude_paths.insert(info.exec_path.clone());

                    if let Err(err) = pid_manager.add(*pid) {
                        tracing::error!("Failed to add exclusion for {pid}: {err}");
                    }
                }
                (false, true) => {
                    if let Err(err) = pid_manager.remove(*pid) {
                        tracing::error!("Failed to remove exclusion for {pid}: {err}");
                    }
                }
                _ => {}
            }

            info.excluded_by_paths = new_exclude_paths;
        }

        Ok(())
    }

    async fn handle_process_event(&mut self, event: ProcessEvent) {
        match event {
            ProcessEvent::Fork {
                parent_pid,
                child_pid,
                path,
            } => {
                tracing::trace!("fork: {parent_pid} {child_pid} {}", path.display());
                self.handle_fork(parent_pid, child_pid, path);
            }
            ProcessEvent::Exec { pid, path } => {
                tracing::trace!("exec: {pid} {}", path.display());
                self.handle_exec(pid, path);
            }
            ProcessEvent::Exit {
                pid,
                parent_pid,
                path,
            } => {
                tracing::trace!("exit: {pid} {parent_pid} {}", path.display());
                self.handle_exit(pid, path);
            }
        }
    }

    fn handle_fork(&mut self, parent_pid: pid_t, pid: pid_t, path: PathBuf) {
        if self.processes.contains_key(&pid) {
            tracing::error!("Conflicting pid! State already contains {pid}");
        }

        // Inherit exclusion status from parent
        let base_info = match self.processes.get(&parent_pid) {
            Some(parent_info) => parent_info.to_owned(),
            None => {
                tracing::error!("{pid}: Unknown parent pid {parent_pid}!");
                ProcessInfo::included(path)
            }
        };

        // no exec yet; only pid and parent pid change
        if base_info.is_excluded() {
            tracing::trace!(
                "{pid} excluded (inherited from {parent_pid}) (exclude paths: {:?})",
                base_info.excluded_by_paths
            );
        }

        self.processes.insert(pid, base_info);
    }

    fn handle_exec(&mut self, pid: pid_t, path: PathBuf) {
        let Some(info) = self.processes.get_mut(&pid) else {
            tracing::error!("exec received for unknown pid {pid}");
            return;
        };

        info.exec_path = path;

        // Check if process is excluded directly by exec path
        if !info.excluded_by_paths.contains(&info.exec_path)
            && self.exclude_paths.contains(&info.exec_path)
        {
            info.excluded_by_paths.insert(info.exec_path.clone());
            tracing::trace!("Excluding {pid} by path: {}", info.exec_path.display());
        }
    }

    fn handle_exit(&mut self, pid: pid_t, _path: PathBuf) {
        if self.processes.remove(&pid).is_none() {
            tracing::error!("exit syscall for unknown pid {pid}");
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
    /// Split tunnel is unavailable
    #[error("split tunnel is unavailable")]
    Unavailable,
}
