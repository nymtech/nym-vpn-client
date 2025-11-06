//! Lazy SOCKS5 wrapper that initializes the Nym mixnet on first connection

use nym_sdk::mixnet::{MixnetClientBuilder, Socks5, Socks5MixnetClient};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncWriteExt, copy_bidirectional};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::{Instant, sleep};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

/// Configuration for the LazySocks5
#[derive(Debug, Clone)]
pub struct LazySocks5Config {
    /// Data directory for mixnet client state
    _mixnet_data_path: PathBuf,
    /// Public SOCKS5 listen address (user-facing)
    listen_address: SocketAddr,
    /// Internal SOCKS5 address (from Nym SDK)
    internal_listen_address: SocketAddr,
    /// Request timeout duration
    request_timeout: Duration,
    /// Idle timeout duration
    idle_timeout: Duration,
    /// Exit node gateway address
    network_requester_address: String,
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
    /// Cancellation token for shutdown
    cancel_token: CancellationToken,
    /// Active connection counter   
    active_connections: Arc<RwLock<u32>>,
    /// Last connection closed timestamp
    last_connection_closed: Arc<RwLock<Option<Instant>>>,
    /// Mixnet client
    mixnet_client: Arc<RwLock<Option<Socks5MixnetClient>>>,
}

impl LazySocks5 {
    /// Create a new lazy SOCKS5 wrapper
    pub fn new(
        mixnet_data_path: PathBuf,
        listen_address: SocketAddr,
        internal_listen_address: SocketAddr,
        request_timeout: Duration,
        idle_timeout: Duration,
        network_requester_address: String,
        cancel_token: CancellationToken,
    ) -> Result<Self, LazySocks5Error> {
        info!(
            "Creating LazySocks5: public={}, internal={}",
            listen_address.to_string(),
            internal_listen_address.to_string()
        );

        Ok(Self {
            config: LazySocks5Config {
                _mixnet_data_path: mixnet_data_path,
                listen_address,
                internal_listen_address,
                request_timeout,
                idle_timeout,
                network_requester_address,
            },
            cancel_token,
            active_connections: Arc::new(RwLock::new(0)),
            last_connection_closed: Arc::new(RwLock::new(None)),
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

        // Accept connections loop
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            debug!("Accepted connection from {}", addr);
                            let wrapper = self.clone();

                            // Spawn task to handle this connection
                            tokio::spawn(async move {
                                if let Err(e) = wrapper.handle_connection(stream, addr).await {
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
        self.shutdown_backend().await;

        info!("Lazy SOCKS5 wrapper stopped");
        Ok(())
    }

    /// Handle a single connection
    async fn handle_connection(
        &self,
        mut client_stream: TcpStream,
        client_addr: SocketAddr,
    ) -> Result<(), LazySocks5Error> {
        // Increment connection counter
        {
            let mut count = self.active_connections.write().await;
            *count += 1;
            debug!("Active connections: {}", *count);
        }

        // Ensure backend is started (lazy initialization)
        if let Err(e) = self.ensure_backend_started().await {
            error!("Failed to start backend for {}: {}", client_addr, e);
            // Send SOCKS5 error response
            let _ = Self::send_socks5_error(&mut client_stream).await;
            self.decrement_connections().await;
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
                self.decrement_connections().await;
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

        // Decrement connection counter
        self.decrement_connections().await;

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
        // Build the mixnet client with SOCKS5 configuration
        let mixnet_client = MixnetClientBuilder::new_ephemeral()
            // .await
            // .map_err(|e| {
            //     error!("Failed to create mixnet client builder: {}", e);
            //     Socks5BackendError::MixnetInitError(e.to_string())
            // })?
            .socks5_config(socks5_config)
            .build()
            .map_err(|e| {
                error!("Failed to build mixnet client: {}", e);
                LazySocks5Error::Internal(e.to_string())
            })?;

        // Connect to the mixnet via SOCKS5
        info!("Connecting to mixnet via SOCKS5...");
        info!("This will spawn the internal SOCKS5 server and establish mixnet connection...");
        let mixnet_client = mixnet_client
            .connect_to_mixnet_via_socks5()
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

    /// Shut down the backend
    async fn shutdown_backend(&self) {
        let mut client_guard = self.mixnet_client.write().await;
        if let Some(mixnet_client) = client_guard.take() {
            info!("Shutting down Nym mixnet client");
            mixnet_client.disconnect().await;
        }
    }

    /// Decrement active connection counter
    async fn decrement_connections(&self) {
        let mut count = self.active_connections.write().await;
        if *count > 0 {
            *count -= 1;
            debug!("Active connections: {}", *count);
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

    /// Get the public listen address
    pub fn public_address(&self) -> SocketAddr {
        self.config.listen_address
    }
}
