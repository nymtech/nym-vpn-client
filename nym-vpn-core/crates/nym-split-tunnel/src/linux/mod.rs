// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Linux split-tunneling implementation using cgroups.
//!
//! It's recommended to read the kernel docs before delving into this module:
//! <https://docs.kernel.org/admin-guide/cgroup-v2.html>

pub mod pid_manager;
pub mod process_monitor;

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
    sync::{
        mpsc::{self, UnboundedSender},
        oneshot,
    },
    task::{JoinError, JoinHandle},
};
use tokio_util::sync::CancellationToken;

use pid_manager::PidManager;
use process_monitor::{ProcessEvent, ProcessMonitor};

use crate::SplitTunnelErrorCause;

pub enum Message {
    SetExcludePaths {
        result_tx: oneshot::Sender<Result<(), Error>>,
        paths: HashSet<PathBuf>,
    },
    GetExcludedCGroup {
        result_tx: oneshot::Sender<Result<Option<nym_cgroup::v2::CGroup2>, Error>>,
    },
    GetNetClsClassId {
        result_tx: oneshot::Sender<Result<Option<u32>, Error>>,
    },
    GetExcludedProcesses {
        result_tx: oneshot::Sender<Result<SplitTunnelExcludedProcessList, Error>>,
    },
    IsSplitTunnelAvailable {
        result_tx: oneshot::Sender<bool>,
    },
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    exec_path: PathBuf,
    // Paths of ancestor executables that led to spawn of this process
    ancestor_exec_paths: HashSet<PathBuf>,
    excluded_by_paths: HashSet<PathBuf>,
}

impl ProcessInfo {
    fn included(exec_path: PathBuf) -> Self {
        ProcessInfo {
            exec_path,
            ancestor_exec_paths: HashSet::new(),
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
            .ok();
        rx.await.map_err(|_| Error::unavailable())?
    }

    pub async fn net_cls_classid(&self) -> Result<Option<u32>, Error> {
        let (tx, rx) = oneshot::channel();
        self.message_tx
            .send(Message::GetNetClsClassId { result_tx: tx })
            .ok();
        rx.await.map_err(|_| Error::unavailable())?
    }

    pub async fn set_exclude_paths(&self, paths: HashSet<PathBuf>) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.message_tx
            .send(Message::SetExcludePaths {
                result_tx: tx,
                paths,
            })
            .ok();
        rx.await.map_err(|_| Error::unavailable())?
    }

    pub async fn get_excluded_processes(&self) -> Result<SplitTunnelExcludedProcessList, Error> {
        let (result_tx, result_rx) = oneshot::channel();
        let _ = self
            .message_tx
            .send(Message::GetExcludedProcesses { result_tx });
        result_rx.await.map_err(|_| Error::unavailable())?
    }

    pub async fn is_available(&self) -> bool {
        let (result_tx, result_rx) = oneshot::channel();
        let _ = self
            .message_tx
            .send(Message::IsSplitTunnelAvailable { result_tx });
        result_rx.await.unwrap_or_default()
    }
}

pub struct SplitTunnel {
    state: State,
    error_handler: Box<dyn Fn(SplitTunnelErrorCause) + Send>,
    message_rx: mpsc::UnboundedReceiver<Message>,
    process_event_rx: mpsc::UnboundedReceiver<ProcessEvent>,
    shutdown_token: CancellationToken,
}

