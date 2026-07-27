//! Lazy SOCKS5 proxy service that routes traffic through the Nym mixnet
//!
//! This module implements a lazy-initialized mixnet, triggered by the first SOCKS5 connection
//! - Listens on a public SOCKS5 port, or HTTP RPC port
//! - Initializes the Nym mixnet on first connection
//! - Proxies SOCKS5 traffic to internal Nym SDK SOCKS5 server
//! - Proxies HTTP RPC traffic to SOCKS5
//! - Shuts down mixnet after idle timeout

mod config;
mod http_rpc;
mod lazy_socks5;
mod util;

use http_rpc::HttpRpc;
use lazy_socks5::{LazySocks5, LazySocks5Config, LazySocks5Error};
use nym_bandwidth_controller::requests::BandwidthControllerRequestSender;
use nym_gateway_directory::GatewayCacheHandle;
use nym_sdk::NymNetworkDetails;
use nym_vpn_lib_types::TunnelState;
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

pub use config::{socks5_idle_timeout, socks5_request_timeout};
pub use nym_vpn_lib_types::{HttpRpcSettings, Socks5Settings, Socks5State, Socks5Status};

/// Configuration for enabling SOCKS5 service
pub struct Socks5EnableConfig {
    pub data_dir: PathBuf,
    pub socks5_listen_address: Option<SocketAddr>,
    pub http_rpc_proxy_listen_address: Option<SocketAddr>,
    pub network_requester_address: Option<String>,
    pub network_requester_rotation_interval: Option<Duration>,
    pub gateway_cache_handle: Option<GatewayCacheHandle>,
    pub request_timeout: Duration,
    pub idle_timeout: Duration,
    pub network_details: Option<NymNetworkDetails>,
    /// VPN exit gateway identity to exclude during random Network Requester selection (for privacy)
    pub vpn_exit_gateway_identity: Option<String>,
    /// Bandwidth controller handle, used as the mixnet client's ticket provider.
    pub bandwidth_command_tx: BandwidthControllerRequestSender,
}

/// SOCKS5 service errors
#[derive(Debug, thiserror::Error)]
pub enum Socks5Error {
    #[error("Server does not support SOCKS5 network requester")]
    GatewayNotSupported,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Lazy SOCKS5 error: {0}")]
    LazySocks5Error(#[from] LazySocks5Error),
}

/// Internal service state
struct Socks5ServiceState {
    /// Current state
    state: Socks5State,
    /// Shared tunnel state
    tunnel_state: Arc<RwLock<TunnelState>>,
    /// SOCKS5 listen address
    socks5_listen_address: Option<SocketAddr>,
    /// HTTP RPC listen address
    http_rpc_proxy_listen_address: Option<SocketAddr>,
    /// Network requester address (optional - None means random selection)
    network_requester_address: Option<String>,
    /// Error message
    error_message: Option<String>,
    /// Cancellation token for the wrapper and HTTP proxy
    cancel_token: Option<CancellationToken>,
    /// Lazy SOCKS5 wrapper
    lazy_socks5: Option<Arc<LazySocks5>>,
    /// Lazy SOCKS5 task handle
    lazy_socks5_handle: Option<JoinHandle<()>>,
    /// HTTP RPC proxy task handle
    http_rpc_proxy_handle: Option<JoinHandle<()>>,
}

impl Socks5ServiceState {
    fn new(tunnel_state: Arc<RwLock<TunnelState>>) -> Self {
        Self {
            state: Socks5State::Disabled,
            tunnel_state,
            socks5_listen_address: None,
            http_rpc_proxy_listen_address: None,
            network_requester_address: None,
            error_message: None,
            cancel_token: None,
            lazy_socks5: None,
            lazy_socks5_handle: None,
            http_rpc_proxy_handle: None,
        }
    }

