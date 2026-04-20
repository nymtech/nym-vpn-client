// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{env, path::PathBuf, process::Stdio, time::Duration};

use nym_socks5_proxy_ipc::{DaemonMessage, InterfaceAddresses, ProxyConfig, ProxyMessage};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, Command},
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

const PROXY_BINARY_NAME: &str = if cfg!(windows) {
    "nym-socks5-proxy.exe"
} else {
    "nym-socks5-proxy"
};

pub(super) type Result<T> = std::result::Result<T, Socks5ProcessError>;

#[derive(Clone)]
pub struct Socks5ProcessHandle {
    msg_tx: mpsc::UnboundedSender<DaemonMessage>,
    shutdown_token: CancellationToken,
}

impl Socks5ProcessHandle {
    pub fn set_tunnel_addrs(&self, tunnel_addrs: InterfaceAddresses) {
        let msg = DaemonMessage::SetTunnelAddresses(tunnel_addrs);
        if self.msg_tx.send(msg).is_err() {
            tracing::warn!("could not send SetTunnnelAddresses to proxy: channel closed");
        }
    }

    pub fn shutdown(&self) {
        self.shutdown_token.cancel();
    }
}

pub struct Socks5ProcessTask;

impl Socks5ProcessTask {
    pub async fn spawn(
        config: ProxyConfig,
        event_tx: mpsc::UnboundedSender<Socks5ProcessEvent>,
        shutdown_token: CancellationToken,
    ) -> Result<(Socks5ProcessHandle, JoinHandle<()>)> {
        let binary_path = find_proxy_binary()?;

        tracing::info!(
            binary = %binary_path.display(),
            listen_port = config.listen_port,
            "Spawning nym-socks5-proxy",
        );

        let mut command = Command::new(&binary_path);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // Proxy logs go to its own log file; discard stderr.
            .stderr(Stdio::null());
        // Run in own process group on Unix so it doesn't inherit a terminal.
        #[cfg(unix)]
        command.process_group(0);

        let mut child = command.spawn().map_err(Socks5ProcessError::Spawn)?;

        let stdin = child
            .stdin
            .take()
            .expect("stdin was piped but is unavailable");
        let stdout = child
            .stdout
            .take()
            .expect("stdout was piped but is unavailable");

        // Channel for sending DaemonMessages from handle → writer task → child stdin.
        let (msg_tx, msg_rx) = mpsc::unbounded_channel::<DaemonMessage>();

        // Send the initial Configure immediately.
        msg_tx
            .send(DaemonMessage::Configure(config.clone()))
            .expect("channel is open");

        // Channel to receive the ready signal from the supervisor.
        let (ready_tx, ready_rx) = oneshot::channel::<std::result::Result<(), String>>();

        // Spawn the supervisor (reads stdout, tracks process exit).
        let supervisor_token = shutdown_token.clone();
        let join_handle = tokio::spawn(supervisor(
            child,
            stdin,
            msg_rx,
            stdout,
            ready_tx,
            event_tx,
            supervisor_token.clone(),
        ));

        // Wait for the proxy to become ready.
        match ready_rx.await {
            Ok(Ok(())) => {}
            Ok(Err(msg)) => return Err(Socks5ProcessError::ProxyError(msg)),
            Err(_) => return Err(Socks5ProcessError::ExitedBeforeReady),
        }

        tracing::info!("nym-socks5-proxy is ready");

        let handle = Socks5ProcessHandle {
            msg_tx,
            shutdown_token: supervisor_token,
        };

        Ok((handle, join_handle))
    }
}

#[derive(Debug)]
pub enum Socks5ProcessEvent {
    Ready,
    StatusUpdate { active_connections: u32 },
    Error { message: String },
    Exited { success: bool },
}

pub fn find_proxy_binary() -> Result<PathBuf> {
    let exe = env::current_exe().map_err(|e| {
        Socks5ProcessError::BinaryNotFound(format!(
            "Could not determine current executable path: {e}"
        ))
    })?;
    let dir = exe.parent().ok_or_else(|| {
        Socks5ProcessError::BinaryNotFound("Current executable has no parent directory".to_string())
    })?;
    let candidate = dir.join(PROXY_BINARY_NAME);
    if candidate.exists() {
        return Ok(candidate);
    }
    Err(Socks5ProcessError::BinaryNotFound(format!(
        "Expected to find '{}' at '{}' but it was not found",
        PROXY_BINARY_NAME,
        candidate.display(),
    )))
}