impl SplitTunnel {
    pub async fn spawn<F>(
        shutdown_token: CancellationToken,
        error_handler: F,
    ) -> (SplitTunnelHandle, JoinHandle<()>)
    where
        F: Fn(SplitTunnelErrorCause) + Send + 'static,
    {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (process_event_tx, process_event_rx) = mpsc::unbounded_channel();

        let split_tunnel = Self {
            state: Self::start_split_tunnel(process_event_tx, shutdown_token.child_token()).await,
            error_handler: Box::new(error_handler),
            message_rx,
            process_event_rx,
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
                            result_tx.send(self.state.net_cls_classid()).ok();
                        }
                        Message::GetExcludedCGroup { result_tx } => {
                            result_tx.send(self.state.excluded_cgroup()).ok();
                        }
                        Message::SetExcludePaths { result_tx, paths } => {
                            result_tx.send(self.set_exclude_paths(paths).await).ok();
                        }
                        Message::GetExcludedProcesses { result_tx } => {
                            result_tx.send(self.state.get_excluded_processes()).ok();
                        }
                        Message::IsSplitTunnelAvailable { result_tx } => {
                            result_tx.send(self.state.is_active()).ok();
                        }
                    }
                }
                // Only query rx when ST is active since tx is dropped otherwise
                event = self.process_event_rx.recv(), if self.state.is_active() => {
                    match event {
                        Some(event) => self.state.handle_event(event).await,
                        None => {
                            tracing::warn!("Process monitor exited unexpectedly!");

                            let error = self.state.wait_for_shutdown().await
                                .err()
                                .unwrap_or(Error::from(InnerError::Internal("Process monitor closed event channel but exited without error")));
                            trace_err_chain!(error, "Process monitor runtime error");

                            let st_error_cause = SplitTunnelErrorCause::from(&error);
                            self.state = State::Failed {
                                error
                            };
                            // Report error to tunnel state machine
                            (self.error_handler)(st_error_cause);
                        }
                    };
                }
                _  = self.shutdown_token.cancelled() => {
                    self.state.wait_for_shutdown().await.ok();
                    break;
                }
            }
        }
    }

    async fn set_exclude_paths(&mut self, paths: HashSet<PathBuf>) -> Result<(), Error> {
        match self.state {
            State::Active(ref mut state) => state.set_exclude_paths(paths).await,
            State::Failed { error: _ } => {
                tracing::debug!("Transitioning out of split tunnel error state");

                // Restart ST on attempt to refresh exclude paths
                let (process_event_tx, process_event_rx) = mpsc::unbounded_channel();
                self.process_event_rx = process_event_rx;

                self.state =
                    Self::start_split_tunnel(process_event_tx, self.shutdown_token.child_token())
                        .await;
                self.state.set_exclude_paths(paths).await
            }
        }
    }

    async fn start_split_tunnel(
        process_event_tx: UnboundedSender<ProcessEvent>,
        shutdown_token: CancellationToken,
    ) -> State {
        Self::start_split_tunnel_inner(process_event_tx, shutdown_token.child_token())
            .await
            .inspect_err(|err| {
                trace_err_chain!(err, "failed to initialize split tunnel");
            })
            .unwrap_or_else(|error| State::Failed { error })
    }

    async fn start_split_tunnel_inner(
        process_event_tx: mpsc::UnboundedSender<ProcessEvent>,
        shutdown_token: CancellationToken,
    ) -> Result<State, Error> {
        let pid_manager =
            PidManager::new().map_err(|err| Error::from(InnerError::CreatePidManager(err)))?;

        let process_monitor = ProcessMonitor::spawn(process_event_tx, shutdown_token)
            .await
            .map_err(InnerError::CreateProcessMonitor)?;

        let processes = process_list_snapshot().map_err(InnerError::ProcessListSnapshot)?;

        Ok(State::Active(ActiveState {
            pid_manager,
            process_monitor,
            processes,
            exclude_paths: HashSet::new(),
        }))
    }
}

struct ActiveState {
    process_monitor: JoinHandle<Result<(), process_monitor::Error>>,
    pid_manager: PidManager,
    exclude_paths: HashSet<PathBuf>,
    processes: HashMap<pid_t, ProcessInfo>,
}

impl ActiveState {
    fn net_cls_classid(&self) -> Option<u32> {
        self.pid_manager.net_cls_classid()
    }

    fn excluded_cgroup(&self) -> Option<nym_cgroup::v2::CGroup2> {
        self.pid_manager.excluded_cgroup()
    }

    fn get_excluded_processes(&mut self) -> Result<SplitTunnelExcludedProcessList, Error> {
        let processes = self
            .pid_manager
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
    }

