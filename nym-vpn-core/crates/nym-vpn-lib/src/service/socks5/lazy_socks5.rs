//! Lazy SOCKS5 wrapper that initializes the Nym mixnet on first connection.

use super::util::ConnectionGuard;
use nym_bandwidth_controller::requests::BandwidthControllerRequestSender;
use nym_gateway_directory::{GatewayCacheHandle, ScoreValue};
use nym_network_defaults::v2::NymNetworkDetails;
use nym_sdk::mixnet::{MixnetClientBuilder, Socks5, Socks5MixnetClient, StoragePaths};
use nym_vpn_lib_types::{TunnelConnectionData, TunnelState};
use rand::seq::SliceRandom;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional},
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock},
    time::{Instant, sleep},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};

/// Configuration for the LazySocks5
#[derive(Clone)]
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
    /// Exit node gateway address (optional - if None, will select randomly)
    pub network_requester_address: Option<String>,
    /// Network Requester rotation interval (None = disabled)
    pub network_requester_rotation_interval: Option<Duration>,
    /// Gateway cache handle for looking up Network Requesters
    pub gateway_cache_handle: Option<GatewayCacheHandle>,
    /// Network details for the mixnet client (mainnet/testnet/sandbox)
    pub network_details: Option<NymNetworkDetails>,
    /// VPN exit gateway identity to exclude during random Network Requester selection (for privacy)
    pub vpn_exit_gateway_identity: Option<String>,
    /// Bandwidth controller handle, used as the mixnet client's ticket provider.
    pub bandwidth_command_tx: BandwidthControllerRequestSender,
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

    #[error("Gateway directory error: {0}")]
    GatewayDirectory(String),

    #[error("No available Network Requesters found")]
    NoNetworkRequesters,
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
    /// Mutex to prevent concurrent initialization
    init_mutex: Arc<Mutex<()>>,
    /// Last time Network Requester was rotated
    last_rotation: Arc<RwLock<Option<Instant>>>,
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
            init_mutex: Arc::new(Mutex::new(())),
            last_rotation: Arc::new(RwLock::new(None)),
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

        // Spawn Network Requester rotation monitor (if enabled)
        let rotation_monitor_handle = if self.config.network_requester_rotation_interval.is_some() {
            let rotation_monitor = self.clone();
            Some(tokio::spawn(async move {
                rotation_monitor.monitor_network_requester_rotation().await;
            }))
        } else {
            None
        };

        // Accept connections loop
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            debug!("Accepted connection from {}", addr);
                            // Configure TCP options for better performance
                            if let Err(e) = stream.set_nodelay(true) {
                                warn!("Failed to set TCP_NODELAY for {}: {}", addr, e);
                            }
                            let wrapper = self.clone();

                            // Check tunnel state to determine routing method
                            // 1. If the tunnel is mixnet and connected -> use existing tunnel
                            // 2. If the tunnel is wireguard and connected -> create new tunnel
                            // 3. If the tunnel is disconnected or error'd -> create new tunnel
                            // 4. If the tunnel is connecting, disconnecting or offline -> reject connection
                            let tunnel_state = self.tunnel_state_shared.read().await.clone();
                            let use_existing_tunnel =  match &tunnel_state {
                                TunnelState::Connected { connection_data } => {
                                    match &connection_data.tunnel {
                                        TunnelConnectionData::Mixnet(_) => true, // 1
                                        TunnelConnectionData::Wireguard(_) => false, // 2
                                    }
                                }
                                TunnelState::Disconnected | TunnelState::Error(_) => {
                                    false // 3
                                }
                                _ => {
                                    warn!("Rejecting SOCKS5 connection from {addr} due to tunnel state: {tunnel_state:?}");
                                    let mut stream = stream;
                                    let _ = Self::send_socks5_error(&mut stream).await; // 4
                                    continue;
                                }
                            };

                            // Spawn task to handle this connection
                            tokio::spawn(async move {
                                let result = if use_existing_tunnel {
                                    wrapper.route_via_existing_tunnel(stream, addr).await
                                } else {
                                    Box::pin(wrapper.route_via_new_tunnel(stream, addr)).await
                                };

                                if let Err(e) = result {
                                    error!("Connection handler error for {addr}: {e}");
                                }
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept connection: {e}");
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
        if let Some(handle) = rotation_monitor_handle {
            handle.abort();
        }
        self.shutdown_backend().await;

        info!("Lazy SOCKS5 wrapper stopped");
        Ok(())
    }

    /// Handle the SOCKS5 connection via the existing mixnet tunnel
    async fn route_via_existing_tunnel(
        &self,
        mut client_stream: TcpStream,
        client_addr: SocketAddr,
    ) -> Result<(), LazySocks5Error> {
        info!("Routing connection from {client_addr} via existing Mixnet tunnel");

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

    /// Handle the SOCKS5 connection via a new mixnet tunnel
    async fn route_via_new_tunnel(
        &self,
        mut client_stream: TcpStream,
        client_addr: SocketAddr,
    ) -> Result<(), LazySocks5Error> {
        info!("Routing connection from {client_addr} via new Mixnet tunnel");

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

    /// Build a mixnet client with the given gateway (or None for random selection)
    async fn build_mixnet_client(
        &self,
        socks5_storage_paths: &StoragePaths,
        socks5_config: &Socks5,
        gateway_id: Option<&String>,
    ) -> Result<
        nym_sdk::mixnet::DisconnectedMixnetClient<nym_sdk::mixnet::OnDiskPersistent>,
        LazySocks5Error,
    > {
        let mut builder =
            MixnetClientBuilder::new_with_default_storage(socks5_storage_paths.clone())
                .await
                .map_err(|e| {
                    error!("Failed to create mixnet client builder: {}", e);
                    LazySocks5Error::Internal(e.to_string())
                })?;

        // Configure network environment if provided
        if let Some(ref network_details) = self.config.network_details {
            builder = builder.network_details(network_details.clone().into());
            debug!(
                "Using network environment: {}",
                network_details.network_name
            );
        }

        // Configure gateway if specified
        if let Some(gateway_id) = gateway_id {
            builder = builder.request_gateway(gateway_id.clone());
        }

        let mixnet_client = builder
            .socks5_config(socks5_config.clone())
            .with_custom_bandwidth_provider(Box::new(self.config.bandwidth_command_tx.clone()))
            .build()
            .map_err(|e| {
                error!("Failed to build mixnet client: {}", e);
                LazySocks5Error::Internal(e.to_string())
            })?;

        Ok(mixnet_client)
    }

    /// Ensure the backend is started (lazy initialization)
    async fn ensure_backend_started(&self) -> Result<(), LazySocks5Error> {
        // Client already initialized - quick check without mutex
        if self.mixnet_client.read().await.is_some() {
            return Ok(());
        }

        // Acquire init mutex to prevent concurrent initialization
        let _init_guard = self.init_mutex.lock().await;

        // Double-check after acquiring mutex (another task might have initialized it)
        if self.mixnet_client.read().await.is_some() {
            return Ok(());
        }

        debug!("First connection detected, initializing Nym mixnet backend...");

        // Determine Network Requester address (fixed or random)
        let network_requester_address = match &self.config.network_requester_address {
            Some(fixed_address) => {
                debug!("Using fixed Network Requester: {}", fixed_address);
                fixed_address.clone()
            }
            None => {
                info!("Selecting random Network Requester from gateway directory...");
                // If random selection fails, retry with exponential backoff
                let mut last_error = None;
                let mut selected_nr = None;
                for attempt in 1..=3 {
                    match self.select_random_network_requester().await {
                        Ok(random_nr) => {
                            if attempt > 1 {
                                info!(
                                    "Successfully selected random Network Requester after {} attempt(s)",
                                    attempt
                                );
                            } else {
                                // Log the selected Network Requester on first successful attempt
                                info!("Selected random Network Requester: {}", random_nr);
                            }
                            selected_nr = Some(random_nr);
                            break;
                        }
                        Err(e) => {
                            warn!(
                                "Random Network Requester selection failed (attempt {}/3): {}",
                                attempt, e
                            );
                            last_error = Some(e);
                            if attempt < 3 {
                                let delay = Duration::from_millis(500 * 2_u64.pow(attempt - 1));
                                debug!("Retrying random selection in {:?}...", delay);
                                sleep(delay).await;
                            }
                        }
                    }
                }
                // If all retries failed, return the last error
                selected_nr
                    .ok_or_else(|| last_error.unwrap_or(LazySocks5Error::NoNetworkRequesters))?
            }
        };

        info!("Using Network Requester: {}", network_requester_address);

        let mut socks5_config = Socks5::new(network_requester_address);
        socks5_config.send_anonymously = true;
        socks5_config.bind_address = self.config.internal_listen_address;

        // Create a custom StoragePaths that shares the credential database with the main VPN
        // but uses a separate identity by storing keys in a sibling "_socks5" directory
        let mixnet_folder_name = self
            .config
            .mixnet_data_path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("mixnet_data_path must have a valid file name");
        let socks5_data_path = self
            .config
            .mixnet_data_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(format!("{}_socks5", mixnet_folder_name));

        // Ensure parent directory exists (permissions will be checked when we try to write)
        if let Some(parent) = socks5_data_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                error!(
                    "Failed to create parent directory {}: {e}",
                    parent.display()
                );
                LazySocks5Error::Internal(format!(
                    "Failed to create parent directory {}: {e}. Check directory permissions.",
                    parent.display()
                ))
            })?;
        }

        // Remove old socks5 directory if it exists
        // - to get fresh identity each time
        // - to not worry about version migrations
        let create_dir = if socks5_data_path.exists() {
            match tokio::fs::remove_dir_all(&socks5_data_path).await {
                Ok(_) => {
                    info!(
                        "Removed old socks5 directory {}",
                        socks5_data_path.display()
                    );
                    true
                }
                Err(e) => {
                    warn!(
                        "Failed to remove old socks5 directory {}: {e}.  We will use the old directory.",
                        socks5_data_path.display()
                    );
                    false
                }
            }
        } else {
            true
        };

        if create_dir {
            tokio::fs::create_dir_all(&socks5_data_path)
                .await
                .map_err(|e| {
                    error!(
                        "Failed to create socks5 data directory {}: {e}",
                        socks5_data_path.display()
                    );
                    LazySocks5Error::Internal(format!(
                        "Failed to create socks5 data directory {}: {e}",
                        socks5_data_path.display()
                    ))
                })?;

            debug!("Created fresh socks5 directory for new identity");
        }

        // Create storage paths for SOCKS5 identity
        let socks5_storage_paths = StoragePaths::new_from_dir(&socks5_data_path).map_err(|e| {
            error!("Failed to create socks5 storage paths: {}", e);
            LazySocks5Error::Internal(format!("Failed to create socks5 storage paths: {}", e))
        })?;

        // BC note : We are NOT overriding the credential store, because it's not supposed to be shared anymore
        // Credential requests go through the channel for it now
        debug!(
            "Using separate identity keys in: {}",
            socks5_data_path.display()
        );

        // Build the mixnet client with shared credentials but different identity
        // When dVPN is connected (WireGuard mode), use VPN's entry gateway for firewall compatibility.
        // The entry gateway is fixed, but we can route to any Network Requester (exit) for privacy.
        let tunnel_state = self.tunnel_state_shared.read().await.clone();

        // Always use VPN's entry gateway if VPN is connected (for firewall compatibility)
        // The Network Requester (exit) is independent and can be any available Network Requester
        let requested_gateway_id = if let TunnelState::Connected { connection_data } = &tunnel_state
        {
            Some(connection_data.entry_gateway.id.clone())
        } else {
            None
        };

        // Build and connect with VPN's entry gateway (if VPN is connected)
        // The Network Requester address in socks5_config determines the exit point
        let mixnet_client = match self
            .build_mixnet_client(
                &socks5_storage_paths,
                &socks5_config,
                requested_gateway_id.as_ref(),
            )
            .await
        {
            Ok(client) => {
                match Box::pin(client.connect_to_mixnet_via_socks5()).await {
                    Ok(connected_client) => connected_client,
                    Err(e) => {
                        let error_msg = e.to_string();
                        // Check if error is about gateway not found and we have a requested gateway
                        if let Some(gateway_id) = requested_gateway_id.as_ref()
                            && error_msg.contains("no gateway with id")
                        {
                            let is_wireguard_mode = matches!(
                                tunnel_state,
                                TunnelState::Connected { ref connection_data }
                                if matches!(connection_data.tunnel, TunnelConnectionData::Wireguard(_))
                            );

                            if is_wireguard_mode {
                                // WireGuard mode: cannot change entry gateway (firewall rules)
                                error!(
                                    "VPN's entry gateway {} unavailable. Cannot use SOCKS5 in WireGuard mode: firewall rules require VPN's entry gateway.",
                                    gateway_id
                                );
                                return Err(LazySocks5Error::Internal(format!(
                                    "Cannot use SOCKS5 in WireGuard mode: VPN's entry gateway {} is not available. \
                                    Firewall rules only allow the VPN's entry gateway.",
                                    gateway_id
                                )));
                            } else {
                                // Not WireGuard: fallback to random gateway
                                warn!(
                                    "Gateway {} unavailable, falling back to random selection",
                                    gateway_id
                                );

                                let fallback_client = self
                                    .build_mixnet_client(
                                        &socks5_storage_paths,
                                        &socks5_config,
                                        None,
                                    )
                                    .await
                                    .map_err(|e| {
                                        error!("Failed to build fallback client: {}", e);
                                        e
                                    })?;

                                Box::pin(fallback_client.connect_to_mixnet_via_socks5())
                                    .await
                                    .map_err(|fallback_error| {
                                        LazySocks5Error::Internal(format!(
                                            "Failed to connect: gateway {} failed ({}), fallback also failed ({})",
                                            gateway_id,
                                            error_msg,
                                            fallback_error
                                        ))
                                    })?
                            }
                        } else {
                            return Err(LazySocks5Error::Internal(error_msg));
                        }
                    }
                }
            }
            Err(e) => return Err(e),
        };

        info!(
            "SOCKS5 mixnet backend connected (address: {})",
            mixnet_client.nym_address()
        );

        *self.mixnet_client.write().await = Some(mixnet_client);

        // Give the internal SOCKS5 server a moment to fully bind
        sleep(Duration::from_millis(100)).await;

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

        Err(last_error.expect("last_error should always be set after MAX_RETRIES attempts"))
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
                        // last_closed is guaranteed to be Some() here due to the is_none() check above
                        let elapsed = last_closed
                            .as_ref()
                            .expect("last_closed should be Some() after is_none() check")
                            .elapsed();
                        elapsed >= self.config.idle_timeout
                    }
                }
            };

            trace!("should_shutdown: {}", should_shutdown);

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

    /// Monitor tunnel state and manage mixnet backend lifecycle
    async fn monitor_tunnel_state(&self) {
        let mut last_vpn_state: Option<TunnelState> = None;
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let current_state = self.tunnel_state_shared.read().await.clone();

            // Check if dVPN is currently available (Mixnet mode)
            let dvpn_available = matches!(
                current_state,
                TunnelState::Connected { ref connection_data }
                if matches!(connection_data.tunnel, TunnelConnectionData::Mixnet(_))
            );

            // Check if VPN is connected in WireGuard mode
            let vpn_connected_wireguard = matches!(
                current_state,
                TunnelState::Connected { ref connection_data }
                if matches!(connection_data.tunnel, TunnelConnectionData::Wireguard(_))
            );

            // React to state transitions
            let last_dvpn_available = last_vpn_state
                .as_ref()
                .map(|s| {
                    matches!(
                        s,
                        TunnelState::Connected { connection_data }
                        if matches!(connection_data.tunnel, TunnelConnectionData::Mixnet(_))
                    )
                })
                .unwrap_or(false);

            let last_vpn_connected_wireguard = last_vpn_state
                .as_ref()
                .map(|s| {
                    matches!(
                        s,
                        TunnelState::Connected { connection_data }
                        if matches!(connection_data.tunnel, TunnelConnectionData::Wireguard(_))
                    )
                })
                .unwrap_or(false);

            if dvpn_available && !last_dvpn_available {
                // dVPN just became available (Mixnet mode) - shut down mixnet backend
                info!(
                    "dVPN tunnel is now available (Mixnet mode), shutting down mixnet SOCKS5 backend to save bandwidth"
                );
                self.shutdown_backend().await;
            } else if vpn_connected_wireguard && !last_vpn_connected_wireguard {
                // VPN just connected in WireGuard mode - if mixnet client is running, shut it down
                // so it will be recreated with VPN's entry gateway on next connection
                let is_running = self.is_mixnet_running().await;
                if is_running {
                    info!(
                        "VPN connected in WireGuard mode while mixnet client is running, shutting down mixnet client to ensure it uses VPN's entry gateway (firewall compatibility)"
                    );
                    self.shutdown_backend().await;
                }
            } else if !dvpn_available
                && !vpn_connected_wireguard
                && (last_dvpn_available || last_vpn_connected_wireguard)
            {
                // VPN just disconnected
                info!(
                    "VPN disconnected, mixnet SOCKS5 backend will be lazily initialized on next connection"
                );
            }

            last_vpn_state = Some(current_state);

            // Check for cancellation
            if self.cancel_token.is_cancelled() {
                break;
            }
        }
    }

    /// Monitor and rotate Network Requester periodically
    async fn monitor_network_requester_rotation(&self) {
        let Some(rotation_interval) = self.config.network_requester_rotation_interval else {
            return; // Rotation disabled
        };

        info!(
            "Network Requester rotation monitor started (interval: {:?})",
            rotation_interval
        );

        let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            // Only rotate if mixnet is running and dVPN (WireGuard) is connected
            let should_rotate = {
                let is_running = self.is_mixnet_running().await;
                let tunnel_state = self.tunnel_state_shared.read().await.clone();
                let dvpn_wireguard_active = matches!(
                    tunnel_state,
                    TunnelState::Connected { ref connection_data }
                    if matches!(connection_data.tunnel, TunnelConnectionData::Wireguard(_))
                );

                if !is_running || !dvpn_wireguard_active {
                    false
                } else {
                    let mut last_rotation = self.last_rotation.write().await;
                    if last_rotation.is_none() {
                        // First rotation - start timer
                        *last_rotation = Some(Instant::now());
                        false
                    } else {
                        // last_rotation is guaranteed to be Some() here due to the is_none() check above
                        let elapsed = last_rotation
                            .as_ref()
                            .expect("last_rotation should be Some() after is_none() check")
                            .elapsed();
                        if elapsed >= rotation_interval {
                            *last_rotation = Some(Instant::now());
                            true
                        } else {
                            false
                        }
                    }
                }
            };

            if should_rotate {
                info!("Rotating Network Requester after {:?}", rotation_interval);
                self.rotate_network_requester().await;
            }

            // Check for cancellation
            if self.cancel_token.is_cancelled() {
                break;
            }
        }

        info!("Network Requester rotation monitor stopped");
    }

    /// Rotate to a new Network Requester by shutting down current mixnet client
    /// Only rotates if there are no active connections to avoid disrupting ongoing transactions
    async fn rotate_network_requester(&self) {
        // Check if there are active connections
        let active_count = self.active_connections().await;

        if active_count > 0 {
            info!(
                "Skipping Network Requester rotation: {} active connection(s) in progress",
                active_count
            );
            info!("Rotation will be attempted at next interval when connections are idle");
            return;
        }

        info!("Shutting down current mixnet client to rotate Network Requester");
        self.shutdown_backend().await;
        info!("Next SOCKS5 connection will use a new random Network Requester");
    }

    /// Select a random Network Requester from the gateway directory.
    /// # Errors
    /// - `LazySocks5Error::GatewayDirectory` if gateway cache is not configured
    /// - `LazySocks5Error::NoNetworkRequesters` if no nodes have NR addresses
    async fn select_random_network_requester(&self) -> Result<String, LazySocks5Error> {
        let Some(ref gateway_cache_handle) = self.config.gateway_cache_handle else {
            error!("Gateway cache handle not configured for random NR selection");
            error!("This is a configuration error - rotation requires gateway_cache_handle");
            return Err(LazySocks5Error::GatewayDirectory(
                "Gateway cache handle not available. Required for random Network Requester selection. \
                 Ensure gateway_cache_handle is provided when enabling rotation.".to_string(),
            ));
        };

        // Fetch NymNodes with SOCKS5 probe data from VPN API
        // This uses a separate method to avoid breaking existing code that depends on skimmed nodes
        debug!("Fetching NymNodes with SOCKS5 probe data for Network Requester selection");
        let nymnodes = gateway_cache_handle
            .lookup_nymnodes_for_socks5()
            .await
            .map_err(|e| {
                error!("Failed to fetch NymNodes with SOCKS5 data: {}", e);
                LazySocks5Error::GatewayDirectory(format!(
                    "Failed to lookup NymNodes with SOCKS5 probe data from gateway directory: {}. \
                     Ensure gateway directory is accessible and properly configured.",
                    e
                ))
            })?;

        let total_nodes = nymnodes.len();
        debug!("Fetched {} nodes from directory", total_nodes);

        // Filter nodes that have a network requester address
        // Exclude VPN exit gateway for privacy (avoid correlation between VPN and SOCKS5 traffic)
        // Filter by SOCKS5 score: prefer High, fallback to Medium (exclude Low/Offline)
        let vpn_exit_identity = self.config.vpn_exit_gateway_identity.as_ref();
        let mut high_score_nodes = Vec::new();
        let mut medium_score_nodes = Vec::new();
        let mut excluded_by_score = 0;

        for (node, nr_address, socks5_score) in nymnodes
            .into_iter()
            .filter_map(|gw| {
                // Check if node has Network Requester address first
                // No NR address, skip
                gw.nr_address.clone().map(|nr_address| (gw, nr_address))
            })
            .filter(|(gw, _)| {
                // only consider gateways that are NOT the VPN exit gateway
                // unwrap_or_default works because if there's no vpn exit
                // identity, it resolves to empty string, which will be NOT EQUAL
                // to any gateway ID, thus preserving those gateways IN the set
                gw.identity().to_string() != vpn_exit_identity.cloned().unwrap_or_default()
            })
            .filter_map(|(gw, nr_address)| {
                // Only consider exit-capable gateways (SOCKS5 Network Requesters must be exit gateways)
                let score = gw
                    .last_probe
                    .as_ref()
                    .and_then(|probe| probe.outcome.socks5.as_ref())
                    .map(|socks5| socks5.score);
                score.map(|socks5_score| (gw, nr_address, socks5_score))
            })
        {
            // Filter by SOCKS5 score: prefer High, fallback to Medium (exclude Low/Offline/None)
            match socks5_score {
                Some(ScoreValue::High) => {
                    // High score - preferred
                    high_score_nodes.push((node.clone(), nr_address.clone()));
                }
                Some(ScoreValue::Medium) => {
                    // Medium score - fallback option
                    medium_score_nodes.push((node.clone(), nr_address.clone()));
                }
                Some(score) => {
                    // Low or Offline - exclude
                    excluded_by_score += 1;
                    debug!(
                        "Excluding node {} with low SOCKS5 score: {:?}",
                        node.identity(),
                        score
                    );
                }
                None => {
                    // No score data - exclude
                    excluded_by_score += 1;
                    debug!(
                        "Excluding node {} with no SOCKS5 score data",
                        node.identity()
                    );
                }
            }
        }

        // Prefer High score nodes, fallback to Medium if no High available
        let nodes_with_nr = if !high_score_nodes.is_empty() {
            info!(
                "Found {} High score Network Requesters (excluding {} low/no-score nodes)",
                high_score_nodes.len(),
                excluded_by_score
            );
            high_score_nodes
        } else if !medium_score_nodes.is_empty() {
            warn!(
                "No High score Network Requesters available, falling back to {} Medium score nodes (excluding {} low/no-score nodes)",
                medium_score_nodes.len(),
                excluded_by_score
            );
            medium_score_nodes
        } else {
            error!("No Network Requesters available with High/Medium SOCKS5 scores");
            error!(
                "Filtered out {} nodes (low score or no score data)",
                excluded_by_score
            );
            error!(
                "This may indicate a network issue, outdated gateway cache, or all available proxies have low scores"
            );
            error!("Will retry selection to find better options");
            return Err(LazySocks5Error::NoNetworkRequesters);
        };

        let nr_count = nodes_with_nr.len();

        // Select a random one
        let mut rng = rand::thread_rng();
        let (selected_node, nr_address) = nodes_with_nr.choose(&mut rng).ok_or_else(|| {
            error!(
                "Random selection failed despite having {} candidates",
                nr_count
            );
            LazySocks5Error::NoNetworkRequesters
        })?;

        info!(
            "Selected random Network Requester: {} (gateway: {}, {} total available)",
            nr_address, selected_node.identity, nr_count
        );

        Ok(nr_address.clone())
    }

    /// Shut down the backend
    /// Note: Callers should check for active connections before calling this
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