    async fn get_status(&self) -> Socks5Status {
        let active_connections = if let Some(lazy_socks5) = &self.lazy_socks5 {
            lazy_socks5.active_connections().await
        } else {
            0
        };

        let is_mixnet_running = if let Some(lazy_socks5) = &self.lazy_socks5 {
            lazy_socks5.is_mixnet_running().await
        } else {
            false
        };

        let state = if is_mixnet_running {
            Socks5State::Connected
        } else {
            self.state
        };

        Socks5Status {
            state,
            socks5_settings: Socks5Settings {
                listen_address: self.socks5_listen_address,
            },
            http_rpc_settings: HttpRpcSettings {
                listen_address: self.http_rpc_proxy_listen_address,
            },
            active_connections,
            error_message: self.error_message.clone(),
        }
    }

    async fn enable(&mut self, config: Socks5EnableConfig) -> Result<(), Socks5Error> {
        let Socks5EnableConfig {
            data_dir,
            socks5_listen_address,
            http_rpc_proxy_listen_address,
            network_requester_address,
            network_requester_rotation_interval,
            gateway_cache_handle,
            request_timeout,
            idle_timeout,
            network_details,
            vpn_exit_gateway_identity,
            bandwidth_command_tx,
        } = config;

        // Prevent concurrent enable calls
        match self.state {
            Socks5State::Idle | Socks5State::Connected => {
                // Already enabled - check if handle is still alive
                if let Some(handle) = &self.lazy_socks5_handle
                    && !handle.is_finished()
                {
                    debug!(
                        "SOCKS5 service already enabled with active wrapper, ignoring duplicate request"
                    );
                    return Ok(());
                }
                // Handle finished, need to clean up and re-enable
                info!("SOCKS5 wrapper handle finished, cleaning up before re-enabling");
                self.cleanup().await;
            }
            Socks5State::Error => {
                info!("SOCKS5 service in error state, cleaning up before re-enabling");
                self.cleanup().await;
            }
            Socks5State::Disabled => {
                // Normal case - service is disabled, proceed to enable
            }
        }

        let Some(socks5_listen_address) = socks5_listen_address else {
            return Err(Socks5Error::InvalidConfig(
                "SOCKS5 listen address must be specified".to_string(),
            ));
        };

        // Internal SOCKS5 address (where Nym SDK will bind)
        let internal_socks5_addr: SocketAddr = "127.0.0.1:1081"
            .parse()
            .expect("Hardcoded internal SOCKS5 address should always be valid");

        if socks5_listen_address == internal_socks5_addr {
            return Err(Socks5Error::InvalidConfig(format!(
                "SOCKS5 listen address cannot be the same as internal Nym SDK address ({internal_socks5_addr})"
            )));
        }

        info!(
            "Enabling lazy SOCKS5 service: network_requester={:?}, socks5_listen={}, http_rpc_listen={:?}, idle_timeout={}s, rotation_interval={:?}",
            network_requester_address,
            socks5_listen_address,
            http_rpc_proxy_listen_address,
            idle_timeout.as_secs(),
            network_requester_rotation_interval
        );

        // Create an independent cancellation token for this enable operation.
        // Both the wrapper and HTTP proxy share this token to ensure synchronized lifecycles.
        let cancel_token = CancellationToken::new();

        // Create lazy SOCKS5 wrapper
        let config = LazySocks5Config {
            mixnet_data_path: data_dir,
            listen_address: socks5_listen_address,
            internal_listen_address: internal_socks5_addr,
            request_timeout,
            idle_timeout,
            network_requester_address: network_requester_address.clone(),
            network_requester_rotation_interval,
            gateway_cache_handle,
            network_details,
            vpn_exit_gateway_identity,
            bandwidth_command_tx,
        };
        let lazy_socks5 = Arc::new(LazySocks5::new(
            config,
            self.tunnel_state.clone(),
            cancel_token.clone(),
        )?);

        // Spawn lazy SOCKS5 task
        let lazy_socks5_clone = lazy_socks5.clone();
        let lazy_socks5_handle = tokio::spawn(async move {
            if let Err(e) = lazy_socks5_clone.run().await {
                error!("Lazy SOCKS5 error: {}", e);
            }
        });

        info!("Lazy SOCKS5 service enabled successfully");
        info!(
            "Listening on {} (mixnet will initialize on first connection)",
            socks5_listen_address
        );

        // Optionally start HTTP RPC proxy
        let http_rpc_proxy_handle = http_rpc_proxy_listen_address.map(|http_rpc_address| {
            info!("Starting HTTP RPC proxy on {}", http_rpc_address);

            // Use the same service_cancel_token to ensure HTTP proxy and wrapper
            // have synchronized lifecycles
            let mut http_proxy =
                HttpRpc::new(http_rpc_address, request_timeout, cancel_token.clone());

            let lazy_socks5_clone = lazy_socks5.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = http_proxy.start(lazy_socks5_clone).await {
                    error!("HTTP RPC proxy error: {}", e);
                }
            });

            info!("HTTP RPC proxy enabled successfully");
            handle
        });