    async fn set_exclude_paths(&mut self, paths: HashSet<PathBuf>) -> Result<(), Error> {
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

            // Check if own path is excluded
            if paths.contains(&info.exec_path) && !new_exclude_paths.contains(&info.exec_path) {
                new_exclude_paths.insert(info.exec_path.clone());
            }

            // Check if path for ancestor app is excluded
            for ancestor_exec_path in info.ancestor_exec_paths.iter() {
                if paths.contains(ancestor_exec_path)
                    && !new_exclude_paths.contains(ancestor_exec_path)
                {
                    new_exclude_paths.insert(ancestor_exec_path.clone());
                }
            }

            let was_excluded = info.is_excluded();
            info.excluded_by_paths = new_exclude_paths;
            let is_excluded = info.is_excluded();

            if !was_excluded && is_excluded {
                tracing::trace!("Add to exclusions {}: {}", pid, info.exec_path.display());
                if let Err(err) = self.pid_manager.add(*pid)
                    && !err.is_no_process_err()
                {
                    trace_err_chain!(err, "failed to add exclusion for {pid}");
                }
            } else if was_excluded && !is_excluded {
                tracing::trace!(
                    "Remove from exclusions {}: {}",
                    pid,
                    info.exec_path.display()
                );
                if let Err(err) = self.pid_manager.remove(*pid)
                    && !err.is_no_process_err()
                {
                    trace_err_chain!(err, "failed to remove exclusion for {pid}");
                }
            }
        }

        self.exclude_paths = paths;

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
        if self.processes.contains_key(&pid) {
            tracing::error!("Conflicting pid! State already contains {pid}");
        }

        // Inherit exclusion status from parent
        let base_info = match self.processes.get(&parent_pid) {
            Some(parent_info) => {
                let mut parent_info = parent_info.to_owned();
                parent_info
                    .ancestor_exec_paths
                    .insert(parent_info.exec_path);
                parent_info.exec_path = path;
                parent_info
            }
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

            if let Err(err) = self.pid_manager.add(pid)
                && !err.is_no_process_err()
            {
                trace_err_chain!(err, "failed to add exclusion for {pid}");
            }
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

        // Check if process is excluded indirectly by one of ascendants
        for ancestor_exec_path in info.ancestor_exec_paths.iter() {
            if self.exclude_paths.contains(ancestor_exec_path) {
                tracing::trace!(
                    "Excluding {pid} by ancestor path: {}",
                    ancestor_exec_path.display()
                );
                info.excluded_by_paths.insert(ancestor_exec_path.clone());
            }
        }

        if info.is_excluded()
            && let Err(err) = self.pid_manager.add(pid)
            && !err.is_no_process_err()
        {
            trace_err_chain!(err, "failed to add exclusion for {pid}");
        }
    }

    fn handle_exit(&mut self, pid: pid_t) {
        if self.processes.remove(&pid).is_none() {
            tracing::error!("exit syscall for unknown pid {pid}");
        }
    }
}

enum State {
    Active(ActiveState),
    Failed {
        #[allow(unused)]
        error: Error,
    },
}

impl State {
    async fn handle_event(&mut self, event: ProcessEvent) {
        match self {
            State::Active(state) => {
                state.handle_process_event(event).await;
            }
            State::Failed { error: _ } => {}
        }
    }

    fn net_cls_classid(&self) -> Result<Option<u32>, Error> {
        match self {
            State::Active(state) => Ok(state.net_cls_classid()),
            State::Failed { error: _ } => Err(Error::unavailable()),
        }
    }

    fn excluded_cgroup(&self) -> Result<Option<nym_cgroup::v2::CGroup2>, Error> {
        match self {
            State::Active(state) => Ok(state.excluded_cgroup()),
            State::Failed { error: _ } => Err(Error::unavailable()),
        }
    }

    fn get_excluded_processes(&mut self) -> Result<SplitTunnelExcludedProcessList, Error> {
        match self {
            State::Active(state) => state.get_excluded_processes(),
            State::Failed { error: _ } => Err(Error::unavailable()),
        }
    }

    fn is_active(&self) -> bool {
        matches!(self, State::Active(..))
    }

