// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, Ipv4Addr},
    process::Command,
};

use tokio::net::UdpSocket;

use crate::tunnel_state_machine::resolver::{
    BoxedLoopbackAlias, Error, LoopbackAlias, random_loopback_ipv4,
};

/// Windows currently doesn't create/remove loopback aliases here.
///
/// We still keep the alias interface around so the rest of the resolver code can use a uniform
/// interface. A future improvement is to add proper loopback aliasing using IP Helper APIs.
struct NoopAlias {
    addr: IpAddr,
}

impl LoopbackAlias for NoopAlias {
    fn addr(&self) -> IpAddr {
        self.addr
    }

    fn unassign(
        self: Box<Self>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        Box::pin(async move {})
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
            ..3 => {
                let addr = random_loopback_ipv4();
                // We don't currently assign the alias at the OS level. Binding will only work if
                // the IP is already usable on loopback. If it fails, we just try again.
                (
                    addr,
                    Some(Box::new(NoopAlias { addr }) as BoxedLoopbackAlias),
                )
            }
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
    let _ = Command::new("ipconfig").arg("/flushdns").output();
}
