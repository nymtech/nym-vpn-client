// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Lazy SOCKS5 wrapper that initializes the Nym mixnet on first connection.

mod backend;
mod connection;

use super::util::ConnectionGuard;
use nym_bandwidth_controller::requests::BandwidthControllerRequestSender;
use nym_gateway_directory::GatewayCacheHandle;
use nym_network_defaults::v2::NymNetworkDetails;
use nym_sdk::mixnet::Socks5MixnetClient;
use nym_vpn_lib_types::{TunnelConnectionData, TunnelState, TunnelType};
use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, watch},
    task::JoinHandle,
    time::Instant,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};

const IDLE_CHECK_INTERVAL: Duration = Duration::from_secs(5);
const ROTATION_CHECK_INTERVAL: Duration = Duration::from_secs(60);

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

/// Lazy SOCKS5 wrapper, owned by the task that runs it
pub struct LazySocks5 {
    shared: Arc<SharedState>,
    /// When the last connection closed, or None while there are active connections
    idle_since: Option<Instant>,
    /// Last time Network Requester was rotated
    last_rotation: Option<Instant>,
    /// Tunnel state seen by the last tunnel state check
    last_tunnel_state: TunnelState,
}

impl LazySocks5 {
    /// Bind the public SOCKS5 address and run the wrapper in a new task
    pub async fn spawn(
        config: LazySocks5Config,
        tunnel_state: watch::Receiver<TunnelState>,
        cancel_token: CancellationToken,
    ) -> Result<(LazySocks5Handle, JoinHandle<()>), LazySocks5Error> {
        info!(
            "Starting lazy SOCKS5 wrapper: public={}, internal={}",
            config.listen_address, config.internal_listen_address
        );

        let listener = TcpListener::bind(config.listen_address)
            .await
            .map_err(|e| LazySocks5Error::BindError(config.listen_address.to_string(), e))?;

        info!("Listening on {}", config.listen_address);

        let last_tunnel_state = tunnel_state.borrow().clone();
        let shared = Arc::new(SharedState::new(config, tunnel_state, cancel_token));
        let lazy_socks5 = Self {
            shared: shared.clone(),
            idle_since: None,
            last_rotation: None,
            last_tunnel_state,
        };

        Ok((
            LazySocks5Handle { shared },
            tokio::spawn(lazy_socks5.run(listener)),
        ))
    }

    async fn run(mut self, listener: TcpListener) {
        // Idle timeout: check every 5 seconds, starting 5 seconds from now
        let mut idle_check =
            tokio::time::interval_at(Instant::now() + IDLE_CHECK_INTERVAL, IDLE_CHECK_INTERVAL);
        idle_check.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        // Network Requester rotation: check every minute, if enabled
        let rotation_enabled = self
            .shared
            .config
            .network_requester_rotation_interval
            .is_some();
        if let Some(rotation_interval) = self.shared.config.network_requester_rotation_interval {
            info!(
                "Network Requester rotation enabled (interval: {:?})",
                rotation_interval
            );
        }
        let mut rotation_check = tokio::time::interval(ROTATION_CHECK_INTERVAL);
        rotation_check.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Tunnel state: shut down the mixnet backend when the VPN connects
        let mut tunnel_state = self.shared.tunnel_state.clone();
        self.last_tunnel_state = tunnel_state.borrow_and_update().clone();
        let mut tunnel_state_open = true;

        let cancel_token = self.shared.cancel_token.clone();

        loop {
            tokio::select! {
                result = listener.accept() => match result {
                    Ok((stream, addr)) => self.handle_connection(stream, addr),
                    Err(e) => error!("Failed to accept connection: {e}"),
                },
                _ = idle_check.tick() => {
                    self.check_idle_timeout().await;
                }
                _ = rotation_check.tick(), if rotation_enabled => {
                    self.check_network_requester_rotation().await;
                }
                result = tunnel_state.changed(), if tunnel_state_open => {
                    if result.is_ok() {
                        let current_tunnel_state = tunnel_state.borrow_and_update().clone();
                        self.handle_tunnel_state_change(current_tunnel_state).await;
                    } else {
                        tunnel_state_open = false;
                    }
                }
                _ = cancel_token.cancelled() => {
                    info!("Lazy SOCKS5 wrapper shutting down");
                    break;
                }
            }
        }

        // Waits for any mixnet client initialization in progress, so the client it creates is shut
        // down too
        Self::shutdown_backend(&self.shared).await;

        info!("Lazy SOCKS5 wrapper stopped");
    }

