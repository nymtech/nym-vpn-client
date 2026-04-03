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
use process_monitor::{ProcessEvent, ProcessMonitor, ProcessSnapshot};

use crate::SplitTunnelErrorCause;

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
                            // todo: try exiting error state
                            // preserve pid manager when entering error state to maintain the same cgroup/netcls
                            // which are set in firewall too during service initialization
                            result_tx.send(self.state.set_exclude_paths(paths).await).ok();
                        }
                        Message::GetExcludedProcesses { result_tx } => {
                            result_tx.send(self.state.get_excluded_processes()).ok();
                        }
                        Message::IsSplitTunnelSupported { result_tx } => {
                            result_tx.send(self.state.is_supported()).ok();
                        }
                    }
                }
                // Only query rx when ST is active since tx is dropped otherwise
                event = self.process_event_rx.recv(), if self.state.is_active() => {
                    match event {
                        Some(event) => {
                            self.state.handle_event(event);
                        }
                        None => {
                            tracing::warn!("Process monitor exited unexpectedly!");

                            let error = self.state.shutdown().await
                                .err()
                                .unwrap_or(Error::from(InnerError::Internal("Process monitor closed event channel but exited without error")));
                            trace_err_chain!(error, "Process monitor runtime error");

                            let st_error_cause = SplitTunnelErrorCause::from(&error);
                            self.state = State::Failed { error };

                            (self.error_handler)(st_error_cause);
                        }
                    };
                }
                _  = self.shutdown_token.cancelled() => {
                    self.state.shutdown().await.ok();
                    break;
                }
            }
        }
    }

    async fn start_split_tunnel(
        process_event_tx: UnboundedSender<ProcessEvent>,
        shutdown_token: CancellationToken,
    ) -> State {
        Self::start_split_tunnel_inner(process_event_tx, shutdown_token)
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
        let pid_manager = PidManager::new().map_err(InnerError::CreatePidManager)?;
        let process_monitor = ProcessMonitor::spawn(process_event_tx, shutdown_token.child_token())
            .await
            .map_err(InnerError::CreateProcessMonitor)?;

        Ok(State::Active(ActiveState {
            process_monitor_token: shutdown_token,
            pid_manager,
            process_monitor,
            processes: HashMap::new(),
            exclude_paths: HashSet::new(),
        }))
    }
}

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
    IsSplitTunnelSupported {
        result_tx: oneshot::Sender<bool>,
    },
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// Executable path
    ///
    /// Optional for the following reasons:
    ///
    /// - System processes may not advertise their associated executable paths
    /// - Application quit very quickly before process monitor was able to obtain the path
    ///
    exec_path: Option<PathBuf>,

    /// Paths of ancestor executables that led to spawn of this process
    ancestor_exec_paths: HashSet<PathBuf>,

    /// Paths by which this process is excluded from split tunnel
    excluded_by_paths: HashSet<PathBuf>,
}

