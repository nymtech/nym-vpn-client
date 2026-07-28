// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_favorites::RecentGatewayCache;
use tokio::{sync::Mutex, task::JoinHandle};
use tokio_util::sync::{CancellationToken, DropGuard};

use nym_gateway_directory::{
    Error as GatewayDirectoryError, GatewayCache, GatewayCacheHandle, GatewayClient, GatewayList,
};
use nym_vpn_lib_types::{Gateway, GatewayType, UserAgent};

use crate::{environment::NymEnvironment, error::VpnError, offline_monitor::NymOfflineMonitor};

#[derive(uniffi::Object)]
pub struct NymGatewayCache {
    gateway_cache_handle: GatewayCacheHandle,
    join_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    shutdown_drop_guard: Arc<Mutex<Option<DropGuard>>>,
}

#[async_trait::async_trait]
impl RecentGatewayCache for &NymGatewayCache {
    async fn lookup_gateways(
        &self,
        gw_type: nym_gateway_directory::GatewayType,
    ) -> Result<GatewayList, GatewayDirectoryError> {
        self.inner_get_gateways(gw_type).await
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl NymGatewayCache {
    // Keep this method async because spawn needs runtime!
    #[uniffi::constructor]
    pub async fn new(
        user_agent: UserAgent,
        environment: Arc<NymEnvironment>,
        offline_monitor: Arc<NymOfflineMonitor>,
    ) -> Result<Self, VpnError> {
        let shutdown_token = CancellationToken::new();

        let network_env = environment.inner();
        let nym_api_urls = network_env.nym_api_urls().unwrap_or_default();
        let nym_vpn_api_urls = network_env.nym_vpn_api_urls().unwrap_or_default();

        // Config::new() will error if nym_api_urls or nym_vpn_api_urls are empty
        let directory_config = nym_gateway_directory::Config::new(
            network_env.nyxd_url(),
            nym_api_urls,
            nym_vpn_api_urls,
            None,
        )
        .map_err(|e| VpnError::InternalError {
            details: format!("Failed to create config: {e:#?}"),
        })?;
        let gateway_client =
            GatewayClient::new(directory_config, user_agent.into()).map_err(VpnError::internal)?;
        let (gateway_cache_handle, join_handle) = GatewayCache::spawn(
            gateway_client,
            offline_monitor.inner(),
            shutdown_token.child_token(),
        );

        Ok(Self {
            gateway_cache_handle,
            join_handle: Arc::new(Mutex::new(Some(join_handle))),
            shutdown_drop_guard: Arc::new(Mutex::new(Some(shutdown_token.drop_guard()))),
        })
    }

    pub async fn shutdown_and_wait(&self) {
        let Some(join_handle) = self.join_handle.lock().await.take() else {
            return;
        };

        drop(self.shutdown_drop_guard.lock().await.take());

        if let Err(e) = join_handle.await {
            tracing::error!("Failed to join on gateway cache handle: {}", e);
        }
    }

    pub async fn get_gateways(&self, gw_type: GatewayType) -> Result<Vec<Gateway>, VpnError> {
        self.inner_get_gateways(gw_type.into())
            .await
            .map(|gateways| {
                gateways
                    .into_inner()
                    .into_iter()
                    .map(Gateway::from)
                    .collect()
            })
            .map_err(|err| VpnError::NetworkConnectionError {
                details: err.to_string(),
            })
    }
}

impl NymGatewayCache {
    pub fn inner(&self) -> GatewayCacheHandle {
        self.gateway_cache_handle.clone()
    }

    async fn inner_get_gateways(
        &self,
        gw_type: nym_gateway_directory::GatewayType,
    ) -> Result<GatewayList, GatewayDirectoryError> {
        self.inner().lookup_gateways(gw_type).await
    }
}
