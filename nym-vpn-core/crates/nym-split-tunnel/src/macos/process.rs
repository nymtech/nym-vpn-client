// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! This module keeps tracks of maintains a list of processes, and keeps it up to date by observing
//! the syscalls `fork`, `exec`, and `exit`.
//! Each process has an exclusion state, based on which paths the process monitor is instructed to
//! exclude.
//! The module currently relies on the `eslogger` tool to do so, which in turn relies on the
//! Endpoint Security framework.

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    io,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, LazyLock, Once},
};

use futures_util::{StreamExt, stream};
use libc::pid_t;
use nym_macos::process::{list_pids, process_path};
use nym_platform_metadata::AppleVersion;
use nym_vpn_lib_types::{SplitTunnelExcludedProcess, SplitTunnelExcludedProcessList};
use tokio::sync::{Mutex, oneshot};

use crate::SplitTunnelErrorCause;

/// Minimum macOS version supported by split tunnel
static MIN_OS_VERSION: LazyLock<AppleVersion> =
    LazyLock::new(|| AppleVersion::from_str("11.0.0").unwrap());

/// Endpoint-sec one-time initialization token
static ENDPOINT_SEC_INIT: Once = Once::new();

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Only macOS 13 and later is supported
    #[error("Unsupported macOS version: {actual}, expected at least {}", *MIN_OS_VERSION)]
    UnsupportedMacosVersion {
        actual: nym_platform_metadata::AppleVersion,
    },

    /// The app requires TCC approval from the user.
    #[error("the app needs TCC approval from the user for Full Disk Access")]
    NeedFullDiskPermissions,

    /// Failed to list processes
    #[error("failed to list processes")]
    InitializePids(#[source] io::Error),

    /// Failed to find path for a process
    #[error("failed to find path for a process: {}", _0)]
    FindProcessPath(#[source] io::Error, pid_t),

    #[error("failed to create ES client: {0}")]
    CreateEsClient(#[source] endpoint_sec::sys::NewClientError),

    #[error("failed to subscribe to ES events")]
    Subscribe(#[source] endpoint_sec::sys::ReturnError),

    #[error("esclient initialization channel closed")]
    EsClientInitChannelClosed,
}

impl From<&Error> for SplitTunnelErrorCause {
    fn from(value: &Error) -> Self {
        match value {
            Error::NeedFullDiskPermissions => SplitTunnelErrorCause::NeedFullDiskPermissions,
            _ => SplitTunnelErrorCause::Other,
        }
    }
}

pub struct ProcessMonitor(());

#[derive(Debug)]
pub struct ProcessMonitorHandle {
    states: ProcessStates,
    client_thread: std::thread::JoinHandle<()>,
    stop_tx: oneshot::Sender<()>,
}

impl ProcessMonitor {
    pub async fn spawn() -> Result<ProcessMonitorHandle, Error> {
        check_os_version_support()?;
        init_endpoint_sec();

        let states = ProcessStates::new()?;
        let cloned_states = states.clone();
        let (msg_tx, mut msg_rx) = tokio::sync::mpsc::channel(1);
        let (stop_tx, stop_rx) = oneshot::channel();
        let (create_client_tx, create_client_rx) = oneshot::channel();

        // ESClient must be initialized and dropped on the same thread
        let client_thread = std::thread::spawn(move || {
            let esclient = match Self::create_esclient(msg_tx) {
                Ok(esclient) => {
                    _ = create_client_tx.send(None);
                    esclient
                }
                Err(err) => {
                    tracing::error!("Failed to initialize esclient: {err}");
                    _ = create_client_tx.send(Some(err));
                    return;
                }
            };

            // Block thread until stop signal is received to keep esclient alive
            let _ = stop_rx.blocking_recv().ok();

            // Explicitly shutdown esclient to catch cleanup errors
            if let Err(err) = esclient.delete() {
                tracing::error!("Failed to delete esclient: {err}");
            }
        });

        create_client_rx
            .await
            .map_err(|_recv_err| Error::EsClientInitChannelClosed)
            .and_then(|inner_err| match inner_err {
                Some(err) => Err(err),
                None => Ok(()),
            })?;

        tokio::spawn(async move {
            while let Some(msg) = msg_rx.recv().await {
                let mut inner = cloned_states.inner.lock().await;
                inner.handle_message(msg);
            }
        });

        Ok(ProcessMonitorHandle {
            states,
            client_thread,
            stop_tx,
        })
    }

    fn create_esclient<'a>(
        msg_tx: tokio::sync::mpsc::Sender<endpoint_sec::Message>,
    ) -> Result<endpoint_sec::Client<'a>, Error> {
        use endpoint_sec::sys::es_event_type_t;

        let mut esclient = endpoint_sec::Client::new(move |_esclient, msg| {
            msg_tx.blocking_send(msg).ok();
        })
        .map_err(|err| {
            if err == endpoint_sec::sys::NewClientError::NotPermitted {
                Error::NeedFullDiskPermissions
            } else {
                Error::CreateEsClient(err)
            }
        })?;

        esclient
            .subscribe(&[
                es_event_type_t::ES_EVENT_TYPE_NOTIFY_EXEC,
                es_event_type_t::ES_EVENT_TYPE_NOTIFY_EXIT,
                es_event_type_t::ES_EVENT_TYPE_NOTIFY_FORK,
            ])
            .map_err(Error::Subscribe)?;

        Ok(esclient)
    }
}

/// Return whether the process has full-disk access
pub fn has_full_disk_access() -> bool {
    endpoint_sec::Client::new(move |_, _| {})
        .map(|_esclient| true)
        .or_else(|err| {
            if err == endpoint_sec::sys::NewClientError::NotPermitted {
                Ok(false)
            } else {
                Err(Error::CreateEsClient(err))
            }
        })
        .unwrap_or_default()
}

impl ProcessMonitorHandle {
    pub async fn shutdown(self) {
        if self.stop_tx.send(()).is_err() {
            tracing::error!("Failed to send stop signal");
        }

        let client_thread = self.client_thread;
        let _ = tokio::task::spawn_blocking(move || client_thread.join()).await;
    }

    pub fn states(&self) -> &ProcessStates {
        &self.states
    }
}

/// Controls the known exclusion states of all processes
#[derive(Debug, Clone)]
pub struct ProcessStates {
    inner: Arc<Mutex<InnerProcessStates>>,
}

/// Possible states of each process
#[derive(Debug, Clone)]
pub enum ExclusionStatus {
    /// The process should be excluded from the VPN
    Excluded,
    /// The process should not be excluded from the VPN
    Included,
    /// The process traffic is routed based on the source IP it binds to:
    /// if the source IP equals the VPN tunnel address, the packet is sent through the VPN;
    /// otherwise it is sent through the default interface.
    /// No source IP rewriting is performed.
    Hybrid,
    /// The process is unknown
    Unknown,
}

#[derive(Debug)]
struct InnerProcessStates {
    processes: HashMap<pid_t, ProcessInfo>,
    exclude_paths: HashSet<PathBuf>,
    hybrid_paths: HashSet<PathBuf>,
}

impl ProcessStates {
    /// Initialize process states
    fn new() -> Result<Self, Error> {
        let mut states = InnerProcessStates {
            processes: HashMap::new(),
            exclude_paths: HashSet::new(),
            hybrid_paths: HashSet::new(),
        };

        let processes = list_pids().map_err(Error::InitializePids)?;

        for pid in processes {
            let path = process_path(pid).map_err(|error| Error::FindProcessPath(error, pid))?;

            let responsible_pid = unsafe { responsibility_get_pid_responsible_for_pid(pid) };
            let responsible_path = if responsible_pid >= 0 {
                process_path(responsible_pid)
                    .map_err(|error| Error::FindProcessPath(error, responsible_pid))?
            } else {
                path.clone() // fallback to itself
            };

            let info = ProcessInfo::included(path, responsible_path);

            states.processes.insert(pid, info);
        }

        Ok(ProcessStates {
            inner: Arc::new(Mutex::new(states)),
        })
    }

    /// Update the set of excluded and hybrid paths.
    ///
    /// - `paths`: executables to be fully excluded from the VPN tunnel (always routed via the
    ///   default interface).
    /// - `hybrid_paths`: executables that are routed based on their source IP binding. Traffic
    ///   bound to the VPN tunnel address is sent through the VPN; all other traffic is sent via
    ///   the default interface. Hybrid status is not inherited by child processes.
    pub async fn set_paths(&self, paths: HashSet<PathBuf>, hybrid_paths: HashSet<PathBuf>) {
        let mut inner = self.inner.lock().await;

        /// Canonicalize a set of paths, resolving symlinks to align user and system paths.
        async fn canonicalize(paths: HashSet<PathBuf>) -> HashSet<PathBuf> {
            stream::iter(paths)
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
                .await
        }

        let paths = canonicalize(paths).await;
        let hybrid_paths = canonicalize(hybrid_paths).await;

        for info in inner.processes.values_mut() {
            // Recompute excluded paths: keep intersection with the new set (preserves inherited
            // exclusions that are still relevant) and add any direct matches.
            let mut new_exclude_paths: HashSet<_> = info
                .excluded_by_paths
                .intersection(&paths)
                .cloned()
                .collect();
            if paths.contains(&info.exec_path) && !new_exclude_paths.contains(&info.exec_path) {
                new_exclude_paths.insert(info.exec_path.clone());
            }
            if paths.contains(&info.responsible_exec_path)
                && !new_exclude_paths.contains(&info.responsible_exec_path)
            {
                new_exclude_paths.insert(info.responsible_exec_path.clone());
            }
            info.excluded_by_paths = new_exclude_paths;

            // Recompute hybrid paths: same intersection-then-add logic.
            // Note: hybrid status is never inherited via fork, so the intersection here only
            // retains entries that were set by a previous exec event.
            let mut new_hybrid_paths: HashSet<_> = info
                .hybrid_by_paths
                .intersection(&hybrid_paths)
                .cloned()
                .collect();
            if hybrid_paths.contains(&info.exec_path) && !new_hybrid_paths.contains(&info.exec_path)
            {
                new_hybrid_paths.insert(info.exec_path.clone());
            }
            if hybrid_paths.contains(&info.responsible_exec_path)
                && !new_hybrid_paths.contains(&info.responsible_exec_path)
            {
                new_hybrid_paths.insert(info.responsible_exec_path.clone());
            }
            info.hybrid_by_paths = new_hybrid_paths;
        }

        inner.exclude_paths = paths;
        inner.hybrid_paths = hybrid_paths;
    }

    pub async fn get_excluded_processes(&self) -> SplitTunnelExcludedProcessList {
        let inner = self.inner.lock().await;
        let mut processes = inner
            .processes
            .iter()
            .filter(|(_, info)| info.is_excluded())
            .map(|(pid, info)| SplitTunnelExcludedProcess {
                pid: *pid,
                exec_path: info.exec_path.clone(),
                responsible_exec_path: info.responsible_exec_path.clone(),
            })
            .collect::<Vec<_>>();
        drop(inner);
        processes.sort_by(sort_process_list);

        SplitTunnelExcludedProcessList { processes }
    }

    pub async fn get_process_status(&self, pid: pid_t) -> ExclusionStatus {
        let inner = self.inner.lock().await;
        match inner.processes.get(&pid) {
            Some(val) if val.is_excluded() => ExclusionStatus::Excluded,
            Some(val) if val.is_hybrid() => ExclusionStatus::Hybrid,
            Some(_) => ExclusionStatus::Included,
            None => ExclusionStatus::Unknown,
        }
    }
}

trait AuditTokenExt {
    fn checked_pid(&self) -> Option<pid_t>;
}

impl AuditTokenExt for endpoint_sec::AuditToken {
    /// Check that `pid` is positive and return it.
    fn checked_pid(&self) -> Option<pid_t> {
        let pid = self.pid();
        (pid > 0).then_some(pid)
    }
}

impl InnerProcessStates {
    fn handle_message(&mut self, msg: endpoint_sec::Message) {
        let Some(pid) = msg.process().audit_token().checked_pid() else {
            tracing::trace!("esclient returned bad pid: {msg:?}");
            return;
        };

        // The PID returned by `responsible_audit_token()` can point at:
        //
        // 1. Process running in different process group that triggered the launch of the app represented by `pid`
        //    This is the case with XPC/WebKit, apps like DuckDuckGo that use system framework for sandboxed web browsing
        // 2. Parent process
        // 3. Point at itself (the value held in `pid`)

        let evt = msg.event();
        match evt {
            Some(endpoint_sec::Event::NotifyFork(evt)) => {
                let executable_path = PathBuf::from(msg.process().executable().path());
                let Some(responsible_pid) = msg
                    .process()
                    .responsible_audit_token()
                    .and_then(|v| v.checked_pid())
                else {
                    tracing::trace!("bad responsible pid: {msg:?}");
                    return;
                };

                self.handle_fork(pid, responsible_pid, executable_path, evt)
            }
            Some(endpoint_sec::Event::NotifyExec(evt)) => {
                let Some(responsible_pid) = msg
                    .process()
                    .responsible_audit_token()
                    .and_then(|v| v.checked_pid())
                else {
                    tracing::trace!("bad responsible pid: {msg:?}");
                    return;
                };

                self.handle_exec(pid, responsible_pid, evt)
            }
            Some(endpoint_sec::Event::NotifyExit(_evt)) => self.handle_exit(pid),
            _ => {}
        }
    }

    // For new processes, inherit all exclusion state from the parent, if there is one.
    // Otherwise, look up excluded paths
    fn handle_fork(
        &mut self,
        parent_pid: pid_t,
        responsible_pid: pid_t,
        exec_path: PathBuf,
        msg: endpoint_sec::EventFork,
    ) {
        let Some(pid) = msg.child().audit_token().checked_pid() else {
            tracing::trace!("esclient returned bad pid: {msg:?}");
            return;
        };

        if self.processes.contains_key(&pid) {
            tracing::error!("Conflicting pid! State already contains {pid}");
        }

        // Inherit exclusion status from parent, but NOT hybrid status.
        // On Windows, ST_PROCESS_SPLIT_STATUS_HYBRID_BY_CONFIG is similarly never propagated
        // to children; only the fully-excluded status is inherited.
        let base_info = match self.processes.get(&parent_pid) {
            Some(parent_info) => {
                let mut cloned_info = parent_info.to_owned();
                // Clear hybrid status — child processes must be configured explicitly
                cloned_info.hybrid_by_paths.clear();
                if let Some(resp_info) = self.processes.get(&responsible_pid) {
                    cloned_info.responsible_exec_path = resp_info.exec_path.clone();
                }
                cloned_info
            }
            None => {
                tracing::error!("{pid}: Unknown parent pid {parent_pid}!");
                let responsible_path = self
                    .processes
                    .get(&responsible_pid)
                    .map(|resp_info| resp_info.exec_path.clone())
                    .unwrap_or_else(|| exec_path.clone());

                ProcessInfo::included(exec_path, responsible_path)
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

    fn handle_exec(&mut self, pid: pid_t, responsible_pid: pid_t, msg: endpoint_sec::EventExec) {
        // Get app responsible for executing `pid`
        let responsible_exec_path = self
            .processes
            .get(&responsible_pid)
            .map(|resp_info| resp_info.exec_path.clone());

        let Some(info) = self.processes.get_mut(&pid) else {
            tracing::error!("exec received for unknown pid {pid}");
            return;
        };
        if msg.target().executable().path_truncated() {
            tracing::error!(
                "Ignoring process {pid} with truncated path: {}",
                msg.target().executable().path().to_string_lossy()
            );
            return;
        }

        info.exec_path = PathBuf::from(msg.target().executable().path());
        info.responsible_exec_path =
            responsible_exec_path.unwrap_or_else(|| info.exec_path.clone());

        // Check if process is excluded directly by exec path
        if !info.excluded_by_paths.contains(&info.exec_path)
            && self.exclude_paths.contains(&info.exec_path)
        {
            info.excluded_by_paths.insert(info.exec_path.clone());
            tracing::trace!("Excluding {pid} by path: {}", info.exec_path.display());
        }

        // Check if process is excluded indirectly by exec path of process responsible for spawning this process
        if !info.excluded_by_paths.contains(&info.responsible_exec_path)
            && self.exclude_paths.contains(&info.responsible_exec_path)
        {
            info.excluded_by_paths
                .insert(info.responsible_exec_path.clone());
            tracing::trace!(
                "Excluding {pid} by responsible path: {}",
                info.responsible_exec_path.display()
            );
        }

        // Check if process should be treated as hybrid by exec path
        if !info.hybrid_by_paths.contains(&info.exec_path)
            && self.hybrid_paths.contains(&info.exec_path)
        {
            info.hybrid_by_paths.insert(info.exec_path.clone());
            tracing::trace!(
                "Hybrid routing for {pid} by path: {}",
                info.exec_path.display()
            );
        }

        // Check if process should be treated as hybrid via the responsible exec path
        if !info.hybrid_by_paths.contains(&info.responsible_exec_path)
            && self.hybrid_paths.contains(&info.responsible_exec_path)
        {
            info.hybrid_by_paths
                .insert(info.responsible_exec_path.clone());
            tracing::trace!(
                "Hybrid routing for {pid} by responsible path: {}",
                info.responsible_exec_path.display()
            );
        }
    }

    fn handle_exit(&mut self, pid: pid_t) {
        if self.processes.remove(&pid).is_none() {
            tracing::error!("exit syscall for unknown pid {pid}");
        }
    }
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    exec_path: PathBuf,
    // Path of executable responsible for launching this process
    responsible_exec_path: PathBuf,
    excluded_by_paths: HashSet<PathBuf>,
    /// Paths in `hybrid_paths` that caused this process to be treated as hybrid.
    /// Unlike `excluded_by_paths`, this is never inherited by forked children.
    hybrid_by_paths: HashSet<PathBuf>,
}

impl ProcessInfo {
    fn included(exec_path: PathBuf, responsible_path: PathBuf) -> Self {
        ProcessInfo {
            exec_path,
            responsible_exec_path: responsible_path,
            excluded_by_paths: HashSet::new(),
            hybrid_by_paths: HashSet::new(),
        }
    }

    fn is_excluded(&self) -> bool {
        !self.excluded_by_paths.is_empty()
    }

    /// Returns true when this process should be given hybrid routing.
    /// Excluded status takes precedence: a process in both lists is treated as fully excluded.
    fn is_hybrid(&self) -> bool {
        !self.hybrid_by_paths.is_empty() && self.excluded_by_paths.is_empty()
    }
}

/// Sort process list grouping by `responsible_exec_path`
fn sort_process_list(a: &SplitTunnelExcludedProcess, b: &SplitTunnelExcludedProcess) -> Ordering {
    a.responsible_exec_path
        .cmp(&b.responsible_exec_path)
        .then_with(|| {
            if a.exec_path == a.responsible_exec_path {
                Ordering::Less
            } else if b.exec_path == b.responsible_exec_path {
                Ordering::Greater
            } else {
                a.exec_path.cmp(&b.exec_path)
            }
        })
        .then_with(|| a.pid.cmp(&b.pid))
}

/// Initialize endpoint-sec crate just once.
fn init_endpoint_sec() {
    ENDPOINT_SEC_INIT.call_once(|| {
        // Pass current macOS version to endpoint-sec to ensure that runtime availability checks function properly
        let os_ver = AppleVersion::current();

        endpoint_sec::version::set_runtime_version(
            u64::from(os_ver.major_version()),
            u64::from(os_ver.minor_version()),
            u64::from(os_ver.patch_version()),
        );
    });
}

/// Check whether the current macOS version is supported, and return an error otherwise
fn check_os_version_support() -> Result<(), Error> {
    check_os_version_support_inner(AppleVersion::current())
}

fn check_os_version_support_inner(version: AppleVersion) -> Result<(), Error> {
    if version >= *MIN_OS_VERSION {
        Ok(())
    } else {
        Err(Error::UnsupportedMacosVersion { actual: version })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use nym_platform_metadata::AppleVersion;

    #[test]
    fn test_min_os_version() {
        assert!(check_os_version_support_inner(MIN_OS_VERSION.clone()).is_ok());

        // test unsupported version
        assert!(check_os_version_support_inner(AppleVersion::from_str("10.7").unwrap()).is_err());

        // test supported version
        assert!(check_os_version_support_inner(AppleVersion::from_str("11.0").unwrap()).is_ok());
    }

    #[test]
    fn test_sort_process_list() {
        use pretty_assertions::assert_eq;

        let mut processes = vec![
            SplitTunnelExcludedProcess {
                pid: 1,
                exec_path: PathBuf::from("/usr/bin/curl"),
                responsible_exec_path: PathBuf::from("/usr/bin/curl"),
            },
            SplitTunnelExcludedProcess {
                pid: 3,
                exec_path: PathBuf::from("/usr/bin/git"),
                responsible_exec_path: PathBuf::from("/usr/bin/nym-vpnd"),
            },
            SplitTunnelExcludedProcess {
                pid: 4,
                exec_path: PathBuf::from("/usr/bin/curl"),
                responsible_exec_path: PathBuf::from("/usr/bin/nym-vpnd"),
            },
            SplitTunnelExcludedProcess {
                pid: 2,
                exec_path: PathBuf::from("/usr/bin/nym-vpnd"),
                responsible_exec_path: PathBuf::from("/usr/bin/nym-vpnd"),
            },
            SplitTunnelExcludedProcess {
                pid: 5,
                exec_path: PathBuf::from("/usr/bin/nym-vpnc"),
                responsible_exec_path: PathBuf::from("/usr/bin/nym-vpnc"),
            },
            SplitTunnelExcludedProcess {
                pid: 6,
                exec_path: PathBuf::from("/usr/bin/base64"),
                responsible_exec_path: PathBuf::from("/usr/bin/nym-vpnc"),
            },
        ];
        processes.sort_by(sort_process_list);

        assert_eq!(
            vec![
                SplitTunnelExcludedProcess {
                    pid: 1,
                    exec_path: PathBuf::from("/usr/bin/curl"),
                    responsible_exec_path: PathBuf::from("/usr/bin/curl")
                },
                SplitTunnelExcludedProcess {
                    pid: 5,
                    exec_path: PathBuf::from("/usr/bin/nym-vpnc"),
                    responsible_exec_path: PathBuf::from("/usr/bin/nym-vpnc"),
                },
                SplitTunnelExcludedProcess {
                    pid: 6,
                    exec_path: PathBuf::from("/usr/bin/base64"),
                    responsible_exec_path: PathBuf::from("/usr/bin/nym-vpnc"),
                },
                SplitTunnelExcludedProcess {
                    pid: 2,
                    exec_path: PathBuf::from("/usr/bin/nym-vpnd"),
                    responsible_exec_path: PathBuf::from("/usr/bin/nym-vpnd")
                },
                SplitTunnelExcludedProcess {
                    pid: 4,
                    exec_path: PathBuf::from("/usr/bin/curl"),
                    responsible_exec_path: PathBuf::from("/usr/bin/nym-vpnd")
                },
                SplitTunnelExcludedProcess {
                    pid: 3,
                    exec_path: PathBuf::from("/usr/bin/git"),
                    responsible_exec_path: PathBuf::from("/usr/bin/nym-vpnd")
                },
            ],
            processes,
        );
    }
}

unsafe extern "C" {
    unsafe fn responsibility_get_pid_responsible_for_pid(pid: libc::pid_t) -> libc::pid_t;
}