    async fn set_exclude_paths(&mut self, paths: HashSet<PathBuf>) -> Result<(), Error> {
        match self {
            State::Active(state) => state.set_exclude_paths(paths).await,
            State::Failed { error: _ } => Err(Error::unavailable()),
        }
    }

    async fn wait_for_shutdown(self) -> Result<(), Error> {
        match self {
            State::Active(state) => match state.process_monitor.await {
                Ok(inner) => {
                    inner.map_err(|err| Error::from(InnerError::ProcessMonitorFailure(err)))
                }
                Err(err) => {
                    tracing::error!("Failed to join on process monitor handle: {err}");
                    Err(Error::from(InnerError::ProcessMonitorPanicked(err)))
                }
            },
            State::Failed { error: _ } => Ok(()),
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
    #[error("failed to create pid manager")]
    CreatePidManager(#[source] nym_cgroup::Error),

    #[error("failed to create process monitor")]
    CreateProcessMonitor(#[source] process_monitor::Error),

    #[error("failed to create initial process list snapshot")]
    ProcessListSnapshot(#[source] procfs::ProcError),

    #[error("failed to list pids in cgroup")]
    ListPids(#[source] nym_cgroup::Error),

    #[error("process monitor runtime error")]
    ProcessMonitorFailure(#[source] process_monitor::Error),

    #[error("process monitor panic")]
    ProcessMonitorPanicked(#[source] JoinError),

    #[error("internal error: {0}")]
    Internal(&'static str),

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
                        tracing::error!("Can't read process: {err}");
                        None
                    }
                }
            }
        }
    });

    let mut proc_by_pid = HashMap::new();
    let mut ppid_table = HashMap::new();

    // Collect process info for all processes
    for proc in proc_iter {
        // Build pid => ppid table
        match proc.status() {
            Ok(status) => {
                ppid_table.insert(proc.pid(), status.ppid);
            }
            Err(procfs::ProcError::NotFound(_)) => {
                // process vanished
            }
            Err(err) => {
                tracing::error!("Can't obtain process status {}: {}", proc.pid(), err);
            }
        }

        proc_by_pid.insert(
            proc.pid(),
            ProcessInfo {
                // Some system processes do not adverise their path, ex: /proc/2/exe (kthreadd)
                // todo: handle this in some way to prevent excluding all system processes using blank path.
                exec_path: proc.exe().unwrap_or_default(),
                ancestor_exec_paths: HashSet::new(),
                excluded_by_paths: HashSet::new(),
            },
        );
    }

    // Populate process info with paths to ancestor executables
    let all_pids = proc_by_pid.keys().copied().collect::<Vec<_>>();
    for pid in all_pids {
        // Build child-parent relationship chain
        let mut pid_chain = Vec::new();
        let mut cur_pid = Some(pid);
        while let Some(pid) = cur_pid.take() {
            pid_chain.push(pid);

            // Locate parent pid
            let Some(ppid) = ppid_table.get(&pid) else {
                tracing::warn!("No parent pid found for {pid}");
                continue;
            };

            // No parent process
            if *ppid == 0 {
                continue;
            }

            // Break circular references
            if pid_chain.iter().any(|processed_pid| processed_pid == ppid) {
                tracing::warn!("Parent pid {ppid} is already in the pid chain!");
            } else {
                cur_pid = Some(*ppid);
            }
        }

        // Walk pid chain in reverse to collect paths to all parent executables into single set.
        let mut parent_paths = HashSet::new();
        for pid in pid_chain.iter().rev() {
            let Some(info) = proc_by_pid.get_mut(pid) else {
                tracing::warn!("No process info found for {pid}");
                continue;
            };

            info.ancestor_exec_paths.extend(parent_paths.clone());
            tracing::trace!(
                "Found ancestors for {}: exec: {}, ancestor execs: {:?}",
                pid,
                info.exec_path.display(),
                info.ancestor_exec_paths
            );

            parent_paths.insert(info.exec_path.clone());
        }
    }

    Ok(proc_by_pid)
}
