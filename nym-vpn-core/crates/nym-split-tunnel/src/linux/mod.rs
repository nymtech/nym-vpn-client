// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Linux split-tunneling implementation using cgroups.
//!
//! It's recommended to read the kernel docs before delving into this module:
//! <https://docs.kernel.org/admin-guide/cgroup-v2.html>

pub mod pid_manager;
pub mod process_event_stream;

mod bindings;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};

use futures_util::{StreamExt, stream};
use libc::pid_t;
use nym_common::trace_err_chain;
use nym_vpn_lib_types::{SplitTunnelExcludedProcess, SplitTunnelExcludedProcessList};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use pid_manager::PidManager;
use process_event_stream::{ProcessEvent, ProcessEventStream};

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
    GetExcludedProcesses {
        result_tx: oneshot::Sender<Result<SplitTunnelExcludedProcessList, Error>>,
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

    pub async fn get_excluded_processes(&self) -> Result<SplitTunnelExcludedProcessList, Error> {
        let (result_tx, result_rx) = oneshot::channel();
        let _ = self
            .message_tx
            .send(Message::GetExcludedProcesses { result_tx });
        result_rx.await.map_err(|_| Error::unavailable())?
    }
}

pub struct SplitTunnel {
    message_rx: mpsc::UnboundedReceiver<Message>,
    pid_manager: Option<PidManager>,
    process_event_rx: mpsc::UnboundedReceiver<ProcessEvent>,
    process_event_stream_handle: Option<JoinHandle<()>>,
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

        let process_event_stream_handle = if pid_manager.is_some() {
            ProcessEventStream::spawn(process_event_tx, shutdown_token.child_token())
                .await
                .inspect_err(|err| trace_err_chain!(err, "failed to initialize split tunnel"))
                .ok()
        } else {
            None
        };

        let processes = process_list_snapshot().unwrap();
        let split_tunnel = Self {
            message_rx,
            pid_manager,
            process_event_rx,
            process_event_stream_handle,
            exclude_paths: HashSet::new(),
            processes,
            shutdown_token,
        };
        let join_handle = tokio::spawn(split_tunnel.run());

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
                        Message::GetExcludedProcesses { result_tx } => {
                            let _ = result_tx.send(self.get_excluded_processes());
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

        if let Some(process_monitor_handle) = self.process_event_stream_handle.take()
            && let Err(err) = process_monitor_handle.await
        {
            tracing::error!("Failed to join on process monitor handle: {err}");
        }
    }

    fn get_excluded_processes(&mut self) -> Result<SplitTunnelExcludedProcessList, Error> {
        if let Some(pid_manager) = self.pid_manager.as_mut() {
            let processes = pid_manager
                .list()
                .map_err(InnerError::ListPids)?
                .into_iter()
                .flat_map(|pid| {
                    let info = self.processes.get(&pid).map(|info| {
                        SplitTunnelExcludedProcess {
                            pid,
                            exec_path: info.exec_path.clone(),
                            // todo: unused on Linux, dupe exec_path
                            responsible_exec_path: info.exec_path.clone(),
                        }
                    });

                    if info.is_none() {
                        tracing::warn!("Pid {pid} is excluded but not found in process list");
                    }

                    info
                })
                .collect::<Vec<_>>();

            Ok(SplitTunnelExcludedProcessList { processes })
        } else {
            Err(Error::unavailable())
        }
    }