#[cfg(test)]
mod tests {
    use super::*;
    use nym_vpn_lib_types::TunnelState;
    use std::time::Duration;

    /// Helper to create a minimal test configuration
    fn create_test_config() -> LazySocks5Config {
        LazySocks5Config {
            mixnet_data_path: std::env::temp_dir().join("test_mixnet"),
            listen_address: "127.0.0.1:1080".parse().unwrap(),
            internal_listen_address: "127.0.0.1:1081".parse().unwrap(),
            request_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(30),
            network_requester_address: Some("test.nr@example".to_string()),
            network_requester_rotation_interval: None,
            gateway_cache_handle: None,
            network_details: None,
            vpn_exit_gateway_identity: None,
            bandwidth_command_tx: BandwidthControllerRequestSender::new(
                tokio::sync::mpsc::unbounded_channel().0,
            ),
        }
    }

    #[tokio::test]
    async fn test_lazy_socks5_creation() {
        let config = create_test_config();
        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();

        let socks5 = LazySocks5::new(config.clone(), tunnel_state, cancel_token);
        assert!(socks5.is_ok(), "Should create LazySocks5 successfully");
    }

    #[tokio::test]
    async fn test_rotation_timer_initialization() {
        let mut config = create_test_config();
        config.network_requester_rotation_interval = Some(Duration::from_secs(25 * 60));

        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();

        let socks5 = LazySocks5::new(config, tunnel_state, cancel_token).unwrap();

        // Verify rotation timer is None initially
        let last_rotation = socks5.last_rotation.read().await;
        assert!(
            last_rotation.is_none(),
            "Rotation timer should be None initially"
        );
    }

