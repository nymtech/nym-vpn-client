// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use libc::{
    CN_IDX_PROC, CN_VAL_PROC, NETLINK_CONNECTOR, NLMSG_DONE, PROC_CN_MCAST_IGNORE,
    PROC_CN_MCAST_LISTEN, PROC_EVENT_EXEC, PROC_EVENT_EXIT, PROC_EVENT_FORK, PROC_EVENT_NONE,
    getpid, pid_t, proc_cn_mcast_op,
};
use netlink_sys::{AsyncSocket, AsyncSocketExt, TokioSocket};
use pidfd_util::{PidFd, PidFdExt};
use std::{os::fd::AsRawFd, path::PathBuf};
use tokio::{fs::read_link, sync::mpsc::UnboundedSender, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use super::bindings::{
    nlcn_event_msg, nlcn_subscribe_msg, proc_event__bindgen_ty_1_exec_proc_event as ExecEvt,
    proc_event__bindgen_ty_1_exit_proc_event as ExitEvt,
    proc_event__bindgen_ty_1_fork_proc_event as ForkEvt,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Proc error")]
    Proc(#[from] procfs::ProcError),
}

#[derive(Debug, Clone)]
pub enum ProcessEvent {
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

pub struct ProcessEventStream {
    nl_sock: TokioSocket,
}

impl ProcessEventStream {
    pub async fn spawn(
        tx: UnboundedSender<ProcessEvent>,
        shutdown_token: CancellationToken,
    ) -> Result<JoinHandle<()>, Error> {
        let pid = unsafe { getpid() };
        let sockaddr = netlink_sys::SocketAddr::new(pid as u32, CN_IDX_PROC);
        let mut nl_sock = TokioSocket::new(NETLINK_CONNECTOR as _)?;
        nl_sock.socket_mut().bind(&sockaddr)?;

        let proc_monitor = Self { nl_sock };
        proc_monitor.subscribe(true).await?;

        Ok(tokio::spawn(proc_monitor.run(tx, shutdown_token)))
    }

    async fn subscribe(&self, enable: bool) -> std::io::Result<()> {
        let mut msg: nlcn_subscribe_msg = unsafe { std::mem::zeroed() };
        msg.nl_hdr.nlmsg_len = std::mem::size_of_val(&msg) as u32;
        msg.nl_hdr.nlmsg_pid = unsafe { getpid() as u32 };
        msg.nl_hdr.nlmsg_type = NLMSG_DONE as u16;

        msg.payload.cn_msg.id.idx = CN_IDX_PROC;
        msg.payload.cn_msg.id.val = CN_VAL_PROC;
        msg.payload.cn_msg.len = std::mem::size_of::<proc_cn_mcast_op>() as u16;
        msg.payload.cn_mcast = if enable {
            PROC_CN_MCAST_LISTEN
        } else {
            PROC_CN_MCAST_IGNORE
        };

        let bytes = unsafe {
            std::slice::from_raw_parts(&msg as *const _ as _, std::mem::size_of_val(&msg))
        };
        self.nl_sock.send(bytes).await?;

        Ok(())
    }

    async fn run(mut self, tx: UnboundedSender<ProcessEvent>, shutdown_token: CancellationToken) {
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

                                if let Some(evt) = self.handle_event(msg).await && tx.send(evt).is_err() {
                                    tracing::trace!("Exiting since event channel is closed.");
                                    return;
                                }
                            }
                        }
                        Err(err) => {
                            if err.raw_os_error() == Some(libc::EINTR) {
                                continue;
                            } else {
                                tracing::error!("failed to read socket: {err}");
                                // todo: report error
                                return;
                            }
                        }
                    }
                },
                _ = shutdown_token.cancelled() => {
                    break;
                }
            }
        }
    }

    async fn handle_event(&mut self, msg: &nlcn_event_msg) -> Option<ProcessEvent> {
        // Ensure the message kind is correct
        if msg.payload.cn_msg.id.idx != CN_IDX_PROC && msg.payload.cn_msg.id.val != CN_VAL_PROC {
            tracing::warn!("idx,val is not CN_IDX_*");
            return None;
        }

        // Only process messages from kernel
        if msg.nl_hdr.nlmsg_pid != 0 {
            tracing::warn!(
                "Ignore message from non-kernel process: {}",
                msg.nl_hdr.nlmsg_pid
            );
            return None;
        }

        match msg.payload.proc_ev.what {
            PROC_EVENT_NONE => {
                tracing::trace!("set mcast listen ok");
                None
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
        }
    }

    async fn handle_fork(&mut self, event: ForkEvt) -> Option<ProcessEvent> {
        if event.child_pid == event.child_tgid {
            let exe_path = query_exec_path("fork", event.child_pid).await.unwrap_or_default();

            tracing::trace!(
                "fork: {} (parent: {}) exe={}",
                event.child_pid,
                event.parent_tgid,
                exe_path.display()
            );

            Some(ProcessEvent::Fork {
                parent_pid: event.parent_tgid,
                child_pid: event.child_pid,
                path: exe_path,
            })
        } else {
            None
        }
    }

    async fn handle_exec(&mut self, event: ExecEvt) -> Option<ProcessEvent> {
        if event.process_pid == event.process_tgid {
            let exe_path = query_exec_path("exec", event.process_pid).await.unwrap_or_default();
            tracing::trace!(
                "exec: {} {}",
                event.process_pid,
                exe_path.display()
            );

            Some(ProcessEvent::Exec {
                pid: event.process_pid,
                path: exe_path,
            })
        } else {
            // tracing::trace!(
            //     "spawn thread: {} {}",
            //     event.process_pid,
            //     event.process_tgid
            // );
            None
        }
    }

    fn handle_exit(&mut self, event: ExitEvt) -> Option<ProcessEvent> {
        // Ignore thread exits
        if event.process_pid == event.process_tgid {
            tracing::trace!(
                "exit: {}",
                event.process_pid
            );
            Some(ProcessEvent::Exit {
                pid: event.process_pid,
            })
        } else {
            // tracing::trace!(
            //     "exit thread: {} (tgid: {})",
            //     event.process_pid,
            //     event.process_tgid,
            // );
            None
        }
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
    let exe_path = procfs::process::Process::new(proc_pid).and_then(|proc| proc.exe())
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
