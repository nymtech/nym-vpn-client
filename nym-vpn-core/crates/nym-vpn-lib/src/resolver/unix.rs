// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{io, net::IpAddr};

use async_trait::async_trait;
use tokio::task::JoinHandle;
use tokio_util::sync::{CancellationToken, DropGuard};

use crate::resolver::{LoopbackAlias, random_loopback_ipv4};

/// Loopback interface name.
#[cfg(target_os = "macos")]
const LOOPBACK: &str = "lo0";
#[cfg(target_os = "linux")]
const LOOPBACK: &str = "lo";

pub struct RandomLoopbackAlias {
    addr: IpAddr,
    drop_guard: DropGuard,
    unassign_task: JoinHandle<()>,
}

impl RandomLoopbackAlias {
    pub async fn assign() -> io::Result<Self> {
        let addr = random_loopback_ipv4();

        assign_loopback_alias(addr).await.inspect_err(|e| {
            tracing::warn!("Failed to add loopback {LOOPBACK} alias {addr}: {e}");
        })?;

        tracing::debug!("Created loopback address {addr}");

        let shutdown_token = CancellationToken::new();

        let child_token = shutdown_token.child_token();
        let unassign_task = tokio::task::spawn(async move {
            child_token.cancelled().await;

            tracing::debug!("Cleaning up loopback address {addr}");
            if let Err(e) = remove_loopback_alias(addr).await {
                tracing::warn!("Failed to clean up {LOOPBACK} alias {addr}: {e}");
            }
        });

        let drop_guard = shutdown_token.drop_guard();

        Ok(Self {
            addr,
            drop_guard,
            unassign_task,
        })
    }
}

#[cfg(target_os = "macos")]
async fn assign_loopback_alias(addr: IpAddr) -> io::Result<()> {
    nym_macos::net::add_alias(LOOPBACK, addr).await
}

#[cfg(target_os = "linux")]
async fn assign_loopback_alias(addr: IpAddr) -> io::Result<()> {
    // Add as /32: the 127.0.0.0/8 route typically already exists on `lo`.
    // TODO: Replace with nym-ifconfig
    let output = run_ip(["addr", "add", &format!("{addr}/32"), "dev", LOOPBACK]).await?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("File exists") {
        return Ok(());
    }

    Err(io::Error::other(format!(
        "failed to add loopback alias {addr}: {stderr}"
    )))
}

#[cfg(target_os = "macos")]
async fn remove_loopback_alias(addr: IpAddr) -> io::Result<()> {
    nym_macos::net::remove_alias(LOOPBACK, addr).await
}

#[cfg(target_os = "linux")]
async fn remove_loopback_alias(addr: IpAddr) -> io::Result<()> {
    // TODO: Replace with nym-ifconfig
    let output = run_ip(["addr", "del", &format!("{addr}/32"), "dev", LOOPBACK]).await?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("Cannot assign requested address") {
        return Ok(());
    }

    Err(io::Error::other(format!(
        "failed to remove loopback alias {addr}: {stderr}"
    )))
}

#[cfg(target_os = "linux")]
// TODO: Replace with nym-ifconfig
async fn run_ip<'a>(args: impl IntoIterator<Item = &'a str>) -> io::Result<std::process::Output> {
    const CANDIDATES: &[&str] = &["ip", "/usr/sbin/ip", "/sbin/ip", "/usr/bin/ip", "/bin/ip"];

    let args: Vec<&str> = args.into_iter().collect();
    let mut last_err: Option<io::Error> = None;

    for bin in CANDIDATES {
        match tokio::process::Command::new(bin).args(&args).output().await {
            Ok(output) => return Ok(output),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                last_err = Some(e);
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Err(last_err.unwrap_or_else(|| io::Error::new(io::ErrorKind::NotFound, "ip binary not found")))
}

#[async_trait]
impl LoopbackAlias for RandomLoopbackAlias {
    fn addr(&self) -> IpAddr {
        self.addr
    }

    async fn unassign(self: Box<Self>) {
        drop(self.drop_guard);
        self.unassign_task.await.ok();
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub async fn flush_system_cache() {
    #[cfg(target_os = "macos")]
    {
        if let Err(error) = kill_mdnsresponder() {
            tracing::error!("Failed to kill mDNSResponder: {error}");
        }
    }

    #[cfg(target_os = "linux")]
    {
        let _ = tokio::process::Command::new("resolvectl")
            .arg("flush-caches")
            .output()
            .await;
        let _ = tokio::process::Command::new("systemd-resolve")
            .arg("--flush-caches")
            .output()
            .await;
    }
}

#[cfg(target_os = "macos")]
const MDNS_RESPONDER_PATH: &str = "/usr/sbin/mDNSResponder";

#[cfg(target_os = "macos")]
fn kill_mdnsresponder() -> io::Result<()> {
    if let Some(mdns_pid) = nym_macos::process::pid_of_path(MDNS_RESPONDER_PATH) {
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(mdns_pid),
            nix::sys::signal::SIGHUP,
        )?;
    }
    Ok(())
}
