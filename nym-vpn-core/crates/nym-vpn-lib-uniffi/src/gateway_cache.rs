// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{GATEWAY_CACHE, error::VpnError};
use nym_gateway_directory::{GatewayCache, GatewayCacheHandle, GatewayClient};
use nym_offline_monitor::ConnectivityHandle;
use nym_vpn_lib_types::UserAgent;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct UniffiGatewayCacheHandle {
    directory_config: nym_gateway_directory::Config,
    gateway_cache_handle: GatewayCacheHandle,
    join_handle: JoinHandle<()>,
    shutdown_token: CancellationToken,
}

impl UniffiGatewayCacheHandle {
    pub async fn new(
        user_agent: UserAgent,
        connectivity_handle: ConnectivityHandle,
    ) -> Result<Self, VpnError> {
        let shutdown_token = CancellationToken::new();
        let directory_config = make_gateway_config().await;
        let gateway_client = GatewayClient::new(directory_config.clone(), user_agent.into())
            .await
            .map_err(VpnError::internal)?;
        let (gateway_cache_handle, join_handle) = GatewayCache::spawn(
            gateway_client,
            connectivity_handle,
            shutdown_token.child_token(),
        );

        Ok(Self {
            directory_config,
            gateway_cache_handle,
            join_handle,
            shutdown_token,
        })
    }

    pub async fn shutdown_and_wait(self) {
        self.shutdown_token.cancel();

        if let Err(e) = self.join_handle.await {
            tracing::error!("Failed to join on gateway cache handle: {}", e);
        }
    }

    pub fn get_config(&self) -> &nym_gateway_directory::Config {
        &self.directory_config
    }

    pub fn inner(&self) -> GatewayCacheHandle {
        self.gateway_cache_handle.clone()
    }
}

async fn make_gateway_config() -> nym_gateway_directory::Config {
    let network_env = crate::environment::current_environment_details()
        .await
        .unwrap();

    match nym_gateway_directory::Config::new(
        network_env.nyxd_url(),
        network_env.nym_api_urls().unwrap_or_default(),
        network_env.nym_vpn_api_urls().unwrap_or_default(),
        None,
    ) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to create gateway directory config: {e:#?}");
            panic!("Inconsistent network environment");
        }
    }
}

pub async fn init_gateway_cache(
    user_agent: UserAgent,
    connectivity_handle: ConnectivityHandle,
) -> Result<(), VpnError> {
    let mut guard = GATEWAY_CACHE.lock().await;
    match guard.as_ref() {
        Some(_) => Err(VpnError::InvalidStateError {
            details: "Gateway cache is already initialized".to_owned(),
        }),
        None => {
            let gateway_cache_handle =
                UniffiGatewayCacheHandle::new(user_agent, connectivity_handle).await?;
            *guard = Some(gateway_cache_handle);
            Ok(())
        }
    }
}

pub async fn get_gateway_config() -> Result<nym_gateway_directory::Config, VpnError> {
    let guard = GATEWAY_CACHE.lock().await;
    match guard.as_ref() {
        Some(cache_handle) => Ok(cache_handle.get_config().clone()),
        None => Err(VpnError::InvalidStateError {
            details: "Gateway cache is not initialized".to_owned(),
        }),
    }
}

pub async fn get_gateway_cache_handle() -> Result<GatewayCacheHandle, VpnError> {
    let guard = GATEWAY_CACHE.lock().await;
    match guard.as_ref() {
        Some(cache_handle) => Ok(cache_handle.inner()),
        None => Err(VpnError::InvalidStateError {
            details: "Gateway cache is not initialized".to_owned(),
        }),
    }
}

pub async fn stop_gateway_cache() -> Result<(), VpnError> {
    let mut guard = GATEWAY_CACHE.lock().await;
    match guard.take() {
        Some(cache_handle) => {
            cache_handle.shutdown_and_wait().await;
            Ok(())
        }
        None => Err(VpnError::InvalidStateError {
            details: "Gateway cache is not initialized".to_owned(),
        }),
    }
}
