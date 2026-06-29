// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::io;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
#[cfg(any(target_os = "ios", target_os = "macos"))]
use std::num::NonZeroU32;
#[cfg(unix)]
use std::os::fd::{FromRawFd, IntoRawFd};
use std::time::Duration;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::TunnelMetadata;

const METADATA_PROXY_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

fn metadata_listen_addr(destination: SocketAddr) -> SocketAddr {
    match destination {
        SocketAddr::V4(_) => SocketAddr::from(([127, 0, 0, 1], 0)),
        SocketAddr::V6(_) => SocketAddr::from((Ipv6Addr::LOCALHOST, 0)),
    }
}

fn metadata_bind_ip(ips: &[IpAddr], destination: SocketAddr) -> Option<IpAddr> {
    match destination {
        SocketAddr::V4(_) => ips.iter().find(|ip| ip.is_ipv4()).copied(),
        SocketAddr::V6(_) => ips.iter().find(|ip| ip.is_ipv6()).copied(),
    }
}

pub struct MetadataTcpProxy {
    pub listen_addr: SocketAddr,
    _task: JoinHandle<()>,
}

impl MetadataTcpProxy {
    pub async fn start(
        tunnel: &TunnelMetadata,
        destination: SocketAddr,
        shutdown: CancellationToken,
    ) -> io::Result<Self> {
        let bind_ip = metadata_bind_ip(&tunnel.ips, destination)
            .ok_or_else(|| io::Error::other("tunnel metadata has no bind address"))?;

        let listener = TcpListener::bind(metadata_listen_addr(destination)).await?;
        let listen_addr = listener.local_addr()?;
        let interface = tunnel.interface.clone();

        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((mut inbound, _)) => {
                                let interface = interface.clone();
                                tokio::spawn(async move {
                                    match connect_via_tunnel_interface(
                                        &interface,
                                        bind_ip,
                                        destination,
                                    )
                                    .await
                                    {
                                        Ok(mut outbound) => {
                                            if let Err(err) = tokio::io::copy_bidirectional(
                                                &mut inbound,
                                                &mut outbound,
                                            )
                                            .await
                                            {
                                                tracing::debug!(
                                                    %err,
                                                    "Metadata proxy session ended with error"
                                                );
                                            }
                                        }
                                        Err(err) => {
                                            tracing::warn!(
                                                %err,
                                                "Metadata proxy failed to connect outbound via tunnel interface"
                                            );
                                        }
                                    }
                                });
                            }
                            Err(err) => {
                                tracing::warn!(%err, "Metadata proxy accept error");
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            listen_addr,
            _task: task,
        })
    }
}

async fn connect_via_tunnel_interface(
    interface: &str,
    bind_ip: IpAddr,
    destination: SocketAddr,
) -> io::Result<tokio::net::TcpStream> {
    let socket = create_bound_socket(interface, bind_ip, destination)?;
    #[cfg(unix)]
    {
        // SAFETY: `socket` is a valid owned fd from `Socket::new`; nonblocking was set before
        // transfer. Ownership moves to `TcpSocket`, which closes the fd on drop.
        let tcp = unsafe { tokio::net::TcpSocket::from_raw_fd(socket.into_raw_fd()) };
        match tokio::time::timeout(METADATA_PROXY_CONNECT_TIMEOUT, tcp.connect(destination)).await {
            Ok(connect_result) => connect_result,
            Err(_) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "metadata proxy outbound connect timed out",
            )),
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (interface, bind_ip, destination, socket);
        Err(io::Error::other(
            "tunnel-bound metadata proxy is unsupported on this platform",
        ))
    }
}

fn create_bound_socket(
    interface: &str,
    bind_ip: IpAddr,
    destination: SocketAddr,
) -> io::Result<Socket> {
    let domain = match destination {
        SocketAddr::V4(_) => Domain::IPV4,
        SocketAddr::V6(_) => Domain::IPV6,
    };

    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_nonblocking(true)?;
    bind_socket_to_interface(&socket, interface, destination)?;
    socket.bind(&SockAddr::from(SocketAddr::new(bind_ip, 0)))?;
    Ok(socket)
}

fn bind_socket_to_interface(
    socket: &Socket,
    interface: &str,
    destination: SocketAddr,
) -> io::Result<()> {
    #[cfg(any(target_os = "android", target_os = "linux"))]
    {
        let _ = destination;
        socket.bind_device(Some(interface.as_bytes()))
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    {
        // bind_device_by_index must run before socket.bind on Apple platforms (IP_BOUND_IF /
        // IPV6_BOUND_IF ordering). `nix` is declared under `[target.'cfg(unix)'.dependencies]`.
        let index = nix::net::if_::if_nametoindex(interface)
            .map_err(|err| io::Error::other(format!("if_nametoindex({interface}): {err}")))?;
        let index = NonZeroU32::new(index)
            .ok_or_else(|| io::Error::other(format!("invalid interface index for {interface}")))?;
        match destination {
            SocketAddr::V4(_) => socket.bind_device_by_index_v4(Some(index)),
            SocketAddr::V6(_) => socket.bind_device_by_index_v6(Some(index)),
        }
    }

    #[cfg(windows)]
    {
        // MetadataTcpProxy is only started on unix; this arm exists so the helper compiles on
        // Windows where outbound connect returns unsupported before bind is exercised.
        let _ = (socket, interface, destination);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use super::*;

    #[test]
    fn metadata_listen_addr_matches_destination_address_family() {
        use std::net::Ipv4Addr;

        assert!(matches!(
            metadata_listen_addr(SocketAddr::from((Ipv4Addr::LOCALHOST, 51830))),
            SocketAddr::V4(_)
        ));
        assert!(matches!(
            metadata_listen_addr(SocketAddr::from((Ipv6Addr::LOCALHOST, 51830))),
            SocketAddr::V6(_)
        ));
    }

    #[test]
    fn metadata_bind_ip_prefers_ipv4_for_ipv4_destination() {
        let ips = [
            IpAddr::V4(Ipv4Addr::new(10, 1, 0, 2)),
            IpAddr::V6(Ipv6Addr::LOCALHOST),
        ];
        let dest = SocketAddr::from((Ipv4Addr::new(10, 1, 0, 1), 51830));

        assert_eq!(
            metadata_bind_ip(&ips, dest),
            Some(IpAddr::V4(Ipv4Addr::new(10, 1, 0, 2)))
        );
    }

    #[test]
    fn metadata_bind_ip_empty_returns_none() {
        assert!(metadata_bind_ip(&[], SocketAddr::from(([127, 0, 0, 1], 1))).is_none());
    }

    #[test]
    fn metadata_bind_ip_ipv6_destination_requires_ipv6_address() {
        let ips = [IpAddr::V4(Ipv4Addr::new(10, 1, 0, 2))];
        let dest = SocketAddr::from((Ipv6Addr::LOCALHOST, 51830));

        assert!(metadata_bind_ip(&ips, dest).is_none());
    }

    #[test]
    fn metadata_bind_ip_ipv6_destination_picks_ipv6_when_present() {
        let ips = [
            IpAddr::V4(Ipv4Addr::new(10, 1, 0, 2)),
            IpAddr::V6(Ipv6Addr::new(0xfd, 0, 0, 0, 0, 0, 0, 2)),
        ];
        let dest = SocketAddr::from((Ipv6Addr::new(0xfd, 0, 0, 0, 0, 0, 0, 1), 51830));

        assert_eq!(
            metadata_bind_ip(&ips, dest),
            Some(IpAddr::V6(Ipv6Addr::new(0xfd, 0, 0, 0, 0, 0, 0, 2)))
        );
    }
}