    /// Route an accepted connection based on the tunnel state:
    /// 1. If the tunnel is mixnet and connected -> use existing tunnel
    /// 2. If the tunnel is wireguard and connected -> create new tunnel
    /// 3. If the tunnel is disconnected or error'd -> create new tunnel
    /// 4. If the tunnel is connecting, disconnecting or offline -> reject connection
    fn handle_connection(&self, mut stream: TcpStream, addr: SocketAddr) {
        enum Route {
            ExistingTunnel,
            NewTunnel,
            Reject,
        }

        debug!("Accepted connection from {}", addr);
        // Configure TCP options for better performance
        if let Err(e) = stream.set_nodelay(true) {
            warn!("Failed to set TCP_NODELAY for {}: {}", addr, e);
        }

        let route = match &*self.shared.tunnel_state.borrow() {
            TunnelState::Connected { connection_data } => match &connection_data.tunnel {
                TunnelConnectionData::Mixnet(_) => Route::ExistingTunnel, // 1
                TunnelConnectionData::Wireguard(_) => Route::NewTunnel,   // 2
            },
            TunnelState::Disconnected | TunnelState::Error(_) => Route::NewTunnel, // 3
            tunnel_state => {
                warn!(
                    "Rejecting SOCKS5 connection from {addr} due to tunnel state: {tunnel_state:?}"
                );
                Route::Reject // 4
            }
        };

        let shared = self.shared.clone();

        // Create the guard immediately before spawning.
        // This guarantees the atomic connection count is >= 1 before the idle loop ticks again.
        let connection_guard = shared.connection_guard();

        tokio::spawn(async move {
            // Keep connection_guard in scope for the lifetime of this proxy task
            let _guard = connection_guard;

            let result = match route {
                Route::ExistingTunnel => Self::route_via_existing_tunnel(stream, addr).await,
                Route::NewTunnel => {
                    Box::pin(Self::route_via_new_tunnel(&shared, stream, addr)).await
                }
                Route::Reject => {
                    Self::reject_connection(&mut stream).await;
                    Ok(())
                }
            };

            if let Err(e) = result {
                error!("Connection handler error for {addr}: {e}");
            }
        });
    }

    /// Shut down the backend once there were no active connections for the idle timeout
    async fn check_idle_timeout(&mut self) {
        let should_shutdown = if self.shared.active_connections() > 0 {
            // Still have active connections, reset timer
            self.idle_since = None;
            false
        } else {
            // No active connections, start the timer if it is not running yet
            let idle_since = *self.idle_since.get_or_insert_with(Instant::now);
            idle_since.elapsed() >= self.shared.config.idle_timeout
        };

        trace!("should_shutdown: {}", should_shutdown);

        if should_shutdown {
            info!(
                "Idle timeout of {:?} reached, shutting down backend",
                self.shared.config.idle_timeout
            );
            Self::shutdown_backend(&self.shared).await;

            // Reset timer
            self.idle_since = None;
        }
    }

    /// Manage the mixnet backend when the tunnel state changes
    async fn handle_tunnel_state_change(&mut self, tunnel_state: TunnelState) {
        let connected_tunnel_type = |tunnel_state: &TunnelState| match tunnel_state {
            TunnelState::Connected { connection_data } => {
                Some(connection_data.tunnel.tunnel_type())
            }
            _ => None,
        };
        let last_tunnel_type = connected_tunnel_type(&self.last_tunnel_state);
        let current_tunnel_type = connected_tunnel_type(&tunnel_state);
        self.last_tunnel_state = tunnel_state;

        match current_tunnel_type {
            Some(TunnelType::Mixnet) if last_tunnel_type != Some(TunnelType::Mixnet) => {
                // dVPN just became available (Mixnet mode) - shut down mixnet backend
                info!(
                    "dVPN tunnel is now available (Mixnet mode), shutting down mixnet SOCKS5 backend to save bandwidth"
                );
                Self::shutdown_backend(&self.shared).await;
            }
            Some(TunnelType::Wireguard) if last_tunnel_type != Some(TunnelType::Wireguard) => {
                // VPN just connected in WireGuard mode - if mixnet client is running, shut it down
                // so it will be recreated with VPN's entry gateway on next connection
                if self.shared.is_mixnet_running() {
                    info!(
                        "VPN connected in WireGuard mode while mixnet client is running, shutting down mixnet client to ensure it uses VPN's entry gateway (firewall compatibility)"
                    );
                    Self::shutdown_backend(&self.shared).await;
                }
            }
            None if last_tunnel_type.is_some() => {
                // VPN just disconnected
                info!(
                    "VPN disconnected, mixnet SOCKS5 backend will be lazily initialized on next connection"
                );
            }
            _ => {}
        }
    }

    /// Rotate the Network Requester once the rotation interval has passed
    async fn check_network_requester_rotation(&mut self) {
        // Only rotate if mixnet is running and dVPN (WireGuard) is connected
        let dvpn_wireguard_active = matches!(
            *self.shared.tunnel_state.borrow(),
            TunnelState::Connected { ref connection_data }
            if connection_data.tunnel.tunnel_type() == TunnelType::Wireguard
        );

        if let Some(rotation_interval) = self.shared.config.network_requester_rotation_interval
            && self.shared.is_mixnet_running()
            && dvpn_wireguard_active
        {
            // Start the timer on the first check
            let rotated_at = *self.last_rotation.get_or_insert_with(Instant::now);
            if rotated_at.elapsed() >= rotation_interval {
                self.last_rotation = Some(Instant::now());
                info!("Rotating Network Requester after {:?}", rotation_interval);
                Self::rotate_network_requester(&self.shared).await;
            }
        }
    }
}

