// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! SOCKS5 backend that manages the Nym mixnet client lifecycle

use nym_sdk::mixnet::{MixnetClientBuilder, Socks5, Socks5MixnetClient, StoragePaths};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

/// Backend that manages the Nym mixnet SOCKS5 client
pub struct Socks5Backend {
    /// Data directory for mixnet client state
    data_path: PathBuf,
    /// Network requester address (recipient format: client_address@gateway_identity)
    network_requester_address: String,
    /// Internal SOCKS5 listen address (e.g., "127.0.0.1:10801")
    internal_listen_address: SocketAddr,
    /// The connected SOCKS5 mixnet client (if connected)
    client: Option<Socks5MixnetClient>,
    /// Cancellation token for shutdown
    cancel_token: CancellationToken,
}

/// Errors from the SOCKS5 backend
#[derive(Debug, thiserror::Error)]
pub enum Socks5BackendError {
    #[error("Failed to initialize mixnet client: {0}")]
    MixnetInitError(String),

    #[error("Failed to connect to network requester: {0}")]
    ConnectionError(String),

    #[error("Client is not connected")]
    NotConnected,

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl Socks5Backend {
    /// Create a new SOCKS5 backend (does not start the mixnet)
    pub fn new(
        data_path: PathBuf,
        internal_listen_address: SocketAddr,
        network_requester_address: String,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            data_path,
            internal_listen_address,
            network_requester_address,
            client: None,
            cancel_token,
        }
    }

    /// Start the SOCKS5 mixnet client on the internal port
    /// This initializes the Nym mixnet and starts the internal SOCKS5 server
    pub async fn start(&mut self) -> Result<(), Socks5BackendError> {
        if self.client.is_some() {
            info!("SOCKS5 backend already started");
            return Ok(());
        }

        info!(
            "Starting SOCKS5 backend on internal address: {}",
            self.internal_listen_address
        );
        info!(
            "Network requester address: {}",
            self.network_requester_address
        );

        // Create SOCKS5 configuration pointing to the network requester
        let mut socks5_config = Socks5::new("J2oXYjn8fRMz9MKFUibatjCTvxvQbVa2r5Uxp7X47aL3.BdyaZLL1cpSeZKS3AfpzXEXFE67WkuoEsSmxwvo5tTVN@3hWtFJbVVPbZZ9iNZuSHPnShHG5AUiFpTPnvJmUibNp9".to_string());

        // CRITICAL: Enable anonymous sending so responses can come back through the mixnet using SURBs
        // By default this is false, which means responses won't be routed back to the SOCKS5 client
        // Setting this to true enables Single Use Reply Blocks (SURBs) for anonymous replies
        socks5_config.send_anonymously = true;
        info!("Enabled anonymous sending (SURBs) for SOCKS5 responses");

        // Set the internal bind address for the SOCKS5 server
        socks5_config.bind_address = self.internal_listen_address;
        info!(
            "SOCKS5 server will bind to internal address: {}",
            self.internal_listen_address
        );

        // Setup storage paths for credential storage (shared with main VPN client)
        let storage_paths = StoragePaths::new_from_dir(&self.data_path)
            .map_err(|e| Socks5BackendError::StorageError(e.to_string()))?;

        info!("Building mixnet client with SOCKS5 configuration...");
        // Build the mixnet client with SOCKS5 configuration
        let mixnet_client = MixnetClientBuilder::new_with_default_storage(storage_paths)
            .await
            .map_err(|e| {
                error!("Failed to create mixnet client builder: {}", e);
                Socks5BackendError::MixnetInitError(e.to_string())
            })?
            .socks5_config(socks5_config)
            .build()
            .map_err(|e| {
                error!("Failed to build mixnet client: {}", e);
                Socks5BackendError::MixnetInitError(e.to_string())
            })?;

        // Connect to the mixnet via SOCKS5
        info!("Connecting to mixnet via SOCKS5...");
        info!("This will spawn the internal SOCKS5 server and establish mixnet connection...");
        let mixnet_client = mixnet_client
            .connect_to_mixnet_via_socks5()
            .await
            .map_err(|e| {
                error!("Failed to connect to mixnet via SOCKS5: {}", e);
                Socks5BackendError::ConnectionError(e.to_string())
            })?;

        info!("SOCKS5 mixnet backend connected successfully");
        info!("Client Nym address: {}", mixnet_client.nym_address());
        info!(
            "Internal SOCKS5 server listening on: {}",
            self.internal_listen_address
        );

        self.client = Some(mixnet_client);
        Ok(())
    }

    /// Stop the SOCKS5 backend and disconnect from mixnet
    pub async fn stop(&mut self) {
        info!("Stopping SOCKS5 mixnet backend");

        if let Some(client) = self.client.take() {
            client.disconnect().await;
            debug!("SOCKS5 mixnet backend disconnected");
        }

        self.cancel_token.cancel();
    }

    /// Check if the backend is running
    pub fn is_running(&self) -> bool {
        self.client.is_some()
    }

    /// Get the internal SOCKS5 listen address
    pub fn internal_address(&self) -> SocketAddr {
        self.internal_listen_address
    }

    /// Get the client's Nym address (if connected)
    pub fn nym_address(&self) -> Option<String> {
        self.client.as_ref().map(|c| c.nym_address().to_string())
    }
}

impl Drop for Socks5Backend {
    fn drop(&mut self) {
        debug!("Dropping SOCKS5 backend");
        self.cancel_token.cancel();
    }
}
