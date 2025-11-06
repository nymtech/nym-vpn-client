//! Lazy SOCKS5 wrapper that initializes the Nym mixnet on first connection

use super::socks5_client::{Socks5Backend, Socks5BackendError};
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

/// Lazy SOCKS5 wrapper that starts the mixnet on first connection
pub struct LazySocks5Wrapper {
    /// Public SOCKS5 listen address (user-facing)
    public_listen_address: SocketAddr,
    /// Internal SOCKS5 address (for Nym SDK)
    internal_listen_address: SocketAddr,
    /// Data directory for mixnet client
    data_path: PathBuf,
    /// Network requester address
    network_requester_address: String,
    /// Idle timeout duration
    idle_timeout: Duration,
    /// Shared backend state
    backend: Arc<RwLock<Option<Socks5Backend>>>,
    /// Active connection counter
    active_connections: Arc<RwLock<u32>>,
    /// Last connection closed timestamp
    last_connection_closed: Arc<RwLock<Option<Instant>>>,
    /// Cancellation token
    cancel_token: CancellationToken,
}

/// Errors from the lazy SOCKS5 wrapper
#[derive(Debug, thiserror::Error)]
pub enum WrapperError {
    #[error("Failed to bind to public address {0}: {1}")]
    BindError(String, std::io::Error),

    #[error("Failed to accept connection: {0}")]
    AcceptError(std::io::Error),

    #[error("Backend error: {0}")]
    BackendError(#[from] Socks5BackendError),

    #[error("Failed to connect to internal SOCKS5 server: {0}")]
    InternalConnectionError(std::io::Error),

    #[error("Proxy error: {0}")]
    ProxyError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl LazySocks5Wrapper {
    /// Create a new lazy SOCKS5 wrapper
    pub fn new(
        data_path: PathBuf,
        idle_timeout: Duration,
        public_listen_address: SocketAddr,
        network_requester_address: String,
        cancel_token: CancellationToken,
    ) -> Self {
        // Calculate internal port by adding offset to public port
        let internal_port = 1081;
        let internal_listen_address = SocketAddr::new(public_listen_address.ip(), internal_port);

        info!(
            "Creating lazy SOCKS5 wrapper: public={}, internal={}",
            public_listen_address, internal_listen_address
        );

        Self {
            public_listen_address,
            internal_listen_address,
            data_path,
            network_requester_address,
            idle_timeout,
            backend: Arc::new(RwLock::new(None)),
            active_connections: Arc::new(RwLock::new(0)),
            last_connection_closed: Arc::new(RwLock::new(None)),
            cancel_token,
        }
    }

    /// Run the lazy SOCKS5 wrapper
    pub async fn run(self: Arc<Self>) -> Result<(), WrapperError> {
        info!(
            "Starting lazy SOCKS5 wrapper on public address: {}",
            self.public_listen_address
        );

        // Bind to public port
        let listener = TcpListener::bind(self.public_listen_address)
            .await
            .map_err(|e| WrapperError::BindError(self.public_listen_address.to_string(), e))?;

        info!("Listening on {}", self.public_listen_address);

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
    ) -> Result<(), WrapperError> {
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

        // Connect to internal SOCKS5 server
        let internal_stream = match TcpStream::connect(self.internal_listen_address).await {
            Ok(stream) => stream,
            Err(e) => {
                error!(
                    "Failed to connect to internal SOCKS5 server at {}: {}",
                    self.internal_listen_address, e
                );
                let _ = Self::send_socks5_error(&mut client_stream).await;
                self.decrement_connections().await;
                return Err(WrapperError::InternalConnectionError(e));
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
    async fn ensure_backend_started(&self) -> Result<(), WrapperError> {
        // Fast path: backend already running
        {
            let backend = self.backend.read().await;
            if backend.is_some() {
                return Ok(());
            }
        }

        // Slow path: need to initialize backend
        let mut backend_guard = self.backend.write().await;

        // Double-check after acquiring write lock (another task might have initialized)
        if backend_guard.is_some() {
            return Ok(());
        }

        info!("First connection detected, initializing Nym mixnet backend...");

        // Create and start backend
        let mut backend = Socks5Backend::new(
            self.data_path.clone(),
            self.internal_listen_address,
            self.network_requester_address.clone(),
            self.cancel_token.child_token(),
        );

        backend.start().await?;

        info!("Nym mixnet backend initialized successfully");

        *backend_guard = Some(backend);
        Ok(())
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
                        elapsed >= self.idle_timeout
                    }
                }
            };

            if should_shutdown {
                info!(
                    "Idle timeout of {:?} reached, shutting down backend",
                    self.idle_timeout
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
        let mut backend_guard = self.backend.write().await;
        if let Some(mut backend) = backend_guard.take() {
            info!("Shutting down Nym mixnet backend");
            backend.stop().await;
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

    /// Check if the backend is running
    pub async fn is_backend_running(&self) -> bool {
        let backend = self.backend.read().await;
        backend.is_some()
    }

    /// Get the public listen address
    pub fn public_address(&self) -> SocketAddr {
        self.public_listen_address
    }

    /// Get the internal listen address
    pub fn internal_address(&self) -> SocketAddr {
        self.internal_listen_address
    }
}
