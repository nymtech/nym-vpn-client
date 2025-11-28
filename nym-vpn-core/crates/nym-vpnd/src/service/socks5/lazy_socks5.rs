//! Lazy SOCKS5 wrapper that initializes the Nym mixnet on first connection

use super::util::ConnectionGuard;
use nym_sdk::mixnet::{MixnetClientBuilder, Socks5, Socks5MixnetClient, StoragePaths};
use nym_vpn_lib_types::{TunnelConnectionData, TunnelState};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional},
    net::{TcpListener, TcpStream},
    sync::RwLock,
    time::{Instant, sleep},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

/// Configuration for the LazySocks5
#[derive(Debug, Clone)]
pub struct LazySocks5Config {
    /// Data directory for mixnet client state
    pub mixnet_data_path: PathBuf,
    /// Public SOCKS5 listen address (user-facing)
    pub listen_address: SocketAddr,
    /// Internal SOCKS5 address (from Nym SDK)
    pub internal_listen_address: SocketAddr,
    /// Request timeout duration
    pub request_timeout: Duration,
    /// Idle timeout duration
    pub idle_timeout: Duration,
    /// Exit node gateway address
    pub network_requester_address: String,
}

/// Errors from the LazySocks5
#[derive(Debug, thiserror::Error)]
pub enum LazySocks5Error {
    #[error("Failed to bind to public address {0}: {1}")]
    BindError(String, std::io::Error),