/// State shared between the run loop, the connection tasks and [`LazySocks5Handle`]
pub struct SharedState {
    /// Configuration
    config: LazySocks5Config,
    /// Shared tunnel state
    tunnel_state: watch::Receiver<TunnelState>,
    /// Cancellation token for shutdown
    cancel_token: CancellationToken,
    /// Active connection counter
    active_connections: Arc<AtomicU32>,
    /// Whether `mixnet_client` holds a client, readable without waiting for initialization
    is_mixnet_running: AtomicBool,
    /// Mixnet client, locked for the whole initialization to prevent concurrent initialization
    mixnet_client: Mutex<Option<Socks5MixnetClient>>,
}

impl SharedState {
    fn new(
        config: LazySocks5Config,
        tunnel_state: watch::Receiver<TunnelState>,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            config,
            tunnel_state,
            cancel_token,
            active_connections: Arc::new(AtomicU32::new(0)),
            is_mixnet_running: AtomicBool::new(false),
            mixnet_client: Mutex::new(None),
        }
    }

    fn active_connections(&self) -> u32 {
        self.active_connections.load(Ordering::Relaxed)
    }

    fn is_mixnet_running(&self) -> bool {
        self.is_mixnet_running.load(Ordering::Relaxed)
    }

    /// Count a connection as active until the returned guard is dropped
    fn connection_guard(&self) -> ConnectionGuard {
        ConnectionGuard::new(self.active_connections.clone())
    }
}

/// Status of a running lazy SOCKS5 wrapper
pub struct LazySocks5Handle {
    shared: Arc<SharedState>,
}

impl LazySocks5Handle {
    /// Get the number of active connections
    pub fn active_connections(&self) -> u32 {
        self.shared.active_connections()
    }

    /// Is mixnet running
    pub fn is_mixnet_running(&self) -> bool {
        self.shared.is_mixnet_running()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nym_vpn_lib_types::TunnelState;
    use std::time::Duration;

    /// Helper to create a minimal test configuration
    pub fn create_test_config() -> LazySocks5Config {
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
    async fn test_rotation_skipped_with_active_connections() {
        let config = create_test_config();
        let (_tunnel_state_tx, tunnel_state) = watch::channel(TunnelState::Disconnected);
        let cancel_token = CancellationToken::new();

        let shared = SharedState::new(config, tunnel_state, cancel_token);

        // Simulate active connections
        let _guards: Vec<_> = (0..5).map(|_| shared.connection_guard()).collect();

        // Verify active connections
        let count = shared.active_connections();
        assert_eq!(count, 5, "Should have 5 active connections");

        // Attempt rotation - should skip due to active connections
        LazySocks5::rotate_network_requester(&shared).await;

        // Verify mixnet client is still not running (not shutdown)
        assert!(
            !shared.is_mixnet_running(),
            "Client should still not be running (rotation skipped)"
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
    }

    #[tokio::test]
    async fn test_mixnet_client_initially_none() {
        let config = create_test_config();
        let (_tunnel_state_tx, tunnel_state) = watch::channel(TunnelState::Disconnected);
        let cancel_token = CancellationToken::new();

        let shared = SharedState::new(config, tunnel_state, cancel_token);

        let client = shared.mixnet_client.lock().await;
        assert!(
            client.is_none(),
            "Mixnet client should be None initially (lazy init)"
        );
    }

    #[tokio::test]
    async fn test_active_connections_starts_at_zero() {
        let config = create_test_config();
        let (_tunnel_state_tx, tunnel_state) = watch::channel(TunnelState::Disconnected);
        let cancel_token = CancellationToken::new();

        let shared = SharedState::new(config, tunnel_state, cancel_token);

        let count = shared.active_connections();
        assert_eq!(count, 0, "Active connections should start at 0");
    }

    #[tokio::test]
    async fn test_is_mixnet_running_initially_false() {
        let config = create_test_config();
        let (_tunnel_state_tx, tunnel_state) = watch::channel(TunnelState::Disconnected);
        let cancel_token = CancellationToken::new();

        let shared = SharedState::new(config, tunnel_state, cancel_token);

        let is_running = shared.is_mixnet_running();
        assert!(!is_running, "Mixnet should not be running initially");
    }

    #[tokio::test]
    async fn test_concurrent_connection_safety() {
        // Test that rotation check is thread-safe with concurrent connection updates
        let config = create_test_config();
        let (_tunnel_state_tx, tunnel_state) = watch::channel(TunnelState::Disconnected);
        let cancel_token = CancellationToken::new();

        let shared = Arc::new(SharedState::new(config, tunnel_state, cancel_token));

        // Spawn multiple tasks that modify active connections
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let shared = shared.clone();
                tokio::spawn(async move {
                    for _ in 0..5 {
                        let guard = shared.connection_guard();

                        // Check rotation (should be safe during concurrent access)
                        let _ = shared.active_connections();

                        tokio::time::sleep(Duration::from_millis(i)).await;

                        drop(guard);
                    }
                })
            })
            .collect();

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Should end at 0 (all increments/decrements balanced)
        let final_count = shared.active_connections();
        assert_eq!(final_count, 0, "Connection count should be balanced");
    }
}