impl ProcessInfo {
    fn included(exec_path: Option<PathBuf>) -> Self {
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

    pub async fn is_supported(&self) -> bool {
        let (result_tx, result_rx) = oneshot::channel();
        let _ = self
            .message_tx
            .send(Message::IsSplitTunnelSupported { result_tx });
        result_rx.await.unwrap_or_default()
    }
}

struct ActiveState {
    process_monitor_token: CancellationToken,
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
                let info = self
                    .processes
                    .get(&pid)
                    .map(|info| SplitTunnelExcludedProcess {
                        pid,
                        exec_path: info.exec_path.clone().unwrap_or_default(),
                        responsible_exec_path: PathBuf::default(), // unused on Linux
                        ancestor_exec_paths: Vec::from_iter(info.ancestor_exec_paths.clone()),
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
            if let Some(exec_path) = info.exec_path.as_ref()
                && paths.contains(exec_path)
                && !new_exclude_paths.contains(exec_path)
            {
                new_exclude_paths.insert(exec_path.clone());
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
                tracing::trace!("Add to exclusions {}: {:?}", pid, info.exec_path);
                if let Err(err) = self.pid_manager.add(*pid)
                    && !err.is_no_process_err()
                {
                    trace_err_chain!(err, "failed to add exclusion for {pid}");
                }
            } else if was_excluded && !is_excluded {
                tracing::trace!("Remove from exclusions {}: {:?}", pid, info.exec_path);
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

    fn handle_subscribed(&mut self, processes: Vec<ProcessSnapshot>) {
        self.processes = process_list_snapshot(processes);
    }

    fn handle_fork(&mut self, parent_pid: pid_t, pid: pid_t, path: Option<PathBuf>) {
        if self.processes.contains_key(&pid) {
            tracing::error!("Conflicting pid! State already contains {pid}");
        }

        // Inherit exclusion status from parent
        let base_info = match self.processes.get(&parent_pid) {
            Some(parent_info) => {
                let mut parent_info = parent_info.to_owned();
                if let Some(parent_exec_path) = parent_info.exec_path {
                    parent_info.ancestor_exec_paths.insert(parent_exec_path);
                }
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

    fn handle_exec(&mut self, pid: pid_t, path: Option<PathBuf>) {
        let Some(info) = self.processes.get_mut(&pid) else {
            tracing::error!("exec received for unknown pid {pid}");
            return;
        };

        info.exec_path = path;

        // Check if process is excluded directly by exec path
        if let Some(exec_path) = info.exec_path.as_ref()
            && !info.excluded_by_paths.contains(exec_path)
            && self.exclude_paths.contains(exec_path)
        {
            info.excluded_by_paths.insert(exec_path.clone());
            tracing::trace!("Excluding {pid} by path: {:?}", info.exec_path);
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
    fn handle_event(&mut self, event: ProcessEvent) {
        match self {
            State::Active(state) => match event {
                ProcessEvent::Subscribed { processes } => {
                    state.handle_subscribed(processes);
                }
                ProcessEvent::Fork {
                    parent_pid,
                    child_pid,
                    path,
                } => {
                    state.handle_fork(parent_pid, child_pid, path);
                }
                ProcessEvent::Exec { pid, path } => {
                    state.handle_exec(pid, path);
                }
                ProcessEvent::Exit { pid } => {
                    state.handle_exit(pid);
                }
            },
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

    fn is_supported(&self) -> bool {
        match self {
            State::Active(_) => true,
            State::Failed { error: _ } => {
                // this will mark ST unavailable even if error happened at runtime
                // todo: improve that once ST can exit error state
                false
            }
        }
    }

    async fn set_exclude_paths(&mut self, paths: HashSet<PathBuf>) -> Result<(), Error> {
        match self {
            State::Active(state) => state.set_exclude_paths(paths).await,
            State::Failed { error: _ } => Err(Error::unavailable()),
        }
    }

    async fn shutdown(self) -> Result<(), Error> {
        match self {
            State::Active(state) => {
                state.process_monitor_token.cancel();

                match state.process_monitor.await {
                    Ok(inner) => {
                        inner.map_err(|err| Error::from(InnerError::ProcessMonitorFailure(err)))
                    }
                    Err(err) => {
                        tracing::error!("Failed to join on process monitor handle: {err}");
                        Err(Error::from(InnerError::Panic(err)))
                    }
                }
            }
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
    CreatePidManager(#[source] pid_manager::Error),

    #[error("failed to create process monitor")]
    CreateProcessMonitor(#[source] process_monitor::Error),

    #[error("failed to list pids in cgroup")]
    ListPids(#[source] pid_manager::Error),

    #[error("process monitor runtime error")]
    ProcessMonitorFailure(#[source] process_monitor::Error),

    #[error("process monitor panic")]
    Panic(#[source] JoinError),

    #[error("internal error: {0}")]
    Internal(&'static str),

    #[error("split tunnel is unavailable")]
    Unavailable,
}

fn process_list_snapshot(snapshots: Vec<ProcessSnapshot>) -> HashMap<pid_t, ProcessInfo> {
    let mut proc_by_pid = HashMap::new();
    let mut ppid_table = HashMap::new();

    // Collect process info for all processes
    for proc in snapshots {
        // Build pid => ppid table
        ppid_table.insert(proc.pid, proc.parent_pid);
        proc_by_pid.insert(
            proc.pid,
            ProcessInfo {
                exec_path: proc.path,
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
                "Found ancestors for {}: exec: {:?}, ancestor execs: {:?}",
                pid,
                info.exec_path,
                info.ancestor_exec_paths
            );

            if let Some(exec_path) = info.exec_path.clone() {
                parent_paths.insert(exec_path);
            }
        }
    }

    proc_by_pid
}