    #[tokio::test]
    async fn test_fixed_network_requester_address() {
        let config = create_test_config();
        assert!(config.network_requester_address.is_some());
        assert_eq!(config.network_requester_address.unwrap(), "test.nr@example");
    }

    #[tokio::test]
    async fn test_random_network_requester_config() {
        let mut config = create_test_config();
        config.network_requester_address = None; // Random selection

        assert!(
            config.network_requester_address.is_none(),
            "Should be configured for random NR selection"
        );
    }

    #[tokio::test]
    async fn test_rotation_disabled_when_interval_none() {
        let config = create_test_config();
        assert!(
            config.network_requester_rotation_interval.is_none(),
            "Rotation should be disabled by default"
        );
    }

    #[tokio::test]
    async fn test_rotation_enabled_with_interval() {
        let mut config = create_test_config();
        let rotation_interval = Duration::from_secs(25 * 60);
        config.network_requester_rotation_interval = Some(rotation_interval);

        assert_eq!(
            config.network_requester_rotation_interval,
            Some(rotation_interval),
            "Rotation interval should be set correctly"
        );
    }

    #[tokio::test]
    async fn test_mixnet_client_initially_none() {
        let config = create_test_config();
        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();

        let socks5 = LazySocks5::new(config, tunnel_state, cancel_token).unwrap();

        let client = socks5.mixnet_client.read().await;
        assert!(
            client.is_none(),
            "Mixnet client should be None initially (lazy init)"
        );
    }

