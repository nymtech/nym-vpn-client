// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::net::UdpSocket;

#[cfg(not(target_os = "ios"))]
use crate::resolver::LoopbackAlias;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use crate::resolver::unix::RandomLoopbackAlias;
#[cfg(windows)]
use crate::resolver::windows::RandomLoopbackAlias;
use crate::resolver::{BoxedLoopbackAlias, Error};

pub async fn new_random_socket(
    port: u16,
    use_random_loopback: bool,
) -> Result<(UdpSocket, Option<BoxedLoopbackAlias>), Error> {
    for attempt in 0.. {
        let (ip, alias): (IpAddr, Option<BoxedLoopbackAlias>) = match attempt {
            ..3 if !use_random_loopback => continue,
            ..3 => {
                #[cfg(not(target_os = "ios"))]
                {
                    match RandomLoopbackAlias::assign().await {
                        Ok(random) => (random.addr(), Some(Box::new(random) as BoxedLoopbackAlias)),
                        Err(_) => continue,
                    }
                }
                // iOS: unsupported
                #[cfg(target_os = "ios")]
                {
                    continue;
                }
            }
            3 => (IpAddr::from(Ipv4Addr::LOCALHOST), None),
            4.. => break,
        };

        match new_udp_socket(SocketAddr::new(ip, port), true).await {
            Ok(socket) => {
                return Ok((socket, alias));
            }
            Err(_err) => {
                // Ensure we clean up the alias before retrying.
                if let Some(alias) = alias {
                    alias.unassign().await;
                }
            }
        }
    }

    Err(Error::UdpBind)
}

pub async fn new_udp_socket(addr: SocketAddr, reuse_addr: bool) -> std::io::Result<UdpSocket> {
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).inspect_err(|err| {
        tracing::error!("Failed to open IPv4/UDP socket: {err}");
    })?;

    // SO_NONBLOCK is required for turning this into a tokio socket.
    sock.set_nonblocking(true).inspect_err(|err| {
        tracing::warn!("Failed to set UDP socket as nonblocking: {err}");
    })?;

    // SO_REUSEADDR enables us to bind to `127.x.y.z` even if another socket is bound to `0.0.0.0`.
    // Best-effort: allow binding even if wildcard is in use. Windows semantics differ but
    // this is harmless.
    if reuse_addr && let Err(err) = sock.set_reuse_address(true) {
        tracing::warn!("Failed to set SO_REUSEADDR on UDP socket: {err}");
    }

    sock.bind(&SockAddr::from(addr)).inspect_err(|err| {
        tracing::warn!("Failed to bind UDP socket to {addr}: {err}");
    })?;

    UdpSocket::from_std(std::net::UdpSocket::from(sock))
}
