// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! SOCKS5 connection handling: the handshake, and proxying through the existing mixnet tunnel or the
//! lazily started mixnet client.

use super::{LazySocks5, LazySocks5Error, SharedState};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional},
    net::TcpStream,
    time::{Instant, sleep, timeout},
};
use tracing::{debug, error, info, warn};

const REJECT_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);

impl LazySocks5 {
    /// Handle the SOCKS5 connection via the existing mixnet tunnel
    pub async fn route_via_existing_tunnel(
        mut client_stream: TcpStream,
        client_addr: SocketAddr,
    ) -> Result<(), LazySocks5Error> {
        debug!("Routing connection from {client_addr} via existing Mixnet tunnel");

        // Parse SOCKS5 handshake and request
        let target_addr = match Self::socks5_handshake(&mut client_stream).await {
            Ok(addr) => addr,
            Err(e) => {
                error!("SOCKS5 handshake failed for {}: {}", client_addr, e);
                return Err(e);
            }
        };

        debug!(
            "dVPN: Connecting from {} to target {}",
            client_addr, target_addr
        );

        // Connect directly to the target (routes through dVPN tunnel)
        // DNS resolution happens through the VPN tunnel, preserving privacy
        let target_stream = match TcpStream::connect(&target_addr).await {
            Ok(stream) => stream,
            Err(e) => {
                error!(
                    "Failed to connect to target {} from {}: {}",
                    target_addr, client_addr, e
                );
                // Send SOCKS5 error response with a dummy bind address
                let reply_code = if e.kind() == std::io::ErrorKind::ConnectionRefused {
                    0x05 // Connection refused
                } else if e.kind() == std::io::ErrorKind::TimedOut {
                    0x06 // TTL expired
                } else {
                    0x04 // Host unreachable
                };
                // Use unspecified address for error responses
                let dummy_addr = SocketAddr::from(([0, 0, 0, 0], 0));
                let _ = Self::send_socks5_reply(&mut client_stream, reply_code, dummy_addr).await;
                return Err(LazySocks5Error::Internal(format!(
                    "Failed to connect to target: {}",
                    e
                )));
            }
        };

        debug!(
            "dVPN: Successfully connected from {} to {}",
            client_addr, target_addr
        );

        // Get the local address of the established connection for SOCKS5 reply
        let bind_addr = target_stream
            .local_addr()
            .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)));

        // Send SOCKS5 success response with the actual bind address
        if let Err(e) = Self::send_socks5_reply(&mut client_stream, 0x00, bind_addr).await {
            error!(
                "Failed to send SOCKS5 success response to {}: {}",
                client_addr, e
            );
            return Err(LazySocks5Error::Internal(format!(
                "Failed to send SOCKS5 response: {}",
                e
            )));
        }

        // Proxy bidirectionally
        let mut target_stream = target_stream;
        match copy_bidirectional(&mut client_stream, &mut target_stream).await {
            Ok((client_to_target, target_to_client)) => {
                debug!(
                    "dVPN connection from {} to {} closed: {}↑ {}↓",
                    client_addr, target_addr, client_to_target, target_to_client
                );
            }
            Err(e) => {
                debug!(
                    "dVPN proxy error for {} to {}: {}",
                    client_addr, target_addr, e
                );
            }
        }

        Ok(())
    }

    /// Handle the SOCKS5 connection via a new mixnet tunnel
    pub async fn route_via_new_tunnel(
        shared: &SharedState,
        mut client_stream: TcpStream,
        client_addr: SocketAddr,
    ) -> Result<(), LazySocks5Error> {
        debug!("Routing connection from {client_addr} via new Mixnet tunnel");

        // Ensure backend is started (lazy initialization) with retries
        if let Err(e) = Box::pin(Self::ensure_backend_started_with_retry(shared, client_addr)).await
        {
            error!("Failed to start backend for {}: {}", client_addr, e);
            Self::reject_connection(&mut client_stream).await;
            return Err(e);
        }

        // Connect to internal SOCKS5 server with retry logic
        // The internal server may take a moment to bind after backend initialization
        let internal_stream = match Self::connect_to_internal_with_retry(shared).await {
            Ok(stream) => stream,
            Err(e) => {
                error!(
                    "Failed to connect to internal SOCKS5 server at {}: {}",
                    shared.config.internal_listen_address.to_owned(),
                    e
                );
                Self::reject_connection(&mut client_stream).await;
                return Err(LazySocks5Error::InternalConnectionError(e));
            }
        };

        debug!(
            "Proxying connection from {} to internal SOCKS5 server",
            client_addr
        );

        // Proxy bidirectionally
        let mut internal_stream = internal_stream;
        match copy_bidirectional(&mut client_stream, &mut internal_stream).await {
            Ok((client_to_server, server_to_client)) => {
                debug!(
                    "Connection from {} closed: {}↑ {}↓",
                    client_addr, client_to_server, server_to_client
                );
            }
            Err(e) => {
                debug!("Proxy error for {}: {}", client_addr, e);
            }
        }

        Ok(())
    }

    /// Read the client's SOCKS5 handshake and answer the request with a general failure
    pub async fn reject_connection(stream: &mut TcpStream) {
        let _ = timeout(REJECT_HANDSHAKE_TIMEOUT, Self::socks5_handshake(stream)).await;
        let _ = Self::send_socks5_error(stream).await;
    }

    /// Send a SOCKS5 general server failure error
    async fn send_socks5_error(stream: &mut TcpStream) -> Result<(), std::io::Error> {
        // SOCKS5 response: version 5, general failure (0x01), reserved, address type 1 (IPv4)
        // Followed by 4 bytes of zeros for address and 2 bytes for port
        let error_response = [0x05, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        stream.write_all(&error_response).await?;
        stream.flush().await?;
        Ok(())
    }

    /// Perform SOCKS5 handshake and parse the target address
    /// Returns a string in "host:port" format to allow DNS resolution through the VPN tunnel
    async fn socks5_handshake(stream: &mut TcpStream) -> Result<String, LazySocks5Error> {
        // Read version and number of auth methods
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf).await.map_err(|e| {
            LazySocks5Error::Internal(format!("Failed to read SOCKS5 version: {}", e))
        })?;

        let version = buf[0];
        let nmethods = buf[1];

        if version != 0x05 {
            return Err(LazySocks5Error::Internal(format!(
                "Unsupported SOCKS version: {}",
                version
            )));
        }

        // Read auth methods
        let mut methods = vec![0u8; nmethods as usize];
        stream.read_exact(&mut methods).await.map_err(|e| {
            LazySocks5Error::Internal(format!("Failed to read auth methods: {}", e))
        })?;

        // Respond with "no authentication required" (0x00)
        stream.write_all(&[0x05, 0x00]).await.map_err(|e| {
            LazySocks5Error::Internal(format!("Failed to send auth response: {}", e))
        })?;

        // Read the CONNECT request
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).await.map_err(|e| {
            LazySocks5Error::Internal(format!("Failed to read SOCKS5 request: {}", e))
        })?;

        let version = buf[0];
        let cmd = buf[1];
        let _reserved = buf[2];
        let atyp = buf[3];

        if version != 0x05 {
            return Err(LazySocks5Error::Internal(format!(
                "Invalid SOCKS version in request: {}",
                version
            )));
        }

        if cmd != 0x01 {
            // Only CONNECT command is supported
            return Err(LazySocks5Error::Internal(format!(
                "Unsupported SOCKS command: {}",
                cmd
            )));
        }

        // Parse destination address based on address type
        let host: String = match atyp {
            0x01 => {
                // IPv4
                let mut buf = [0u8; 4];
                stream.read_exact(&mut buf).await.map_err(|e| {
                    LazySocks5Error::Internal(format!("Failed to read IPv4 address: {}", e))
                })?;
                Ipv4Addr::from(buf).to_string()
            }
            0x03 => {
                // Domain name - DO NOT resolve locally to preserve privacy!
                // Let TcpStream::connect handle DNS through the VPN tunnel
                let mut len_buf = [0u8; 1];
                stream.read_exact(&mut len_buf).await.map_err(|e| {
                    LazySocks5Error::Internal(format!("Failed to read domain length: {}", e))
                })?;
                let domain_len = len_buf[0] as usize;

                let mut domain_buf = vec![0u8; domain_len];
                stream.read_exact(&mut domain_buf).await.map_err(|e| {
                    LazySocks5Error::Internal(format!("Failed to read domain name: {}", e))
                })?;

                String::from_utf8(domain_buf)
                    .map_err(|e| LazySocks5Error::Internal(format!("Invalid domain name: {}", e)))?
            }
            0x04 => {
                // IPv6
                let mut buf = [0u8; 16];
                stream.read_exact(&mut buf).await.map_err(|e| {
                    LazySocks5Error::Internal(format!("Failed to read IPv6 address: {}", e))
                })?;
                // IPv6 literals need brackets so the port separator is unambiguous
                format!("[{}]", Ipv6Addr::from(buf))
            }
            _ => {
                return Err(LazySocks5Error::Internal(format!(
                    "Unsupported address type: {}",
                    atyp
                )));
            }
        };

        // Read port
        let mut port_buf = [0u8; 2];
        stream
            .read_exact(&mut port_buf)
            .await
            .map_err(|e| LazySocks5Error::Internal(format!("Failed to read port: {}", e)))?;
        let port = u16::from_be_bytes(port_buf);

        // Return "host:port" format - TcpStream::connect will handle DNS resolution through VPN
        Ok(format!("{}:{}", host, port))
    }

    /// Send SOCKS5 reply
    async fn send_socks5_reply(
        stream: &mut TcpStream,
        reply_code: u8,
        bind_addr: SocketAddr,
    ) -> Result<(), LazySocks5Error> {
        // SOCKS5 reply: VER | REP | RSV | ATYP | BND.ADDR | BND.PORT
        let mut response = vec![0x05, reply_code, 0x00];

        match bind_addr.ip() {
            IpAddr::V4(ipv4) => {
                response.push(0x01); // IPv4
                response.extend_from_slice(&ipv4.octets());
            }
            IpAddr::V6(ipv6) => {
                response.push(0x04); // IPv6
                response.extend_from_slice(&ipv6.octets());
            }
        }

        response.extend_from_slice(&bind_addr.port().to_be_bytes());

        stream.write_all(&response).await.map_err(|e| {
            LazySocks5Error::Internal(format!("Failed to send SOCKS5 reply: {}", e))
        })?;

        Ok(())
    }

    /// Connect to internal SOCKS5 server with retry logic
    /// The internal server may take a moment to bind after backend initialization
    async fn connect_to_internal_with_retry(
        shared: &SharedState,
    ) -> Result<TcpStream, std::io::Error> {
        const MAX_RETRIES: u32 = 100;
        const RETRY_DELAY_MS: u64 = 50;

        let start = Instant::now();
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            // Check if we've exceeded total timeout
            if start.elapsed() > shared.config.request_timeout {
                debug!(
                    "Timeout waiting for internal SOCKS5 server after {:?}",
                    start.elapsed()
                );
                break;
            }

            match TcpStream::connect(shared.config.internal_listen_address).await {
                Ok(stream) => {
                    // Configure TCP options for better performance
                    if let Err(e) = stream.set_nodelay(true) {
                        warn!("Failed to set TCP_NODELAY for internal connection: {}", e);
                    }
                    if attempt > 0 {
                        info!(
                            "Connected to internal SOCKS5 server after {} attempts ({:?})",
                            attempt + 1,
                            start.elapsed()
                        );
                    } else {
                        debug!("Connected to internal SOCKS5 server on first attempt");
                    }
                    return Ok(stream);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < MAX_RETRIES - 1 {
                        // Only log after a few attempts to avoid spam
                        if attempt > 0 && attempt % 10 == 0 {
                            debug!(
                                "Internal SOCKS5 server not ready yet (attempt {}/{}), retrying...",
                                attempt + 1,
                                MAX_RETRIES
                            );
                        }
                        sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Timeout waiting for internal SOCKS5 server",
            )
        }))
    }
}
