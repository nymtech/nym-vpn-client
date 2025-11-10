// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Module implementing TCP connection probe

use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

#[cfg(unix)]
use std::os::fd::{FromRawFd, IntoRawFd};
#[cfg(windows)]
use std::os::windows::io::{FromRawSocket, IntoRawSocket};

use socket2::{Domain, SockAddr, Socket, Type};
use tokio::net::TcpSocket;

use super::{
    BoxedProbeError, ConnectionProbe, DEFAULT_IPV4_PROBE_IP, DEFAULT_IPV6_PROBE_IP,
    DEFAULT_TCP_PROBE_PORT, ProbeError,
};

/// TCP probe configuration
#[derive(Debug, Clone)]
pub struct TcpProbeConfig {
    /// Probe IP address and port
    pub probe_address: SocketAddr,

    /// Bind socket to local address
    pub local_address: Option<SocketAddr>,

    /// Bind socket to interface
    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "ios",
        target_os = "macos"
    ))]
    pub interface: Option<String>,
}

impl TcpProbeConfig {
    /// Returns default configuration for probing over IPv4
    pub fn default_v4() -> Self {
        Self::new(SocketAddr::new(
            IpAddr::from(DEFAULT_IPV4_PROBE_IP),
            DEFAULT_TCP_PROBE_PORT,
        ))
    }

    /// Returns default configuration for probing over IPv6
    pub fn default_v6() -> Self {
        Self::new(SocketAddr::new(
            IpAddr::from(DEFAULT_IPV6_PROBE_IP),
            DEFAULT_TCP_PROBE_PORT,
        ))
    }

    /// Create new configuration with the given probe IP address and port
    pub fn new(probe_address: impl Into<SocketAddr>) -> Self {
        Self {
            probe_address: probe_address.into(),
            local_address: None,
            #[cfg(any(
                target_os = "linux",
                target_os = "android",
                target_os = "ios",
                target_os = "macos"
            ))]
            interface: None,
        }
    }

    /// Bind socket to local address
    pub fn with_local_address(mut self, local_addr: SocketAddr) -> Self {
        self.local_address = Some(local_addr);
        self
    }

    /// Bind socket to interface
    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "ios",
        target_os = "macos"
    ))]
    pub fn with_interface(mut self, interface: String) -> Self {
        self.interface = Some(interface);
        self
    }
}

// The probe that utilizes TCP handshake to determine if the connection is viable
pub struct TcpProbe {
    config: TcpProbeConfig,
}

impl TcpProbe {
    pub fn new(config: TcpProbeConfig) -> Result<TcpProbe, TcpProbeError> {
        let tcp_probe = Self {
            config: config.clone(),
        };

        // Test socket without connecting it
        tcp_probe.create_socket()?;

        Ok(tcp_probe)
    }

    fn create_socket(&self) -> Result<TcpSocket, TcpProbeError> {
        let domain = match self.config.probe_address {
            SocketAddr::V4(_) => Domain::IPV4,
            SocketAddr::V6(_) => Domain::IPV6,
        };

        let socket = Socket::new(domain, Type::STREAM, None)
            .map_err(|err| TcpProbeError::from(TcpProbeInnerError::CreateTcpSocket(err)))?;
        socket
            .set_nonblocking(true)
            .map_err(|err| TcpProbeError::from(TcpProbeInnerError::SetNonblocking(err)))?;

        if let Some(local_address) = self.config.local_address {
            let sockaddr = SockAddr::from(local_address);
            socket
                .bind(&sockaddr)
                .map_err(|err| TcpProbeError::from(TcpProbeInnerError::BindAddress(err)))?;
        }

        #[cfg(any(target_os = "android", target_os = "linux"))]
        if let Some(interface) = self.config.interface.as_deref() {
            socket
                .bind_device(Some(interface.as_bytes()))
                .map_err(|err| TcpProbeError::from(TcpProbeInnerError::BindInterface(err)))?;
        }

        #[cfg(any(target_os = "ios", target_os = "macos"))]
        if let Some(interface) = self.config.interface.as_deref() {
            // Convert interface name to index
            let index = nix::net::if_::if_nametoindex(interface)
                .map_err(|err| TcpProbeInnerError::GetInterfaceIndex(interface.to_owned(), err))
                .and_then(|index| {
                    std::num::NonZeroU32::new(index).ok_or(
                        TcpProbeInnerError::InvalidInterfaceIndex(interface.to_owned()),
                    )
                })?;
            match self.config.probe_address {
                SocketAddr::V4(_) => socket.bind_device_by_index_v4(Some(index)),
                SocketAddr::V6(_) => socket.bind_device_by_index_v6(Some(index)),
            }
            .map_err(TcpProbeInnerError::BindInterface)?;
        }

        #[cfg(windows)]
        let socket = unsafe { TcpSocket::from_raw_socket(socket.into_raw_socket()) };
        #[cfg(unix)]
        let socket = unsafe { TcpSocket::from_raw_fd(socket.into_raw_fd()) };

        Ok(socket)
    }
}

