// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::resolver::{BoxedLoopbackAlias, Error, LoopbackAlias, random_loopback_ipv4};
use async_trait::async_trait;
use std::{
    io,
    net::{IpAddr, Ipv4Addr},
};
use tokio::{net::UdpSocket, task::JoinHandle};
use tokio_util::sync::{CancellationToken, DropGuard};

/// Loopback interface name.
#[cfg(target_os = "macos")]
const LOOPBACK: &str = "lo0";
#[cfg(target_os = "linux")]
const LOOPBACK: &str = "lo";

struct RandomLoopbackAlias {
    addr: IpAddr,
    drop_guard: DropGuard,
    unassign_task: JoinHandle<()>,
}

impl RandomLoopbackAlias {
    async fn assign() -> io::Result<Self> {
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

#[cfg(target_os = "macos")]
async fn remove_loopback_alias(addr: IpAddr) -> io::Result<()> {
    nym_macos::net::remove_alias(LOOPBACK, addr).await
}

#[cfg(target_os = "linux")]
async fn assign_loopback_alias(addr: IpAddr) -> io::Result<()> {
    // Add as /32: the 127.0.0.0/8 route typically already exists on `lo`.
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

#[cfg(target_os = "linux")]
async fn remove_loopback_alias(addr: IpAddr) -> io::Result<()> {
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

            #[cfg(target_os = "macos")]
            ..3 => match RandomLoopbackAlias::assign().await {
                Ok(random) => (random.addr(), Some(Box::new(random) as BoxedLoopbackAlias)),
                Err(_) => continue,
            },

            #[cfg(target_os = "linux")]
            ..3 => match RandomLoopbackAlias::assign().await {
                Ok(random) => (random.addr(), Some(Box::new(random) as BoxedLoopbackAlias)),
                Err(error) => {
                    // Still keep the address random even if alias assignment fails.
                    tracing::warn!(
                        "Failed to add loopback alias on Linux; falling back to random bind: {error}"
                    );
                    (random_loopback_ipv4(), None)
                }
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
            Err(err) => {
                tracing::warn!("Failed to bind DNS server to {socket_addr}: {err}");
                if let Some(on_drop) = on_drop {
                    on_drop.unassign().await;
                }
            }
        }
    }

    Err(Error::UdpBind)
}

pub(crate) fn flush_system_cache() {
    #[cfg(target_os = "macos")]
    {
        if let Err(error) = kill_mdnsresponder() {
            tracing::error!("Failed to kill mDNSResponder: {error}");
        }
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("resolvectl")
            .arg("flush-caches")
            .output();
        let _ = std::process::Command::new("systemd-resolve")
            .arg("--flush-caches")
            .output();
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