    async fn set_exclude_paths(&mut self, paths: HashSet<PathBuf>) -> Result<(), Error> {
        let pid_manager = self.pid_manager.as_ref().ok_or(Error::unavailable())?;

        // Resolve symlinks and canonicalize paths to align user and system paths
        let paths: HashSet<PathBuf> = stream::iter(paths)
            .then(|path| async move {
                tokio::fs::canonicalize(&path)
                    .await
                    .inspect_err(|err| {
                        if err.kind() != std::io::ErrorKind::NotFound {
                            tracing::warn!(
                                "Failed to canonicalize path: {}. Error: {}",
                                path.display(),
                                err
                            );
                        }
                    })
                    .unwrap_or(path)
            })
            .collect()
            .await;

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
                        trace_err_chain!(err, "failed to add exclusion for {pid}");
                    }
                }
                (false, true) => {
                    if let Err(err) = pid_manager.remove(*pid) {
                        trace_err_chain!(err, "failed to remove exclusion for {pid}");
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
                self.handle_fork(parent_pid, child_pid, path);
            }
            ProcessEvent::Exec { pid, path } => {
                self.handle_exec(pid, path);
            }
            ProcessEvent::Exit { pid } => {
                self.handle_exit(pid);
            }
        }
    }

    fn handle_fork(&mut self, parent_pid: pid_t, pid: pid_t, path: PathBuf) {
        // tracing::trace!("fork: parent={parent_pid} pid={pid} {}", path.display());

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

        if base_info.is_excluded() {
            tracing::trace!(
                "{pid} excluded (inherited from {parent_pid}) (exclude paths: {:?})",
                base_info.excluded_by_paths
            );

            if let Some(pid_manager) = self.pid_manager.as_mut()
                && let Err(err) = pid_manager.add(pid)
            {
                trace_err_chain!(err, "failed to add exclusion for {pid}");
            }
        }

        tracing::trace!("insert info for {pid}: {}", base_info.exec_path.display());
        self.processes.insert(pid, base_info);
    }

    fn handle_exec(&mut self, pid: pid_t, path: PathBuf) {
        // tracing::trace!("exec: {pid} {}", path.display());

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

        if info.is_excluded()
            && let Some(pid_manager) = self.pid_manager.as_mut()
            && let Err(err) = pid_manager.add(pid)
        {
            trace_err_chain!(err, "failed to add exclusion for {pid}");
        }
        tracing::trace!("update info for {pid}: {}", info.exec_path.display());
    }

    fn handle_exit(&mut self, pid: pid_t) {
        match self.processes.remove(&pid) {
            Some(info) => {
                tracing::trace!("remove info for {pid}: {}", info.exec_path.display());
                if info.is_excluded()
                    && let Some(pid_manager) = self.pid_manager.as_mut()
                    && let Err(err) = pid_manager.remove(pid)
                {
                    trace_err_chain!(err, "failed to remove exclusion for {pid}");
                }
            }
            None => {
                tracing::trace!("remove info for {pid}");
                tracing::warn!("exit syscall for unknown pid {pid}");
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
    fn from(_value: &Error) -> Self {
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
    #[error("failed to list pids in cgroup")]
    ListPids(#[source] nym_cgroup::Error),

    /// Split tunnel is unavailable
    #[error("split tunnel is unavailable")]
    Unavailable,
}

fn process_list_snapshot() -> procfs::ProcResult<HashMap<pid_t, ProcessInfo>> {
    let proc_iter = procfs::process::all_processes()?.filter_map(|proc| {
        match proc {
            Ok(proc) => Some(proc),
            Err(err) => {
                match err {
                    procfs::ProcError::NotFound(_) => {
                        // process vanished
                        None
                    }
                    procfs::ProcError::Io(err, path) => {
                        tracing::trace!("io error when listing process {path:?}: {err}");
                        None
                    }
                    err => {
                        tracing::error!("can't read process: {err}");
                        None
                    }
                }
            }
        }
    });

    let mut proc_by_pid = HashMap::new();

    for proc in proc_iter {
        match proc.exe() {
            Ok(exec_path) => {
                proc_by_pid.insert(
                    proc.pid(),
                    ProcessInfo {
                        exec_path,
                        excluded_by_paths: HashSet::new(),
                    },
                );
            }
            Err(err) => {
                // It's possible that process no longer exists
                if !matches!(err, procfs::ProcError::NotFound(_)) {
                    tracing::error!("failed to obtain exec path for {}: {}", proc.pid(), err);
                }
            }
        }
    }

    Ok(proc_by_pid)
}