async fn stdin_writer(
    mut stdin: tokio::process::ChildStdin,
    mut msg_rx: mpsc::UnboundedReceiver<DaemonMessage>,
    shutdown_token: CancellationToken,
) {
    loop {
        tokio::select! {
            msg = msg_rx.recv() => {
                match msg {
                    Some(m) => {
                        let line = format!("{m}");
                        if let Err(e) = stdin.write_all(line.as_bytes()).await {
                            tracing::warn!("failed to write to nym-socks5-proxy stdin: {e}");
                            break;
                        }
                        if let Err(e) = stdin.write_all(b"\n").await {
                            tracing::warn!("failed to write newline to nym-socks5-proxy stdin: {e}");
                            break;
                        }
                        if let Err(e) = stdin.flush().await {
                            tracing::warn!("failed to flush nym-socks5-proxy stdin: {e}");
                            break;
                        }
                    }
                    None => {
                        // All senders dropped — nothing more to send.
                        break;
                    }
                }
            }
            _ = shutdown_token.cancelled() => {
                tracing::debug!("stdin writer: shutdown requested, sending Terminate then closing stdin pipe");
                // Best-effort: ask the proxy to stop cleanly before EOF.
                let terminate = format!("{}", DaemonMessage::Terminate);
                let _ = stdin.write_all(terminate.as_bytes()).await;
                let _ = stdin.write_all(b"\n").await;
                let _ = stdin.flush().await;
                break;
            }
        }
    }
    // Drop stdin → proxy sees EOF on its stdin → proxy exits.
}

async fn supervisor(
    mut child: Child,
    stdin: tokio::process::ChildStdin,
    msg_rx: mpsc::UnboundedReceiver<DaemonMessage>,
    stdout: tokio::process::ChildStdout,
    ready_tx: oneshot::Sender<std::result::Result<(), String>>,
    event_tx: mpsc::UnboundedSender<Socks5ProcessEvent>,
    shutdown_token: CancellationToken,
) {
    tokio::spawn(stdin_writer(stdin, msg_rx, shutdown_token.clone()));

    let mut lines = BufReader::new(stdout).lines();
    let mut ready_tx = Some(ready_tx);

    loop {
        tokio::select! {
            result = lines.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        handle_proxy_line(&line, &mut ready_tx, &event_tx);
                    }
                    Ok(None) => {
                        tracing::debug!("nym-socks5-proxy stdout closed");
                        break;
                    }
                    Err(err) => {
                        tracing::error!("error reading nym-socks5-proxy stdout: {err}");
                        break;
                    }
                }
            }
            _ = shutdown_token.cancelled() => {
                tracing::debug!("nym-socks5-proxy: shutdown requested, waiting for child");
                break;
            }
        }
    }

    let success = match tokio::time::timeout(Duration::from_secs(5), child.wait()).await {
        Ok(Ok(status)) => {
            tracing::info!("nym-socks5-proxy exited with status {status}");
            status.success()
        }
        Ok(Err(err)) => {
            tracing::error!("error waiting for nym-socks5-proxy: {err}");
            false
        }
        Err(_) => {
            tracing::warn!("nym-socks5-proxy did not exit within timeout, killing it");
            let _ = child.kill().await;
            false
        }
    };

    if let Some(tx) = ready_tx.take() {
        let _ = tx.send(Err(
            "nym-socks5-proxy exited before sending ready message".to_string()
        ));
    }

    let _ = event_tx.send(Socks5ProcessEvent::Exited { success });
}

fn handle_proxy_line(
    line: &str,
    ready_tx: &mut Option<oneshot::Sender<std::result::Result<(), String>>>,
    event_tx: &mpsc::UnboundedSender<Socks5ProcessEvent>,
) {
    let msg = match line.parse::<ProxyMessage>() {
        Ok(m) => m,
        Err(err) => {
            tracing::warn!("failed to parse nym-socks5-proxy message: {err}; raw: {line}");
            return;
        }
    };

    match msg {
        ProxyMessage::Ack => {
            if let Some(tx) = ready_tx.take() {
                // First Ack — proxy has bound its listener and is ready.
                tracing::debug!("nym-socks5-proxy ready (startup Ack)");
                let _ = tx.send(Ok(()));
                let _ = event_tx.send(Socks5ProcessEvent::Ready);
            } else {
                // Subsequent Acks — acknowledgement of a VPN state-change message.
                tracing::debug!("nym-socks5-proxy acknowledged message");
            }
        }
        ProxyMessage::Status(info) => {
            tracing::debug!(
                active_connections = info.active_connections,
                "nym-socks5-proxy status",
            );
            let _ = event_tx.send(Socks5ProcessEvent::StatusUpdate {
                active_connections: info.active_connections,
            });
        }
        ProxyMessage::Error(info) => {
            tracing::error!(message = %info.message, "nym-socks5-proxy reported error");
            if let Some(tx) = ready_tx.take() {
                let _ = tx.send(Err(info.message.clone()));
            }
            let _ = event_tx.send(Socks5ProcessEvent::Error {
                message: info.message,
            });
        }
    }
}

pub(super) struct Socks5ProxyProcess {
    pub(super) handle: Socks5ProcessHandle,
    pub(super) join_handle: JoinHandle<()>,
}

#[derive(Debug, thiserror::Error)]
pub enum Socks5ProcessError {
    #[error("Could not find nym-socks5-proxy executable: {0}")]
    BinaryNotFound(String),

    #[error("Failed to spawn nym-socks5-proxy: {0}")]
    Spawn(#[source] std::io::Error),

    #[error("Failed to send message to proxy: {0}")]
    #[allow(dead_code)]
    Send(#[source] std::io::Error),

    #[error("Proxy process exited before reporting ready")]
    ExitedBeforeReady,

    #[error("Proxy reported an error: {0}")]
    ProxyError(String),
}
