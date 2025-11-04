// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Service wrapper for the lazy SOCKS5 wrapper

use super::http_rpc_proxy::HttpRpcProxy;
use super::socks5_wrapper::LazySocks5Wrapper;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

// Re-export shared types from nym-vpn-lib-types
pub use nym_vpn_lib_types::{Socks5Settings, Socks5State, Socks5Status};

/// SOCKS5 service errors
#[derive(Debug, thiserror::Error)]
pub enum LazySocks5Error {
    #[error("Gateway does not support SOCKS5 network requester")]
    GatewayNotSupported,

    #[error("Failed to start SOCKS5 service: {0}")]
    StartError(String),

    #[error("Service is not enabled")]
    NotEnabled,

    #[error("Service is already enabled")]
    AlreadyEnabled,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Handle to the lazy SOCKS5 service
pub struct LazySocks5Service {
    state: Arc<RwLock<ServiceState>>,
    shutdown_token: CancellationToken,
}

impl LazySocks5Service {
    /// Create a new lazy SOCKS5 service (starts in disabled state)
    pub fn new(shutdown_token: CancellationToken) -> Self {
        Self {
            state: Arc::new(RwLock::new(ServiceState::new())),
            shutdown_token,
        }
    }

    /// Enable the lazy SOCKS5 proxy with optional HTTP RPC proxy
    pub async fn enable(
        &self,
        data_dir: PathBuf,
        socks5_listen_address: String,
        http_rpc_listen_address: String,
        network_requester_address: String,
        idle_timeout_secs: u64,
    ) -> Result<(), LazySocks5Error> {
        let mut state = self.state.write().await;
        state
            .enable(
                data_dir,
                socks5_listen_address,
                http_rpc_listen_address,
                network_requester_address,
                idle_timeout_secs,
                self.shutdown_token.child_token(),
            )
            .await
    }

    /// Disable the lazy SOCKS5 proxy
    pub async fn disable(&self) -> Result<(), LazySocks5Error> {
        let mut state = self.state.write().await;
        state.disable().await;
        Ok(())
    }

    /// Get the current status
    pub async fn get_status(&self) -> Result<Socks5Status, LazySocks5Error> {
        let state = self.state.read().await;
        Ok(state.get_status().await)
    }

    /// Shutdown the service
    pub async fn shutdown(&self) {
        let mut state = self.state.write().await;
        state.cleanup().await;
    }
}

/// Internal service state
struct ServiceState {
    state: Socks5State,
    socks5_listen_address: String,
    http_rpc_listen_address: String,
    network_requester_address: String,
    idle_timeout_secs: u64,
    error_message: Option<String>,
    wrapper: Option<Arc<LazySocks5Wrapper>>,
    wrapper_handle: Option<JoinHandle<()>>,
    http_rpc_handle: Option<JoinHandle<()>>,
    cancel_token: Option<CancellationToken>,
}

impl ServiceState {
    fn new() -> Self {
        Self {
            state: Socks5State::Disabled,
            socks5_listen_address: "127.0.0.1:1080".to_string(),
            http_rpc_listen_address: "127.0.0.1:8545".to_string(),
            network_requester_address: String::new(),
            idle_timeout_secs: 60,
            error_message: None,
            wrapper: None,
            wrapper_handle: None,
            http_rpc_handle: None,
            cancel_token: None,
        }
    }

    async fn get_status(&self) -> Socks5Status {
        let active_connections = if let Some(wrapper) = &self.wrapper {
            wrapper.active_connections().await
        } else {
            0
        };

        Socks5Status {
            state: self.state,
            socks5_settings: Socks5Settings {
                listen_address: self.socks5_listen_address.clone(),
            },
            http_rpc_settings: nym_vpn_lib_types::HttpRpcSettings {
                listen_address: self.http_rpc_listen_address.clone(),
            },
            active_connections,
            error_message: self.error_message.clone(),
        }
    }

    async fn enable(
        &mut self,
        data_dir: PathBuf,
        socks5_listen_address: String,
        http_rpc_listen_address: String,
        network_requester_address: String,
        idle_timeout_secs: u64,
        cancel_token: CancellationToken,
    ) -> Result<(), LazySocks5Error> {
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
            "Enabling lazy SOCKS5 service: network_requester_address={}, socks5_listen_address={}, idle_timeout={}s",
            network_requester_address, socks5_listen_address, idle_timeout_secs
        );

        // Parse listen address
        let listen_addr = socks5_listen_address.parse().map_err(|e| {
            LazySocks5Error::InvalidConfig(format!("Invalid listen address: {}", e))
        })?;

        // Create lazy wrapper
        let wrapper = Arc::new(
            LazySocks5Wrapper::new(
                listen_addr,
                data_dir,
                network_requester_address.clone(),
                cancel_token.child_token(),
            )
            .with_idle_timeout(Duration::from_secs(idle_timeout_secs)),
        );

        // Spawn wrapper task
        let wrapper_clone = wrapper.clone();
        let wrapper_handle = tokio::spawn(async move {
            if let Err(e) = wrapper_clone.run().await {
                error!("Lazy SOCKS5 wrapper error: {}", e);
            }
        });

        info!("Lazy SOCKS5 service enabled successfully");
        info!(
            "Listening on {} (mixnet will initialize on first connection)",
            socks5_listen_address
        );

        // Optionally start HTTP RPC proxy
        let http_rpc_handle = if !http_rpc_listen_address.is_empty() {
            info!("Starting HTTP RPC proxy on {}", http_rpc_listen_address);

            let mut http_proxy =
                HttpRpcProxy::new(http_rpc_listen_address.clone(), cancel_token.child_token());

            let wrapper_clone = wrapper.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = http_proxy.start(wrapper_clone).await {
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
        self.http_rpc_listen_address = http_rpc_listen_address;
        self.network_requester_address = network_requester_address;
        self.idle_timeout_secs = idle_timeout_secs;
        self.state = Socks5State::Idle;
        self.error_message = None;
        self.wrapper = Some(wrapper);
        self.wrapper_handle = Some(wrapper_handle);
        self.http_rpc_handle = http_rpc_handle;
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
        if let Some(handle) = self.http_rpc_handle.take() {
            debug!("Stopping HTTP RPC proxy");
            handle.abort();
            let _ = handle.await;
        }

        // Stop wrapper task
        if let Some(handle) = self.wrapper_handle.take() {
            debug!("Stopping lazy SOCKS5 wrapper");
            handle.abort();
            let _ = handle.await;
        }

        self.wrapper = None;
    }
}

impl Drop for LazySocks5Service {
    fn drop(&mut self) {
        self.shutdown_token.cancel();
    }
}