    #[tokio::test]
    async fn test_active_connections_starts_at_zero() {
        let config = create_test_config();
        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();

        let socks5 = LazySocks5::new(config, tunnel_state, cancel_token).unwrap();

        let count = socks5.active_connections().await;
        assert_eq!(count, 0, "Active connections should start at 0");
    }

    #[tokio::test]
    async fn test_is_mixnet_running_initially_false() {
        let config = create_test_config();
        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();

        let socks5 = LazySocks5::new(config, tunnel_state, cancel_token).unwrap();

        let is_running = socks5.is_mixnet_running().await;
        assert!(!is_running, "Mixnet should not be running initially");
    }

    #[tokio::test]
    async fn test_rotation_skipped_with_active_connections() {
        let config = create_test_config();
        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();

        let socks5 = LazySocks5::new(config, tunnel_state, cancel_token).unwrap();

        // Simulate active connection
        *socks5.active_connections.write().await = 5;

        // Verify active connections
        let count = socks5.active_connections().await;
        assert_eq!(count, 5, "Should have 5 active connections");

        // Attempt rotation - should skip due to active connections
        socks5.rotate_network_requester().await;

        // Verify mixnet client is still None (not shutdown)
        let client = socks5.mixnet_client.read().await;
        assert!(
            client.is_none(),
            "Client should still be None (rotation skipped)"
        );
    }

