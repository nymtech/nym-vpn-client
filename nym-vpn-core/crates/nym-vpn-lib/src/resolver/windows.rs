// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::resolver::{LoopbackAlias, random_loopback_ipv4};
use async_trait::async_trait;
use nym_windows::net::{
    add_ip_address_for_interface, loopback_luid, remove_ip_address_for_interface,
};
use std::net::IpAddr;
use tokio::task::JoinHandle;
use tokio_util::sync::{CancellationToken, DropGuard};

pub struct RandomLoopbackAlias {
    addr: IpAddr,
    drop_guard: DropGuard,
    unassign_task: JoinHandle<()>,
}

impl RandomLoopbackAlias {
    pub async fn assign() -> std::io::Result<Self> {
        let addr = random_loopback_ipv4();
        let luid = loopback_luid()?;

        // Adding/removing IPs typically requires elevation.
        // If this fails, the caller will just try another address or fall back to 127.0.0.1.
        add_ip_address_for_interface(luid, addr).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("failed to add loopback alias {addr}: {e}"),
            )
        })?;

        tracing::debug!("Created Windows loopback address {addr}");

        let shutdown_token = CancellationToken::new();

        let child_token = shutdown_token.child_token();
        let unassign_task = tokio::task::spawn(async move {
            child_token.cancelled().await;

            tracing::debug!("Cleaning up Windows loopback address {addr}");
            if let Err(e) = remove_ip_address_for_interface(luid, addr) {
                tracing::warn!("Failed to clean up loopback alias {addr}: {e}");
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

pub async fn flush_system_cache() {
    // Best-effort. If this fails we still keep running.
    if let Err(err) = nym_dns::flush_resolver_cache() {
        tracing::warn!("Failed to flush dns: {err}");
    }
}
