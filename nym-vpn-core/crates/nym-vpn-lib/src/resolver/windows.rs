// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::resolver::{BoxedLoopbackAlias, Error, LoopbackAlias, random_loopback_ipv4};
use async_trait::async_trait;
use nym_windows::net::{
    add_ip_address_for_interface, loopback_luid, remove_ip_address_for_interface,
};
use std::net::{IpAddr, Ipv4Addr};
use tokio::{net::UdpSocket, task::JoinHandle};
use tokio_util::sync::{CancellationToken, DropGuard};

struct RandomLoopbackAlias {
    addr: IpAddr,
    drop_guard: DropGuard,
    unassign_task: JoinHandle<()>,
}

impl RandomLoopbackAlias {
    async fn assign() -> std::io::Result<Self> {
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

pub(crate) async fn new_random_socket(
    port: u16,
    use_random_loopback: bool,
) -> Result<(UdpSocket, Option<BoxedLoopbackAlias>), Error> {
    use socket2::{Domain, Protocol, Socket, Type};

    // Try a few random loopback IPs first to reduce collisions with other local resolvers.
    for attempt in 0.. {
        let (ip, alias): (IpAddr, Option<BoxedLoopbackAlias>) = match attempt {
            ..3 if !use_random_loopback => continue,
            ..3 => match RandomLoopbackAlias::assign().await {
                Ok(random) => (random.addr(), Some(Box::new(random) as BoxedLoopbackAlias)),
                Err(e) => {
                    tracing::warn!("Failed to add random loopback alias: {e}");
                    continue;
                }
            },
            3 => (IpAddr::V4(Ipv4Addr::LOCALHOST), None),
            4.. => break,
        };

        let sock = match Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to create IPv4/UDP socket: {e}");
                continue;
            }
        };

        // Required for tokio.
        if let Err(e) = sock.set_nonblocking(true) {
            tracing::warn!("Failed to set socket nonblocking: {e}");
            continue;
        }

        // Best-effort: allow binding even if wildcard is in use. Windows semantics differ but
        // this is harmless.
        let _ = sock.set_reuse_address(true);

        let addr = std::net::SocketAddr::new(ip, port);
        if let Err(e) = sock.bind(&addr.into()) {
            tracing::warn!("Failed to bind DNS server to {ip}: {e}");
            // Ensure we clean up the alias before retrying.
            if let Some(alias) = alias {
                alias.unassign().await;
            }
            continue;
        }

        let std_socket: std::net::UdpSocket = sock.into();
        let socket = UdpSocket::from_std(std_socket).map_err(|_| Error::UdpBind)?;
        return Ok((socket, alias));
    }

    Err(Error::UdpBind)
}

pub(crate) fn flush_system_cache() {
    // Best-effort. If this fails we still keep running.
    if let Err(err) = nym_dns::flush_resolver_cache() {
        tracing::warn!("Failed to flush dns: {err}");
    }
}