/// Public error type for the TCP probe.
#[derive(Debug)]
pub struct TcpProbeError {
    inner: TcpProbeInnerError,
}

impl TcpProbeError {
    pub fn inner(&self) -> &(dyn std::error::Error + 'static) {
        &self.inner
    }
}

impl std::fmt::Display for TcpProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for TcpProbeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl From<TcpProbeInnerError> for TcpProbeError {
    fn from(value: TcpProbeInnerError) -> Self {
        Self { inner: value }
    }
}

/// Private error type for the ICMP probe.
#[derive(Debug, thiserror::Error)]
enum TcpProbeInnerError {
    #[error("failed to create tcp socket")]
    CreateTcpSocket(#[source] std::io::Error),

    #[error("failed to bind tcp socket to address")]
    BindAddress(#[source] std::io::Error),

    #[cfg(any(
        target_os = "android",
        target_os = "linux",
        target_os = "ios",
        target_os = "macos"
    ))]
    #[error("failed to bind tcp socket to interface")]
    BindInterface(#[source] std::io::Error),

    #[error("failed to set non-blocking mode for tcp socket")]
    SetNonblocking(#[source] std::io::Error),

    #[error("TCP timeout")]
    Timeout,

    #[error("Failed to establish TCP connection")]
    EstablishConnection(#[source] std::io::Error),

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[error("failed to get interface index for {0}")]
    GetInterfaceIndex(String, #[source] nix::errno::Errno),

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[error("received invalid interface index for {0}")]
    InvalidInterfaceIndex(String),
}

impl ProbeError for TcpProbeError {
    fn is_timeout(&self) -> bool {
        matches!(self.inner, TcpProbeInnerError::Timeout)
    }
}

impl From<TcpProbeError> for BoxedProbeError {
    fn from(value: TcpProbeError) -> Self {
        BoxedProbeError(Box::new(value))
    }
}

#[async_trait::async_trait]
impl ConnectionProbe for TcpProbe {
    async fn send(&self, timeout: Duration) -> Result<(), BoxedProbeError> {
        let socket = self.create_socket()?;

        let _tcp_stream = tokio::time::timeout(timeout, socket.connect(self.config.probe_address))
            .await
            .map_err(|_timeout_err| TcpProbeError::from(TcpProbeInnerError::Timeout))?
            .map_err(|err| TcpProbeError::from(TcpProbeInnerError::EstablishConnection(err)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use super::*;

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    use tokio_util::sync::CancellationToken;

    const LOOPBACK_V4: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
    const LOOPBACK_V6: SocketAddr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0);

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_send_loopback_tcp_v4() {
        let shutdown_token = CancellationToken::new();
        let server_addr = start_tcp_listener(LOOPBACK_V4, shutdown_token.child_token()).await;
        let _drop_guard = shutdown_token.drop_guard();

        let probe = TcpProbe::new(get_config(server_addr, LOOPBACK_V4)).unwrap();
        probe
            .send(Duration::from_millis(100))
            .await
            .expect("failed to send probe");
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_send_loopback_tcp_v6() {
        let shutdown_token = CancellationToken::new();
        let server_addr = start_tcp_listener(LOOPBACK_V6, shutdown_token.child_token()).await;
        let _drop_guard = shutdown_token.drop_guard();

        let probe = TcpProbe::new(get_config(server_addr, LOOPBACK_V6)).unwrap();
        probe
            .send(Duration::from_millis(100))
            .await
            .expect("failed to send probe");
    }

    fn get_config(probe_address: SocketAddr, _local_addr: SocketAddr) -> TcpProbeConfig {
        let mut config = TcpProbeConfig::new(probe_address);
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            config = config.with_interface("lo".to_owned());
        }
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        {
            config = config.with_interface("lo0".to_owned());
        }
        #[cfg(not(any(
            target_os = "linux",
            target_os = "android",
            target_os = "ios",
            target_os = "macos"
        )))]
        {
            config = config.with_local_address(_local_addr);
        }
        config
    }

    async fn start_tcp_listener(
        bind_addr: SocketAddr,
        shutdown_token: CancellationToken,
    ) -> SocketAddr {
        let listener = TcpListener::bind(bind_addr).await.unwrap();
        let local_addr = listener.local_addr().unwrap();
        tracing::info!("Listening on {local_addr}");
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    incoming = listener.accept() => {
                        match incoming {
                            Ok((mut socket, _addr)) => {
                                let mut buf = [0; 1];
                                if let Err(err) = socket.read(&mut buf).await {
                                    tracing::error!("Error reading from socket: {}", err);
                                }
                                if let Err(err) = socket.write(&buf).await {
                                    tracing::error!("Error writing to socket: {}", err);
                                }
                            }
                            Err(err) => {
                                tracing::error!("Error accepting connection: {}", err);
                            }
                        }
                    }
                    _ = shutdown_token.cancelled() => {
                        break;
                    }
                }
            }
        });
        local_addr
    }
}