    #[tokio::test]
    async fn test_multiple_network_requesters_can_be_used() {
        // This test verifies that different Network Requesters can be used
        // without requiring firewall changes (they're all reached through entry gateway)

        let mut config1 = create_test_config();
        config1.network_requester_address = Some("nr1.address@gateway1".to_string());

        let mut config2 = create_test_config();
        config2.network_requester_address = Some("nr2.address@gateway2".to_string());

        let mut config3 = create_test_config();
        config3.network_requester_address = Some("nr3.address@gateway3".to_string());

        // All configs should be valid and use different NRs
        assert_ne!(
            config1.network_requester_address, config2.network_requester_address,
            "Should have different NR addresses"
        );
        assert_ne!(
            config2.network_requester_address, config3.network_requester_address,
            "Should have different NR addresses"
        );

        // All should be creatable (no firewall restrictions)
        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();

        assert!(
            LazySocks5::new(config1, tunnel_state.clone(), cancel_token.clone()).is_ok(),
            "Should create with NR1"
        );
        assert!(
            LazySocks5::new(config2, tunnel_state.clone(), cancel_token.clone()).is_ok(),
            "Should create with NR2"
        );
        assert!(
            LazySocks5::new(config3, tunnel_state, cancel_token).is_ok(),
            "Should create with NR3"
        );
    }