    #[error("Failed to connect to internal SOCKS5 server: {0}")]
    InternalConnectionError(std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Lazy SOCKS5 state
pub struct LazySocks5 {
    /// Configuration
    config: LazySocks5Config,
    /// Shared tunnel state
    tunnel_state_shared: Arc<RwLock<TunnelState>>,
    /// Cancellation token for shutdown
    cancel_token: CancellationToken,
    /// Active connection counter   
    active_connections: Arc<RwLock<u32>>,
    /// Last connection closed timestamp
    last_connection_closed: Arc<RwLock<Option<Instant>>>,
    /// Is mixnet running
    is_mixnet_running: Arc<RwLock<bool>>,
    /// Mixnet client
    mixnet_client: Arc<RwLock<Option<Socks5MixnetClient>>>,
}

impl LazySocks5 {
    /// Create a new lazy SOCKS5 wrapper
    pub fn new(
        config: LazySocks5Config,
        tunnel_state_shared: Arc<RwLock<TunnelState>>,
        cancel_token: CancellationToken,
    ) -> Result<Self, LazySocks5Error> {
        info!(
            "Creating LazySocks5: public={}, internal={}",
            config.listen_address.to_string(),
            config.internal_listen_address.to_string()
        );

        Ok(Self {
            config,
            tunnel_state_shared,
            cancel_token,
            active_connections: Arc::new(RwLock::new(0)),
            last_connection_closed: Arc::new(RwLock::new(None)),
            is_mixnet_running: Arc::new(RwLock::new(false)),
            mixnet_client: Arc::new(RwLock::new(None)),
        })
    }

    /// Run the lazy SOCKS5 wrapper
    pub async fn run(self: Arc<Self>) -> Result<(), LazySocks5Error> {
        let public_listen_address = self.config.listen_address.to_string();

        info!(
            "Starting lazy SOCKS5 wrapper on public address: {}",
            public_listen_address
        );

        // Bind to public port
        let listener = TcpListener::bind(&public_listen_address)
            .await
            .map_err(|e| LazySocks5Error::BindError(public_listen_address.clone(), e))?;

        info!("Listening on {}", self.config.listen_address.to_string());

        // Spawn idle timeout monitor
        let idle_monitor = self.clone();
        let idle_monitor_handle = tokio::spawn(async move {
            idle_monitor.monitor_idle_timeout().await;
        });

        // Spawn tunnel state monitor to shut down mixnet when dVPN is available
        let state_monitor = self.clone();
        let state_monitor_handle = tokio::spawn(async move {
            state_monitor.monitor_tunnel_state().await;
        });

        // Accept connections loop
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            debug!("Accepted connection from {}", addr);
                            let wrapper = self.clone();

                            // Check tunnel state to determine routing method
                            let tunnel_state = self.tunnel_state_shared.read().await.clone();
                            let use_dvpn = matches!(
                                tunnel_state,
                                TunnelState::Connected { ref connection_data }
                                if matches!(connection_data.tunnel, TunnelConnectionData::Mixnet(_))
                            );

                            // Spawn task to handle this connection
                            tokio::spawn(async move {
                                let result = if use_dvpn {
                                    info!("Routing connection from {} through dVPN tunnel", addr);
                                    wrapper.handle_connection_with_dvpn(stream, addr).await
                                } else {
                                    debug!("Routing connection from {} through mixnet", addr);
                                    Box::pin(wrapper.handle_connection_with_mixnet(stream, addr)).await
                                };

                                if let Err(e) = result {
                                    error!("Connection handler error for {}: {}", addr, e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept connection: {}", e);
                        }
                    }
                }
                _ = self.cancel_token.cancelled() => {
                    info!("Lazy SOCKS5 wrapper shutting down");
                    break;
                }
            }
        }

        // Clean up
        idle_monitor_handle.abort();
        state_monitor_handle.abort();
        self.shutdown_backend().await;

        info!("Lazy SOCKS5 wrapper stopped");
        Ok(())
    }

    /// Handle a single connection with dVPN tunnel (direct connection)
    async fn handle_connection_with_dvpn(
        &self,
        mut client_stream: TcpStream,
        client_addr: SocketAddr,
    ) -> Result<(), LazySocks5Error> {
        // Create connection guard - will automatically decrement on drop
        let _guard = ConnectionGuard::new(self.active_connections.clone()).await;

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
                Ipv6Addr::from(buf).to_string()
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

    /// Handle a single connection with mixnet
    async fn handle_connection_with_mixnet(
        &self,
        mut client_stream: TcpStream,
        client_addr: SocketAddr,
    ) -> Result<(), LazySocks5Error> {
        // Create connection guard - will automatically decrement on drop
        let _guard = ConnectionGuard::new(self.active_connections.clone()).await;

        // Ensure backend is started (lazy initialization) with retries
        if let Err(e) = Box::pin(self.ensure_backend_started_with_retry(client_addr)).await {
            error!("Failed to start backend for {}: {}", client_addr, e);
            // Send SOCKS5 error response
            let _ = Self::send_socks5_error(&mut client_stream).await;
            return Err(e);
        }

        // Connect to internal SOCKS5 server with retry logic
        // The internal server may take a moment to bind after backend initialization
        let internal_stream = match self.connect_to_internal_with_retry().await {
            Ok(stream) => stream,
            Err(e) => {
                error!(
                    "Failed to connect to internal SOCKS5 server at {}: {}",
                    self.config.internal_listen_address.to_owned(),
                    e
                );
                let _ = Self::send_socks5_error(&mut client_stream).await;
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

    /// Ensure the backend is started (lazy initialization)
    async fn ensure_backend_started(&self) -> Result<(), LazySocks5Error> {
        // Client already initialized
        if self.mixnet_client.read().await.is_some() {
            return Ok(());
        }

        info!("First connection detected, initializing Nym mixnet backend...");

        let mut socks5_config = Socks5::new(self.config.network_requester_address.clone());
        socks5_config.send_anonymously = true;
        socks5_config.bind_address = self.config.internal_listen_address;

        info!("Building mixnet client with SOCKS5 configuration...");

        // Create a custom StoragePaths that shares the credential database with the main VPN
        // but uses a separate identity by storing keys in a sibling "_socks5" directory
        let mixnet_folder_name = self
            .config
            .mixnet_data_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap();
        let socks5_data_path = self
            .config
            .mixnet_data_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(format!("{}_socks5", mixnet_folder_name));

        // Remove old socks5 directory if it exists
        // - to get fresh identity each time
        // - to not worry about version migrations
        if socks5_data_path.exists() {
            info!(
                "Removing old socks5 directory: {}",
                socks5_data_path.display()
            );
            tokio::fs::remove_dir_all(&socks5_data_path)
                .await
                .map_err(|e| {
                    error!("Failed to remove old socks5 directory: {}", e);
                    LazySocks5Error::Internal(format!(
                        "Failed to remove old socks5 directory: {}",
                        e
                    ))
                })?;
        }

        // Create fresh socks5 subdirectory
        tokio::fs::create_dir_all(&socks5_data_path)
            .await
            .map_err(|e| {
                error!("Failed to create socks5 data directory: {}", e);
                LazySocks5Error::Internal(format!("Failed to create socks5 data directory: {}", e))
            })?;

        info!("Created fresh socks5 directory for new identity");

        // Create base storage paths for the main VPN (to get the shared credential DB path)
        let main_storage_paths = StoragePaths::new_from_dir(&self.config.mixnet_data_path)
            .map_err(|e| {
                error!("Failed to create main storage paths: {}", e);
                LazySocks5Error::Internal(format!("Failed to create main storage paths: {}", e))
            })?;

        // Create storage paths for SOCKS5 identity
        let mut socks5_storage_paths =
            StoragePaths::new_from_dir(&socks5_data_path).map_err(|e| {
                error!("Failed to create socks5 storage paths: {}", e);
                LazySocks5Error::Internal(format!("Failed to create socks5 storage paths: {}", e))
            })?;

        // Override the credential database path to use the shared one from main VPN
        socks5_storage_paths.credential_database_path = main_storage_paths.credential_database_path;

        info!(
            "Using shared credential store: {}",
            socks5_storage_paths.credential_database_path.display()
        );
        info!(
            "Using separate identity keys in: {}",
            socks5_data_path.display()
        );

        // Build the mixnet client with shared credentials but different identity
        let mixnet_client: nym_sdk::mixnet::DisconnectedMixnetClient<
            nym_sdk::mixnet::OnDiskPersistent,
        > = MixnetClientBuilder::new_with_default_storage(socks5_storage_paths)
            .await
            .map_err(|e| {
                error!("Failed to create mixnet client builder: {}", e);
                LazySocks5Error::Internal(e.to_string())
            })?
            .socks5_config(socks5_config)
            .build()
            .map_err(|e| {
                error!("Failed to build mixnet client: {}", e);
                LazySocks5Error::Internal(e.to_string())
            })?;

        // Connect to the mixnet via SOCKS5
        info!("Connecting to mixnet via SOCKS5...");
        info!("This will spawn the internal SOCKS5 server and establish mixnet connection...");
        let mixnet_client = Box::pin(mixnet_client.connect_to_mixnet_via_socks5())
            .await
            .map_err(|e| {
                error!("Failed to connect to mixnet via SOCKS5: {}", e);
                LazySocks5Error::Internal(e.to_string())
            })?;

        info!("SOCKS5 mixnet backend connected successfully");
        info!("Client Nym address: {}", mixnet_client.nym_address());
        info!(
            "Internal SOCKS5 server should be listening on: {}",
            self.config.internal_listen_address.to_string()
        );

        *self.mixnet_client.write().await = Some(mixnet_client);

        // Give the internal SOCKS5 server a moment to fully bind
        sleep(Duration::from_millis(100)).await;
        info!("Backend initialization complete");

        Ok(())
    }

    /// Ensure backend is started with retry logic
    /// Retries backend initialization in case of transient failures (e.g., DNS timeouts)
    async fn ensure_backend_started_with_retry(
        &self,
        client_addr: SocketAddr,
    ) -> Result<(), LazySocks5Error> {
        const MAX_RETRIES: u32 = 3;
        const INITIAL_RETRY_DELAY: Duration = Duration::from_millis(500);

        let mut last_error = None;
        for attempt in 1..=MAX_RETRIES {
            match Box::pin(self.ensure_backend_started()).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    error!(
                        "Failed to start backend for {} (attempt {}/{}): {}",
                        client_addr, attempt, MAX_RETRIES, e
                    );
                    last_error = Some(e);

                    if attempt < MAX_RETRIES {
                        let delay = INITIAL_RETRY_DELAY * 2_u32.pow(attempt - 1);
                        info!("Retrying backend start in {:?}...", delay);
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Connect to internal SOCKS5 server with retry logic
    /// The internal server may take a moment to bind after backend initialization
    async fn connect_to_internal_with_retry(&self) -> Result<TcpStream, std::io::Error> {
        const MAX_RETRIES: u32 = 100;
        const RETRY_DELAY_MS: u64 = 50;

        let start = Instant::now();
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            // Check if we've exceeded total timeout
            if start.elapsed() > self.config.request_timeout {
                debug!(
                    "Timeout waiting for internal SOCKS5 server after {:?}",
                    start.elapsed()
                );
                break;
            }

            match TcpStream::connect(self.config.internal_listen_address).await {
                Ok(stream) => {
                    if attempt > 0 {
                        info!(
                            "Connected to internal SOCKS5 server after {} attempts ({:?})",
                            attempt + 1,
                            start.elapsed()
                        );
                    } else {
                        debug!("Connected to internal SOCKS5 server on first attempt");
                    }
                    *self.is_mixnet_running.write().await = true;
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

    /// Monitor idle timeout and shut down backend when idle
    async fn monitor_idle_timeout(&self) {
        loop {
            // Wait a bit before checking
            sleep(Duration::from_secs(5)).await;

            // Check if we should shut down
            let should_shutdown = {
                let count = self.active_connections.read().await;
                if *count > 0 {
                    // Still have active connections, reset timer
                    let mut last_closed = self.last_connection_closed.write().await;
                    *last_closed = None;
                    false
                } else {
                    // No active connections
                    let mut last_closed = self.last_connection_closed.write().await;
                    if last_closed.is_none() {
                        // Just became idle, start timer
                        *last_closed = Some(Instant::now());
                        false
                    } else {
                        // Check if timeout elapsed
                        let elapsed = last_closed.unwrap().elapsed();
                        elapsed >= self.config.idle_timeout
                    }
                }
            };

            debug!("should_shutdown: {}", should_shutdown);

            if should_shutdown {
                info!(
                    "Idle timeout of {:?} reached, shutting down backend",
                    self.config.idle_timeout
                );
                self.shutdown_backend().await;

                // Reset timer
                let mut last_closed = self.last_connection_closed.write().await;
                *last_closed = None;
            }

            // Check for cancellation
            if self.cancel_token.is_cancelled() {
                break;
            }
        }
    }

    /// Monitor tunnel state and shut down mixnet backend when dVPN is available
    async fn monitor_tunnel_state(&self) {
        let mut last_dvpn_available = false;

        loop {
            // Wait a bit before checking
            sleep(Duration::from_secs(2)).await;

            // Check if dVPN is currently available
            let dvpn_available = {
                let tunnel_state = self.tunnel_state_shared.read().await;
                matches!(
                    *tunnel_state,
                    TunnelState::Connected { ref connection_data }
                    if matches!(connection_data.tunnel, TunnelConnectionData::Mixnet(_))
                )
            };

            // React to state transitions
            if dvpn_available && !last_dvpn_available {
                // dVPN just became available - shut down mixnet backend
                info!(
                    "dVPN tunnel is now available, shutting down mixnet SOCKS5 backend to save bandwidth"
                );
                self.shutdown_backend().await;
            } else if !dvpn_available && last_dvpn_available {
                // dVPN just became unavailable
                info!(
                    "dVPN tunnel is no longer available, mixnet SOCKS5 backend will be lazily initialized on next connection"
                );
            }

            last_dvpn_available = dvpn_available;

            // Check for cancellation
            if self.cancel_token.is_cancelled() {
                break;
            }
        }
    }

    /// Shut down the backend
    async fn shutdown_backend(&self) {
        let mut client_guard = self.mixnet_client.write().await;
        if let Some(mixnet_client) = client_guard.take() {
            info!("Shutting down Nym mixnet client");
            *self.is_mixnet_running.write().await = false;
            mixnet_client.disconnect().await;
        }
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

    /// Get the number of active connections
    pub async fn active_connections(&self) -> u32 {
        *self.active_connections.read().await
    }

    /// Is mixnet running
    pub async fn is_mixnet_running(&self) -> bool {
        *self.is_mixnet_running.read().await
    }

    /// Get the public listen address
    pub fn public_address(&self) -> SocketAddr {
        self.config.listen_address
    }
}