        // Store state
        self.socks5_listen_address = Some(socks5_listen_address);
        self.http_rpc_proxy_listen_address = http_rpc_proxy_listen_address;
        self.network_requester_address = network_requester_address;
        self.state = Socks5State::Idle;
        self.error_message = None;
        self.lazy_socks5 = Some(lazy_socks5);
        self.lazy_socks5_handle = Some(lazy_socks5_handle);
        self.http_rpc_proxy_handle = http_rpc_proxy_handle;
        self.cancel_token = Some(cancel_token);

        Ok(())
    }

    async fn disable(&mut self) {
        if self.state == Socks5State::Disabled {
            return;
        }

        info!("Disabling lazy SOCKS5 service");
        self.cleanup().await;
        self.state = Socks5State::Disabled;
        self.error_message = None;
    }

    async fn cleanup(&mut self) {
        // Cancel all operations via the shared token
        if let Some(token) = self.cancel_token.take() {
            debug!("Cancelling SOCKS5 service token");
            token.cancel();
        }

        // Abort and await HTTP RPC proxy task
        if let Some(handle) = self.http_rpc_proxy_handle.take() {
            debug!("Stopping HTTP RPC proxy task");
            handle.abort();
            if let Err(e) = handle.await
                && !e.is_cancelled()
            {
                error!("HTTP RPC proxy task error: {:?}", e);
            }
        }

        // Await lazy SOCKS5 task
        if let Some(handle) = self.lazy_socks5_handle.take() {
            debug!("Stopping lazy SOCKS5 wrapper task");
            handle.abort();
            if let Err(e) = handle.await
                && !e.is_cancelled()
            {
                error!("Lazy SOCKS5 wrapper task error: {:?}", e);
            }
        }

        self.lazy_socks5 = None;
        debug!("SOCKS5 service cleanup complete");
    }
}

/// Handle to the lazy SOCKS5 service
#[derive(Clone)]
pub struct Socks5Service {
    /// Socks5 service state
    state: Arc<RwLock<Socks5ServiceState>>,
}

impl Socks5Service {
    /// Create a new lazy SOCKS5 service (starts in disabled state)
    pub fn new(tunnel_state: Arc<RwLock<TunnelState>>) -> Self {
        Self {
            state: Arc::new(RwLock::new(Socks5ServiceState::new(tunnel_state))),
        }
    }

    /// Enable the lazy SOCKS5 proxy with optional HTTP RPC proxy
    pub async fn enable(&self, config: Socks5EnableConfig) -> Result<(), Socks5Error> {
        let mut state = self.state.write().await;
        state.enable(config).await
    }

    /// Disable the lazy SOCKS5 proxy
    pub async fn disable(&self) -> Result<(), Socks5Error> {
        let mut state = self.state.write().await;
        state.disable().await;
        Ok(())
    }

    /// Get the current status
    pub async fn get_status(&self) -> Result<Socks5Status, Socks5Error> {
        let state = self.state.read().await;
        Ok(state.get_status().await)
    }

    /// Check if SOCKS5 is currently enabled
    pub fn is_enabled(&self) -> bool {
        // Use try_read to avoid blocking - if we can't read, assume not enabled
        self.state
            .try_read()
            .map(|state| state.state != Socks5State::Disabled)
            .unwrap_or(false)
    }

    /// Shutdown the service
    pub async fn shutdown(&self) {
        let mut state = self.state.write().await;
        state.cleanup().await;
    }
}