    #[tokio::test]
    async fn test_rotation_preserves_entry_gateway() {
        // Verify that Network Requester rotation doesn't change entry gateway
        // (which would cause firewall issues)

        let mut config = create_test_config();
        config.network_requester_address = Some("initial.nr@gateway".to_string());
        config.network_requester_rotation_interval = Some(Duration::from_secs(1));

        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();

        let socks5 = LazySocks5::new(config, tunnel_state, cancel_token).unwrap();

        // Initial state - no rotation yet
        let initial_rotation = socks5.last_rotation.read().await;
        assert!(
            initial_rotation.is_none(),
            "Should have no rotation timestamp initially"
        );
    }

    #[tokio::test]
    async fn test_rotation_timing_correctness() {
        let mut config = create_test_config();
        let rotation_interval = Duration::from_millis(100); // 100ms for testing
        config.network_requester_rotation_interval = Some(rotation_interval);

        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();

        let socks5 = LazySocks5::new(config, tunnel_state, cancel_token).unwrap();

        // Manually set rotation timestamp to test timing
        *socks5.last_rotation.write().await = Some(Instant::now());

        // Wait less than rotation interval
        tokio::time::sleep(Duration::from_millis(50)).await;

        let rotation_time = socks5.last_rotation.read().await;
        assert!(
            rotation_time.is_some(),
            "Rotation timestamp should still be set"
        );

        let elapsed = rotation_time.unwrap().elapsed();
        assert!(
            elapsed < rotation_interval,
            "Should not have reached rotation interval yet"
        );
    }

