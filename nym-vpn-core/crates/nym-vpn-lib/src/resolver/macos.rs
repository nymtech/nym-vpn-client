// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    io,
    net::{IpAddr, Ipv4Addr},
};

use tokio::{net::UdpSocket, task::JoinHandle};
use tokio_util::sync::{CancellationToken, DropGuard};

use crate::resolver::{BoxedLoopbackAlias, Error, LoopbackAlias, random_loopback_ipv4};

/// Loopback interface name.
const LOOPBACK: &str = "lo0";

struct RandomLoopbackAlias {
    addr: IpAddr,
    drop_guard: DropGuard,
    unassign_task: JoinHandle<()>,
}

impl RandomLoopbackAlias {
    async fn assign() -> io::Result<Self> {
        let addr = random_loopback_ipv4();

        nym_macos::net::add_alias(LOOPBACK, addr)
            .await
            .inspect_err(|e| {
                tracing::warn!("Failed to add loopback {LOOPBACK} alias {addr}: {e}");
            })?;

        tracing::debug!("Created loopback address {addr}");

        let shutdown_token = CancellationToken::new();

        let child_token = shutdown_token.child_token();
        let unassign_task = tokio::task::spawn(async move {
            child_token.cancelled().await;

            tracing::debug!("Cleaning up loopback address {addr}");
            if let Err(e) = nym_macos::net::remove_alias(LOOPBACK, addr).await {
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

impl LoopbackAlias for RandomLoopbackAlias {
    fn addr(&self) -> IpAddr {
        self.addr
    }

    fn unassign(
        self: Box<Self>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        Box::pin(async move {
            drop(self.drop_guard);
            self.unassign_task.await.ok();
        })
    }
}

pub(crate) async fn new_random_socket(
    port: u16,
    use_random_loopback: bool,
) -> Result<(UdpSocket, Option<BoxedLoopbackAlias>), Error> {
    use nix::{
        fcntl,
        sys::socket::{self, AddressFamily, SockFlag, SockProtocol, SockType, SockaddrStorage},
    };
    use std::os::fd::AsRawFd;

    for attempt in 0.. {
        let (socket_addr, on_drop): (IpAddr, Option<BoxedLoopbackAlias>) = match attempt {
            ..3 if !use_random_loopback => continue,
            ..3 => match RandomLoopbackAlias::assign().await {
                Ok(random) => (random.addr(), Some(Box::new(random) as BoxedLoopbackAlias)),
                Err(_) => continue,
            },
            3 => (IpAddr::from(Ipv4Addr::LOCALHOST), None),
            4.. => break,
        };

        let sock = match socket::socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            SockProtocol::Udp,
        ) {
            Ok(sock) => sock,
            Err(error) => {
                tracing::error!("Failed to open IPv4/UDP socket: {error}");
                continue;
            }
        };

        // SO_NONBLOCK is required for turning this into a tokio socket.
        if let Err(error) = fcntl::fcntl(&sock, fcntl::F_SETFL(fcntl::OFlag::O_NONBLOCK)) {
            tracing::warn!("Failed to set socket as nonblocking: {error}");
            continue;
        }

        // SO_REUSEADDR allows us to bind to `127.x.y.z` even if another socket is bound to
        // `0.0.0.0`.
        if let Err(error) = socket::setsockopt(&sock, socket::sockopt::ReuseAddr, &true) {
            tracing::warn!("Failed to set SO_REUSEADDR on resolver socket: {error}");
        }

        let sin = SockaddrStorage::from(std::net::SocketAddr::new(socket_addr, port));

        match socket::bind(sock.as_raw_fd(), &sin) {
            Ok(()) => {
                let socket = UdpSocket::from_std(sock.into()).expect("socket is non-blocking");
                return Ok((socket, on_drop));
            }
            Err(err) => tracing::warn!("Failed to bind DNS server to {socket_addr}: {err}"),
        }
    }

    Err(Error::UdpBind)
}

pub(crate) fn flush_system_cache() {
    if let Err(error) = kill_mdnsresponder() {
        tracing::error!("Failed to kill mDNSResponder: {error}");
    }
}

const MDNS_RESPONDER_PATH: &str = "/usr/sbin/mDNSResponder";

fn kill_mdnsresponder() -> io::Result<()> {
    if let Some(mdns_pid) = nym_macos::process::pid_of_path(MDNS_RESPONDER_PATH) {
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(mdns_pid),
            nix::sys::signal::SIGHUP,
        )?;
    }
    Ok(())
}
