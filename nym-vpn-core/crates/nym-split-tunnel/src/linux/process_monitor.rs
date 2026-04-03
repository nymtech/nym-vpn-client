// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Process monitor implemented using netlink process connector API.

use std::{
    collections::{HashMap, HashSet},
    os::fd::AsRawFd,
    path::PathBuf,
    sync::LazyLock,
};

use libc::{
    CN_IDX_PROC, CN_VAL_PROC, NETLINK_CONNECTOR, NLMSG_DONE, PROC_CN_MCAST_LISTEN, PROC_EVENT_EXEC,
    PROC_EVENT_EXIT, PROC_EVENT_FORK, PROC_EVENT_NONE, pid_t, proc_cn_mcast_op,
};
use netlink_sys::{AsyncSocket, AsyncSocketExt, TokioSocket};
use pidfd_util::{PidFd, PidFdExt};
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use super::bindings::{
    nlcn_event_msg, nlcn_subscribe_msg, proc_event__bindgen_ty_1_exec_proc_event as ExecEvt,
    proc_event__bindgen_ty_1_exit_proc_event as ExitEvt,
    proc_event__bindgen_ty_1_fork_proc_event as ForkEvt,
};

/// Check and cache whether kernel has proc events enabled
static KERNEL_SUPPORTS_PROC_EVENTS: LazyLock<bool> =
    LazyLock::new(|| match procfs::kernel_config() {
        Ok(config) => {
            let proc_events = config.get("CONFIG_PROC_EVENTS");
            let is_enabled = proc_events == Some(&procfs::ConfigSetting::Yes);

            tracing::info!("CONFIG_PROC_EVENTS is set to {proc_events:?}");
            is_enabled
        }
        Err(err) => {
            tracing::warn!(
                "Failed to read kernel config. Consider split-tunnel unavailable: {err}"
            );
            false
        }
    });

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Proc error")]
    Proc(#[from] procfs::ProcError),

    #[error("Proc events is not supported")]
    Unsupported,
}

#[derive(Debug)]
pub struct ProcessSnapshot {
    pub pid: pid_t,
    pub parent_pid: pid_t,
    pub path: Option<PathBuf>,
}

#[derive(Debug)]
pub enum ProcessEvent {
    Subscribed {
        processes: Vec<ProcessSnapshot>,
    },
    Fork {
        parent_pid: pid_t,
        child_pid: pid_t,
        path: PathBuf,
    },
    Exec {
        pid: pid_t,
        path: PathBuf,
    },
    Exit {
        pid: pid_t,
    },
}

pub struct ProcessMonitor {
    nl_sock: TokioSocket,
    threads: HashMap<pid_t, HashSet<pid_t>>,
}

impl ProcessMonitor {
    pub async fn spawn(
        tx: UnboundedSender<ProcessEvent>,
        shutdown_token: CancellationToken,
    ) -> Result<JoinHandle<Result<(), Error>>, Error> {
        if !*KERNEL_SUPPORTS_PROC_EVENTS {
            return Err(Error::Unsupported);
        }

        let pid = unsafe { libc::getpid() };
        let sockaddr = netlink_sys::SocketAddr::new(pid as u32, CN_IDX_PROC);
        let mut nl_sock = TokioSocket::new(NETLINK_CONNECTOR as _)?;
        nl_sock.socket_mut().bind(&sockaddr)?;

        let proc_monitor = Self {
            nl_sock,
            threads: HashMap::new(),
        };
        proc_monitor.subscribe().await?;

        Ok(tokio::spawn(proc_monitor.run(tx, shutdown_token)))
    }

    async fn subscribe(&self) -> std::io::Result<()> {
        let mut msg: nlcn_subscribe_msg = unsafe { std::mem::zeroed() };
        msg.nl_hdr.nlmsg_len = std::mem::size_of_val(&msg) as u32;
        msg.nl_hdr.nlmsg_pid = unsafe { libc::getpid() as u32 };
        msg.nl_hdr.nlmsg_type = NLMSG_DONE as u16;

        msg.payload.cn_msg.id.idx = CN_IDX_PROC;
        msg.payload.cn_msg.id.val = CN_VAL_PROC;
        msg.payload.cn_msg.len = std::mem::size_of::<proc_cn_mcast_op>() as u16;
        msg.payload.cn_mcast = PROC_CN_MCAST_LISTEN;

        let bytes = unsafe {
            std::slice::from_raw_parts(&msg as *const _ as _, std::mem::size_of_val(&msg))
        };
        self.nl_sock.send(bytes).await?;

        Ok(())
    }

