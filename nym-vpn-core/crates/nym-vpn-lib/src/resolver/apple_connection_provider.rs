// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Custom implementation of runtime provider able to bind to interface
//! The original source can be found in hickory-proto/src/runtime.rs

use std::{
    ffi::CString,
    io,
    net::{SocketAddr, UdpSocket},
    num::NonZero,
    os::fd::{FromRawFd, IntoRawFd},
    pin::Pin,
    time::Duration,
};

use hickory_server::{
    net::runtime::{RuntimeProvider, TokioHandle, TokioTime, iocompat::AsyncIoTokioAsStd},
    resolver::Resolver,
};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::net::{TcpSocket, TcpStream, UdpSocket as TokioUdpSocket};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

pub type TokioResolver = Resolver<AppleConnectionProvider>;

/// Tokio-based runtime provider for hickory that supports binding sockets to network interface
#[derive(Clone, Default)]
pub struct AppleConnectionProvider {
    tokio_handle: TokioHandle,
    bind_interface: Option<String>,
}

impl AppleConnectionProvider {
    pub fn new(bind_interface: Option<String>) -> Self {
        Self {
            tokio_handle: TokioHandle::default(),
            bind_interface,
        }
    }
}

impl RuntimeProvider for AppleConnectionProvider {
    type Handle = TokioHandle;
    type Timer = TokioTime;
    type Udp = TokioUdpSocket;
    type Tcp = AsyncIoTokioAsStd<TcpStream>;

    fn create_handle(&self) -> Self::Handle {
        self.tokio_handle.clone()
    }

    fn connect_tcp(
        &self,
        server_addr: SocketAddr,
        bind_addr: Option<SocketAddr>,
        wait_for: Option<Duration>,
    ) -> Pin<Box<dyn Send + Future<Output = io::Result<Self::Tcp>>>> {
        let bind_interface = self.bind_interface.clone();

        Box::pin(async move {
            let domain = Domain::for_address(server_addr);
            let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
            socket.set_nonblocking(true)?;
            if let Some(bind_interface) = bind_interface {
                bind_to_iface(&socket, server_addr, &bind_interface)?;
            }

            // SAFETY: we own the socket and know it's valid
            let socket = unsafe { TcpSocket::from_raw_fd(socket.into_raw_fd()) };
            if let Some(bind_addr) = bind_addr {
                socket.bind(bind_addr)?;
            }

            socket.set_nodelay(true)?;

            let future = socket.connect(server_addr);
            let wait_for = wait_for.unwrap_or(CONNECT_TIMEOUT);
            match tokio::time::timeout(wait_for, future).await {
                Ok(Ok(socket)) => Ok(AsyncIoTokioAsStd(socket)),
                Ok(Err(e)) => Err(e),
                Err(_) => Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("connection to {server_addr:?} timed out after {wait_for:?}"),
                )),
            }
        })
    }

    fn bind_udp(
        &self,
        local_addr: SocketAddr,
        server_addr: SocketAddr,
    ) -> Pin<Box<dyn Send + Future<Output = io::Result<Self::Udp>>>> {
        let bind_interface = self.bind_interface.clone();

        Box::pin(async move {
            let domain = Domain::for_address(server_addr);
            let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;
            socket.set_nonblocking(true)?;
            if let Some(bind_interface) = bind_interface {
                bind_to_iface(&socket, server_addr, &bind_interface)?;
            }

            let sa = SockAddr::from(local_addr);
            socket.bind(&sa)?;

            // Safety: we know the socket is bound and non-blocking
            let std_sock = unsafe { UdpSocket::from_raw_fd(socket.into_raw_fd()) };
            tokio::net::UdpSocket::from_std(std_sock)
        })
    }
}

fn bind_to_iface(socket: &Socket, server_addr: SocketAddr, ifname: &str) -> io::Result<()> {
    let ifname = CString::new(ifname).map_err(|_| io::Error::other("invalid interface name"))?;
    let if_index = unsafe { libc::if_nametoindex(ifname.as_ptr()) };

    match NonZero::new(if_index) {
        Some(if_index) => match server_addr {
            SocketAddr::V4(_) => socket.bind_device_by_index_v4(Some(if_index)),
            SocketAddr::V6(_) => socket.bind_device_by_index_v6(Some(if_index)),
        },
        None => Err(io::Error::last_os_error()),
    }
}
