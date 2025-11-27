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
use nym_vpn_lib_types::TunnelState;
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

pub use config::{socks5_idle_timeout, socks5_request_timeout};
pub use nym_vpn_lib_types::{HttpRpcSettings, Socks5Settings, Socks5State, Socks5Status};

/// Configuration for enabling SOCKS5 service
struct Socks5EnableConfig {
    data_dir: PathBuf,
    socks5_listen_address: String,
    http_rpc_proxy_listen_address: String,
    network_requester_address: String,
    request_timeout: Duration,
    idle_timeout: Duration,
    cancel_token: CancellationToken,
}

/// SOCKS5 service errors
#[derive(Debug, thiserror::Error)]
pub enum Socks5Error {
    #[error("Gateway does not support SOCKS5 network requester")]
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
    socks5_listen_address: String,
    /// HTTP RPC listen address
    http_rpc_proxy_listen_address: String,
    /// Network requester address
    network_requester_address: String,
    /// Error message
    error_message: Option<String>,
    /// Cancellation token
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
            socks5_listen_address: String::new(),
            http_rpc_proxy_listen_address: String::new(),
            network_requester_address: String::new(),
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
                listen_address: self.socks5_listen_address.clone(),
            },
            http_rpc_settings: nym_vpn_lib_types::HttpRpcSettings {
                listen_address: self.http_rpc_proxy_listen_address.clone(),
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
            request_timeout,
            idle_timeout,
            cancel_token,
        } = config;
        // Check if already enabled
        if self.state != Socks5State::Disabled {
            info!(
                "Lazy SOCKS5 service is in {:?} state, cleaning up existing service first",
                self.state
            );
            self.cleanup().await;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        info!(
            "Enabling lazy SOCKS5 service: network_requester={}, socks5_listen={}, http_rpc_listen={}, idle_timeout={}s",
            network_requester_address,
            socks5_listen_address,
            http_rpc_proxy_listen_address,
            idle_timeout.as_secs()
        );

        // Parse listen addresses
        let socks5_listen_addr: SocketAddr = socks5_listen_address.parse().map_err(|e| {
            Socks5Error::InvalidConfig(format!("Invalid SOCKS5 listen address: {}", e))
        })?;

        // Internal SOCKS5 address (where Nym SDK will bind)
        let internal_socks5_addr: SocketAddr = "127.0.0.1:1081".parse().unwrap();

        // Create lazy SOCKS5 wrapper
        let config = LazySocks5Config {
            mixnet_data_path: data_dir,
            listen_address: socks5_listen_addr,
            internal_listen_address: internal_socks5_addr,
            request_timeout,
            idle_timeout,
            network_requester_address: network_requester_address.clone(),
        };
        let lazy_socks5 = Arc::new(LazySocks5::new(
            config,
            self.tunnel_state.clone(),
            cancel_token.child_token(),
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
        let http_rpc_proxy_handle = if !http_rpc_proxy_listen_address.is_empty() {
            let http_rpc_addr: SocketAddr = http_rpc_proxy_listen_address.parse().map_err(|e| {
                Socks5Error::InvalidConfig(format!("Invalid HTTP RPC listen address: {}", e))
            })?;

            info!(
                "Starting HTTP RPC proxy on {}",
                http_rpc_proxy_listen_address
            );

            let mut http_proxy =
                HttpRpc::new(http_rpc_addr, request_timeout, cancel_token.child_token());

            let lazy_socks5_clone = lazy_socks5.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = http_proxy.start(lazy_socks5_clone).await {
                    error!("HTTP RPC proxy error: {}", e);
                }
            });

            info!("HTTP RPC proxy enabled successfully");
            Some(handle)
        } else {
            None
        };

        // Store state
        self.socks5_listen_address = socks5_listen_address;
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
        // Cancel all operations
        if let Some(token) = self.cancel_token.take() {
            token.cancel();
        }

        // Stop HTTP RPC proxy task
        if let Some(handle) = self.http_rpc_proxy_handle.take() {
            debug!("Stopping HTTP RPC proxy");
            handle.abort();
            let _ = handle.await;
        }

        // Stop lazy SOCKS5 task
        if let Some(handle) = self.lazy_socks5_handle.take() {
            debug!("Stopping lazy SOCKS5 wrapper");
            handle.abort();
            let _ = handle.await;
        }

        self.lazy_socks5 = None;
    }
}

/// Handle to the lazy SOCKS5 service
pub struct Socks5Service {
    /// Socks5 service state
    state: Arc<RwLock<Socks5ServiceState>>,
    /// Shutdown token
    shutdown_token: CancellationToken,
}

impl Socks5Service {
    /// Create a new lazy SOCKS5 service (starts in disabled state)
    pub fn new(tunnel_state: Arc<RwLock<TunnelState>>, shutdown_token: CancellationToken) -> Self {
        Self {
            state: Arc::new(RwLock::new(Socks5ServiceState::new(tunnel_state))),
            shutdown_token,
        }
    }

    /// Enable the lazy SOCKS5 proxy with optional HTTP RPC proxy
    pub async fn enable(
        &self,
        data_dir: PathBuf,
        socks5_listen_address: String,
        http_rpc_proxy_listen_address: String,
        network_requester_address: String,
        request_timeout: Duration,
        idle_timeout: Duration,
    ) -> Result<(), Socks5Error> {
        let mut state = self.state.write().await;
        state
            .enable(Socks5EnableConfig {
                data_dir,
                socks5_listen_address,
                http_rpc_proxy_listen_address,
                network_requester_address,
                request_timeout,
                idle_timeout,
                cancel_token: self.shutdown_token.child_token(),
            })
            .await
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

    /// Shutdown the service
    pub async fn shutdown(&self) {
        let mut state = self.state.write().await;
        state.cleanup().await;
    }
}

impl Drop for Socks5Service {
    fn drop(&mut self) {
        self.shutdown_token.cancel();
    }
}
