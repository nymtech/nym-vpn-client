// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{env, path::PathBuf, process::Stdio, result::Result, time::Duration};

use nym_socks5_proxy_ipc::{DaemonMessage, InterfaceAddresses, ProxyConfig, ProxyMessage};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use super::Socks5ProxyEvent;

const PROXY_BINARY_NAME: &str = if cfg!(windows) {
    "nym-socks5-proxy.exe"
} else {
    "nym-socks5-proxy"
};

/// Default maximum time to wait for nym-socks5-proxy to report readiness before
/// giving up on it. Overridable at runtime via `READY_TIMEOUT_ENV` for slow CI or
/// constrained environments.
const READY_TIMEOUT: Duration = Duration::from_secs(30);

const READY_TIMEOUT_ENV: &str = "NYM_SOCKS5_PROXY_READY_TIMEOUT_SECS";

fn ready_timeout() -> Duration {
    resolve_ready_timeout(env::var(READY_TIMEOUT_ENV).ok())
}

fn resolve_ready_timeout(raw: Option<String>) -> Duration {
    match raw {
        Some(val) => match val.parse::<u64>() {
            Ok(secs) => Duration::from_secs(secs),
            Err(_) => {
                tracing::warn!(
                    "invalid {READY_TIMEOUT_ENV}={val:?}; using default {READY_TIMEOUT:?}"
                );
                READY_TIMEOUT
            }
        },
        None => READY_TIMEOUT,
    }
}

pub struct RunningProcess {
    msg_tx: mpsc::UnboundedSender<DaemonMessage>,
    shutdown_token: CancellationToken,
    join_handle: JoinHandle<()>,
}

impl RunningProcess {
    pub fn set_tunnel_addrs(&self, tunnel_addrs: InterfaceAddresses) {
        let msg = DaemonMessage::SetTunnelAddresses(tunnel_addrs);
        if self.msg_tx.send(msg).is_err() {
            tracing::warn!("could not send SetTunnelAddresses to proxy: channel closed");
        }
    }

    pub fn shutdown(&self) {
        self.shutdown_token.cancel();
    }

    pub async fn join(self) {
        if let Err(err) = self.join_handle.await {
            tracing::error!("Failed to join socks5 process task: {err}");
        }
    }
}

pub async fn spawn(
    config: ProxyConfig,
    event_tx: mpsc::UnboundedSender<Socks5ProxyEvent>,
    shutdown_token: CancellationToken,
) -> Result<RunningProcess, SpawnError> {
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
        // Capture stderr and forward it into the daemon log (backup for proxy logs)
        .stderr(Stdio::piped());
    // Run in own process group on Unix so it doesn't inherit a terminal.
    #[cfg(unix)]
    command.process_group(0);

    let mut child = command.spawn().map_err(SpawnError::Spawn)?;

    let stdin = child
        .stdin
        .take()
        .expect("stdin was piped but is unavailable");
    let stdout = child
        .stdout
        .take()
        .expect("stdout was piped but is unavailable");
    let stderr = child
        .stderr
        .take()
        .expect("stderr was piped but is unavailable");

    tokio::spawn(stderr_forwarder(stderr));

    // Channel for sending DaemonMessages from handle → writer task → child stdin.
    let (msg_tx, msg_rx) = mpsc::unbounded_channel::<DaemonMessage>();

    // Send the initial Configure immediately.
    msg_tx
        .send(DaemonMessage::Configure(config.clone()))
        .expect("channel is open");

    // Channel to receive the ready signal from the supervisor.
    let (ready_tx, ready_rx) = oneshot::channel::<Result<(), String>>();

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

    // Wait for the proxy to become ready, but never block forever: a proxy that
    // starts yet never reports ready would otherwise wedge the tunnel state
    // machine.
    let ready_timeout = ready_timeout();
    if let Err(err) = await_ready(ready_rx, ready_timeout).await {
        if matches!(err, SpawnError::ReadyTimeout(_)) {
            tracing::warn!(
                "nym-socks5-proxy did not report ready within {ready_timeout:?}; tearing it down and continuing without it"
            );
        }
        // Safe for all three error variants: cancellation is idempotent and
        // joining a supervisor that has already exited (ProxyError,
        // ExitedBeforeReady) or is mid-exit returns immediately without racing.
        teardown_after_failed_start(&supervisor_token, join_handle).await;
        return Err(err);
    }

    tracing::info!("nym-socks5-proxy is ready");

    Ok(RunningProcess {
        msg_tx,
        shutdown_token: supervisor_token,
        join_handle,
    })
}