    async fn run(
        mut self,
        tx: UnboundedSender<ProcessEvent>,
        shutdown_token: CancellationToken,
    ) -> Result<(), Error> {
        const MSG_LEN: usize = std::mem::size_of::<nlcn_event_msg>();

        let mut buf = bytes::BytesMut::with_capacity(MSG_LEN);

        loop {
            tokio::select! {
                res = self.nl_sock.recv(&mut buf) => {
                    match res {
                        Ok(()) => {
                            while buf.len() >= MSG_LEN {
                                let msg_buf = buf.split_to(MSG_LEN);
                                let msg = msg_buf.as_ptr() as *const nlcn_event_msg;
                                let msg = unsafe { &*msg };

                                if let Some(evt) = self.handle_event(msg).await? && tx.send(evt).is_err() {
                                    tracing::trace!("Exiting since event channel is closed.");
                                    return Ok(());
                                }
                            }
                        }
                        Err(err) => {
                            if err.raw_os_error() == Some(libc::EINTR) {
                                continue;
                            } else {
                                tracing::error!("Exiting due to failure to read socket: {err}");
                                return Err(Error::Io(err));
                            }
                        }
                    }
                },
                _ = shutdown_token.cancelled() => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_event(&mut self, msg: &nlcn_event_msg) -> Result<Option<ProcessEvent>, Error> {
        // Ensure the message kind is correct
        if msg.payload.cn_msg.id.idx != CN_IDX_PROC && msg.payload.cn_msg.id.val != CN_VAL_PROC {
            tracing::warn!("idx,val is not CN_IDX_*");
            return Ok(None);
        }

        // Only process messages from kernel
        if msg.nl_hdr.nlmsg_pid != 0 {
            tracing::warn!(
                "Ignore message from non-kernel process: {}",
                msg.nl_hdr.nlmsg_pid
            );
            return Ok(None);
        }

        Ok(match msg.payload.proc_ev.what {
            PROC_EVENT_NONE => {
                tracing::trace!("Subscribed");
                self.handle_subscribed()?
            }
            PROC_EVENT_FORK => {
                let fork = unsafe { msg.payload.proc_ev.event_data.fork };
                self.handle_fork(fork).await
            }
            PROC_EVENT_EXEC => {
                let exec = unsafe { msg.payload.proc_ev.event_data.exec };
                self.handle_exec(exec).await
            }
            PROC_EVENT_EXIT => {
                let exit = unsafe { msg.payload.proc_ev.event_data.exit };
                self.handle_exit(exit)
            }
            _ => None,
        })
    }

    async fn handle_fork(&mut self, event: ForkEvt) -> Option<ProcessEvent> {
        let threads = self.threads.entry(event.child_tgid).or_default();
        threads.insert(event.child_pid);

        // Main thread started
        if event.child_pid == event.child_tgid {
            let exe_path = query_exec_path("fork", event.child_tgid)
                .await
                .unwrap_or_default();

            tracing::trace!(
                "fork: {} (parent: {}) exe={}",
                event.child_tgid,
                event.parent_tgid,
                exe_path.display()
            );

            Some(ProcessEvent::Fork {
                parent_pid: event.parent_tgid,
                child_pid: event.child_pid,
                path: exe_path,
            })
        } else {
            // Subordinate thread spawned
            None
        }
    }

    async fn handle_exec(&mut self, event: ExecEvt) -> Option<ProcessEvent> {
        // Both ids are always equal
        if event.process_pid == event.process_tgid {
            let exe_path = query_exec_path("exec", event.process_pid)
                .await
                .unwrap_or_default();
            tracing::trace!("exec: {} {}", event.process_pid, exe_path.display());

            Some(ProcessEvent::Exec {
                pid: event.process_pid,
                path: exe_path,
            })
        } else {
            None
        }
    }

    fn handle_exit(&mut self, event: ExitEvt) -> Option<ProcessEvent> {
        let Some(threads) = self.threads.get_mut(&event.process_tgid) else {
            tracing::warn!("Exit for pid {} with no known threads!", event.process_tgid);
            return None;
        };

        threads.remove(&event.process_pid);

        // Send exit events only when all threads exit
        if threads.is_empty() {
            self.threads.remove(&event.process_tgid);

            tracing::trace!("exit: {}", event.process_tgid);
            Some(ProcessEvent::Exit {
                pid: event.process_tgid,
            })
        } else {
            None
        }
    }

    fn handle_subscribed(&mut self) -> Result<Option<ProcessEvent>, Error> {
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

        let mut proc_snapshots = Vec::new();

        for proc in proc_iter {
            let pid = proc.pid();
            let parent_pid = match proc.status() {
                Ok(status) => status.ppid,
                Err(procfs::ProcError::NotFound(_)) => {
                    // process vanished
                    continue;
                }
                Err(err) => {
                    tracing::error!("Can't obtain process status {pid}: {err}");
                    return Err(Error::Proc(err));
                }
            };

            // Some system processes do not adverise their executable path, ex: /proc/2/exe (kthreadd)
            let path = proc
                .exe()
                .inspect_err(|err| {
                    if !matches!(err, procfs::ProcError::NotFound(_)) {
                        tracing::trace!("couldn't read exe() for pid {}: {}", proc.pid(), err);
                    }
                })
                .ok();

            proc_snapshots.push(ProcessSnapshot {
                pid,
                parent_pid,
                path,
            });

            match proc.tasks() {
                Ok(task_iter) => {
                    let tids = task_iter
                        .into_iter()
                        .filter_map(|task_result| {
                            match task_result {
                                Ok(task) => Some(task.tid),
                                Err(procfs::ProcError::NotFound(_)) => {
                                    // task vanished
                                    None
                                }
                                Err(err) => {
                                    tracing::error!("Failed to list tasks for {pid}: {err}");
                                    None
                                }
                            }
                        })
                        .collect::<HashSet<pid_t>>();

                    if !tids.is_empty() {
                        let threads = self.threads.entry(pid).or_default();
                        threads.extend(tids);
                    }
                }
                Err(procfs::ProcError::NotFound(_)) => {
                    // process vanished
                }
                Err(procfs::ProcError::Io(err, path)) => {
                    tracing::trace!("io error when querying tasks {path:?}: {err}");
                }
                Err(err) => {
                    tracing::error!("can't read process: {err}");
                }
            }
        }

        Ok(Some(ProcessEvent::Subscribed {
            processes: proc_snapshots,
        }))
    }
}

async fn query_exec_path(event_type: &'static str, proc_pid: pid_t) -> Option<PathBuf> {
    // Open pidfd to ensure that no pid recycling happened after obtaining path to executable
    let pid_fd = PidFd::from_pid(proc_pid)
        .inspect_err(|err| {
            // Do not print "process not found" errors. Assume that the process was very short lived and we can ignore it.
            if err.raw_os_error() != Some(libc::ESRCH) {
                tracing::error!(?event_type, ?proc_pid, "failed to obtain pidfd: {err}")
            }
        })
        .ok()?;

    // Obtain path to executable
    let exe_path = procfs::process::Process::new(proc_pid)
        .and_then(|proc| proc.exe())
        .inspect_err(|err| {
            // Ignore "no such file" errors, which could indicate that the process is short lived.
            if !matches!(err, procfs::ProcError::NotFound(_)) {
                tracing::error!(?event_type, ?proc_pid, "failed to obtain proc exe: {err}")
            }
        })
        .ok()?;

    // Check whether process quit while reading executable path
    let is_process_exited = check_pidfd_exited(&pid_fd)
        .inspect_err(|err| {
            tracing::error!("failed to poll pid_fd: {err}");
        })
        .ok()?;

    if is_process_exited {
        tracing::trace!(
            ?event_type,
            "process with pid {} is already terminated",
            proc_pid
        );
        None
    } else {
        Some(exe_path)
    }
}

fn check_pidfd_exited(pid_fd: &PidFd) -> std::io::Result<bool> {
    let mut pollfd = libc::pollfd {
        fd: pid_fd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };

    let ret = unsafe { libc::poll(&mut pollfd, 1, 0) };

    if ret == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        // ret > 0 with POLLIN means the pidfd is readable → process exited
        // ret == 0 means nothing ready → process still alive
        Ok(ret > 0 && (pollfd.revents & libc::POLLIN) != 0)
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use pidfd_util::{PidFd, PidFdExt};

    use super::*;

    #[test]
    fn test_pidfd_exited() {
        let child = Command::new("sleep").arg("0.5").spawn().unwrap();
        let pid = child.id();
        let pidfd = PidFd::from_pid(pid as _).unwrap();

        assert!(!check_pidfd_exited(&pidfd).expect("process must still run"));

        std::thread::sleep(std::time::Duration::from_secs(1));

        assert!(check_pidfd_exited(&pidfd).expect("process must be gone"));
    }
}
