// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, Ipv4Addr};

use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::net::UdpSocket;

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "linux"))]
use crate::resolver::unix::RandomLoopbackAlias;
#[cfg(windows)]
use crate::resolver::windows::RandomLoopbackAlias;
use crate::resolver::{BoxedLoopbackAlias, Error, LoopbackAlias};

pub async fn new_random_socket(
    port: u16,
    use_random_loopback: bool,
) -> Result<(UdpSocket, Option<BoxedLoopbackAlias>), Error> {
    for attempt in 0.. {
        let (ip, alias): (IpAddr, Option<BoxedLoopbackAlias>) = match attempt {
            ..3 if !use_random_loopback => continue,
            ..3 => match RandomLoopbackAlias::assign().await {
                Ok(random) => (random.addr(), Some(Box::new(random) as BoxedLoopbackAlias)),
                Err(_) => continue,
            },
            3 => (IpAddr::from(Ipv4Addr::LOCALHOST), None),
            4.. => break,
        };

        let sock = match Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)) {
            Ok(sock) => sock,
            Err(err) => {
                tracing::error!("Failed to open IPv4/UDP socket: {err}");
                continue;
            }
        };

        // SO_NONBLOCK is required for turning this into a tokio socket.
        if let Err(err) = sock.set_nonblocking(true) {
            tracing::warn!("Failed to set UDP socket as nonblocking: {err}");
            continue;
        }

        // SO_REUSEADDR enables us to bind to `127.x.y.z` even if another socket is bound to `0.0.0.0`.
        // Best-effort: allow binding even if wildcard is in use. Windows semantics differ but
        // this is harmless.
        if let Err(err) = sock.set_reuse_address(true) {
            tracing::warn!("Failed to set SO_REUSEADDR on UDP socket: {err}");
        }

        let sa = SockAddr::from(std::net::SocketAddr::new(ip, port));
        if let Err(err) = sock.bind(&sa) {
            tracing::warn!("Failed to bind UDP socket to {ip}: {err}");
            // Ensure we clean up the alias before retrying.
            if let Some(alias) = alias {
                alias.unassign().await;
            }
            continue;
        }

        let socket =
            UdpSocket::from_std(std::net::UdpSocket::from(sock)).expect("socket is non-blocking");
        return Ok((socket, alias));
    }

    Err(Error::UdpBind)
}