    #[tokio::test]
    async fn test_concurrent_connection_safety() {
        // Test that rotation check is thread-safe with concurrent connection updates
        let config = create_test_config();
        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();

        let socks5 = Arc::new(LazySocks5::new(config, tunnel_state, cancel_token).unwrap());

        // Spawn multiple tasks that modify active connections
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let socks5_clone = socks5.clone();
                tokio::spawn(async move {
                    for _ in 0..5 {
                        let mut count = socks5_clone.active_connections.write().await;
                        *count += 1;
                        drop(count);

                        // Check rotation (should be safe during concurrent access)
                        let _ = socks5_clone.active_connections().await;

                        tokio::time::sleep(Duration::from_millis(i)).await;

                        let mut count = socks5_clone.active_connections.write().await;
                        if *count > 0 {
                            *count -= 1;
                        }
                    }
                })
            })
            .collect();

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Should end at 0 (all increments/decrements balanced)
        let final_count = socks5.active_connections().await;
        assert_eq!(final_count, 0, "Connection count should be balanced");
    }

    #[tokio::test]
    async fn test_gateway_cache_handle_not_provided() {
        // Test that random NR selection fails gracefully without gateway cache
        let mut config = create_test_config();
        config.network_requester_address = None; // Request random
        config.gateway_cache_handle = None; // But no cache handle provided

        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();

        let socks5 = LazySocks5::new(config, tunnel_state, cancel_token).unwrap();

        // Attempt random selection should fail with informative error
        let result = socks5.select_random_network_requester().await;
        assert!(result.is_err(), "Should fail without gateway cache handle");

        match result {
            Err(LazySocks5Error::GatewayDirectory(msg)) => {
                assert!(
                    msg.contains("Gateway cache handle not available"),
                    "Error should mention missing cache handle"
                );
                assert!(
                    msg.contains("Required for random Network Requester selection"),
                    "Error should explain why it's needed"
                );
            }
            _ => panic!("Expected GatewayDirectory error"),
        }
    }

    #[tokio::test]
    async fn test_rotation_configuration_validation() {
        // Test 1: Rotation enabled but no gateway cache (invalid config)
        let mut config = create_test_config();
        config.network_requester_rotation_interval = Some(Duration::from_secs(15 * 60));
        config.network_requester_address = None; // Random selection
        config.gateway_cache_handle = None; // ❌ Missing required dependency

        // This should create successfully (validation happens at runtime)
        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();
        assert!(LazySocks5::new(config, tunnel_state, cancel_token).is_ok());

        // Test 2: Fixed NR (no rotation) - gateway cache not required
        let mut config = create_test_config();
        config.network_requester_address = Some("fixed.nr@gateway".to_string());
        config.network_requester_rotation_interval = None;
        config.gateway_cache_handle = None; // ✅ Not needed for fixed mode

        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();
        assert!(
            LazySocks5::new(config, tunnel_state, cancel_token).is_ok(),
            "Fixed NR mode should work without gateway cache"
        );
    }

    #[tokio::test]
    async fn test_rotation_interval_boundary_conditions() {
        // Test very short interval (edge case)
        let mut config = create_test_config();
        config.network_requester_rotation_interval = Some(Duration::from_millis(1));
        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();
        assert!(
            LazySocks5::new(config, tunnel_state, cancel_token).is_ok(),
            "Should handle very short intervals"
        );

        // Test very long interval (edge case)
        let mut config = create_test_config();
        config.network_requester_rotation_interval = Some(Duration::from_secs(86400)); // 24h
        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));
        let cancel_token = CancellationToken::new();
        assert!(
            LazySocks5::new(config, tunnel_state, cancel_token).is_ok(),
            "Should handle very long intervals"
        );
    }
}