/// Classify the proxy readiness outcome, bounded by `timeout`.
async fn await_ready(
    ready_rx: oneshot::Receiver<Result<(), String>>,
    timeout: Duration,
) -> Result<(), SpawnError> {
    match tokio::time::timeout(timeout, ready_rx).await {
        Ok(Ok(Ok(()))) => Ok(()),
        Ok(Ok(Err(msg))) => Err(SpawnError::ProxyError(msg)),
        Ok(Err(_)) => Err(SpawnError::ExitedBeforeReady),
        Err(_) => Err(SpawnError::ReadyTimeout(timeout)),
    }
}

async fn teardown_after_failed_start(
    supervisor_token: &CancellationToken,
    join_handle: JoinHandle<()>,
) {
    supervisor_token.cancel();
    if let Err(join_err) = join_handle.await {
        tracing::error!("nym-socks5-proxy supervisor task panicked during teardown: {join_err}");
    }
}

async fn stderr_forwarder(stderr: ChildStderr) {
    let mut lines = BufReader::new(stderr).lines();
    loop {
        match lines.next_line().await {
            Ok(Some(line)) => tracing::warn!("nym-socks5-proxy stderr: {line}"),
            Ok(None) => break,
            Err(err) => {
                tracing::debug!("error reading nym-socks5-proxy stderr: {err}");
                break;
            }
        }
    }
}

async fn supervisor(
    mut child: Child,
    stdin: ChildStdin,
    msg_rx: mpsc::UnboundedReceiver<DaemonMessage>,
    stdout: ChildStdout,
    ready_tx: oneshot::Sender<Result<(), String>>,
    event_tx: mpsc::UnboundedSender<Socks5ProxyEvent>,
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
                        tracing::debug!("nym-socks5-proxy stdout: {line}");
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

    let _ = event_tx.send(Socks5ProxyEvent::Exited { success });
}

fn handle_proxy_line(
    line: &str,
    ready_tx: &mut Option<oneshot::Sender<Result<(), String>>>,
    event_tx: &mpsc::UnboundedSender<Socks5ProxyEvent>,
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
                let _ = event_tx.send(Socks5ProxyEvent::Ready);
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
        }
        ProxyMessage::Error(info) => {
            tracing::error!(message = %info.message, "nym-socks5-proxy reported error");
            if let Some(tx) = ready_tx.take() {
                let _ = tx.send(Err(info.message.clone()));
            }
            let _ = event_tx.send(Socks5ProxyEvent::Error {
                message: info.message,
            });
        }
    }
}

pub fn find_proxy_binary() -> Result<PathBuf, SpawnError> {
    let exe = env::current_exe().map_err(|e| {
        SpawnError::BinaryNotFound(format!("Could not determine current executable path: {e}"))
    })?;
    let dir = exe.parent().ok_or_else(|| {
        SpawnError::BinaryNotFound("Current executable has no parent directory".to_string())
    })?;
    let candidate = dir.join(PROXY_BINARY_NAME);
    if candidate.exists() {
        return Ok(candidate);
    }
    Err(SpawnError::BinaryNotFound(format!(
        "Expected to find '{}' at '{}' but it was not found",
        PROXY_BINARY_NAME,
        candidate.display(),
    )))
}

async fn stdin_writer(
    mut stdin: ChildStdin,
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

#[derive(Debug, thiserror::Error)]
pub enum SpawnError {
    #[error("Could not find nym-socks5-proxy executable: {0}")]
    BinaryNotFound(String),

    #[error("Failed to spawn nym-socks5-proxy: {0}")]
    Spawn(#[source] std::io::Error),

    #[error("Proxy process exited before reporting ready")]
    ExitedBeforeReady,

    #[error("Proxy did not report ready within {0:?}")]
    ReadyTimeout(Duration),

    #[error("Proxy reported an error: {0}")]
    ProxyError(String),
}
