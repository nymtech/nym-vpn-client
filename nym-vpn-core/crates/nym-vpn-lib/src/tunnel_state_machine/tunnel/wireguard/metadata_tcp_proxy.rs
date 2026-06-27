// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::io;
use std::net::{IpAddr, SocketAddr};
#[cfg(any(target_os = "ios", target_os = "macos"))]
use std::num::NonZeroU32;
#[cfg(unix)]
use std::os::fd::{FromRawFd, IntoRawFd};

use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::TunnelMetadata;

fn metadata_bind_ip(ips: &[IpAddr], destination: SocketAddr) -> Option<IpAddr> {
    ips.iter()
        .find(|ip| matches!(destination, SocketAddr::V4(_) if ip.is_ipv4()))
        .or_else(|| ips.first())
        .copied()
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

        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).await?;
        let listen_addr = listener.local_addr()?;
        let interface = tunnel.interface.clone();

        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    accept_result = listener.accept() => {
                        let Ok((mut inbound, _)) = accept_result else {
                            tracing::debug!("Metadata proxy accept failed; stopping listener");
                            break;
                        };
                        let interface = interface.clone();
                        tokio::spawn(async move {
                            let Ok(mut outbound) =
                                connect_via_tunnel_interface(&interface, bind_ip, destination).await
                            else {
                                return;
                            };
                            let _ = tokio::io::copy_bidirectional(&mut inbound, &mut outbound).await;
                        });
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
        let tcp = unsafe { tokio::net::TcpSocket::from_raw_fd(socket.into_raw_fd()) };
        tcp.connect(destination).await
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
        let _ = (socket, interface, destination);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use super::*;

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
}
