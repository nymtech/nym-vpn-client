// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_config::defaults::NymNetworkDetails;
use nym_sdk::mixnet::{
    MixnetClient, MixnetClientBuilder, MixnetClientSender, MixnetMessageSender, NodeIdentity,
    Recipient, StoragePaths,
};
#[cfg(target_family = "unix")]
use std::os::fd::RawFd;
#[cfg(not(target_family = "unix"))]
use std::os::raw::c_int as RawFd;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};

use crate::{
    credentials::check_imported_credential,
    error::{Error, Result},
    MixnetClientConfig,
};

#[derive(Clone)]
pub struct SharedMixnetClient(Arc<tokio::sync::Mutex<Option<MixnetClient>>>);

impl SharedMixnetClient {
    pub fn new(mixnet_client: MixnetClient) -> Self {
        Self(Arc::new(tokio::sync::Mutex::new(Some(mixnet_client))))
    }

    pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, Option<MixnetClient>> {
        self.0.lock().await
    }

    pub async fn nym_address(&self) -> Recipient {
        *self.lock().await.as_ref().unwrap().nym_address()
    }

    pub async fn split_sender(&self) -> MixnetClientSender {
        self.lock().await.as_ref().unwrap().split_sender()
    }

    pub async fn gateway_ws_fd(&self) -> Option<RawFd> {
        self.lock()
            .await
            .as_ref()
            .unwrap()
            .gateway_connection()
            .gateway_ws_fd
    }

    pub async fn send(&self, msg: nym_sdk::mixnet::InputMessage) -> Result<()> {
        self.lock().await.as_mut().unwrap().send(msg).await?;
        Ok(())
    }

    pub async fn disconnect(self) -> Self {
        let handle = self.lock().await.take().unwrap();
        handle.disconnect().await;
        self
    }

    pub fn inner(&self) -> Arc<tokio::sync::Mutex<Option<MixnetClient>>> {
        self.0.clone()
    }
}

fn true_to_enabled(val: bool) -> &'static str {
    if val {
        "enabled"
    } else {
        "disabled"
    }
}

fn true_to_disabled(val: bool) -> &'static str {
    if val {
        "disabled"
    } else {
        "enabled"
    }
}

fn apply_mixnet_client_config(
    mixnet_client_config: &MixnetClientConfig,
    debug_config: &mut nym_client_core::config::DebugConfig,
) {
    let MixnetClientConfig {
        enable_poisson_rate,
        disable_background_cover_traffic,
        enable_credentials_mode: _enable_credentials_mode,
        min_mixnode_performance,
        min_gateway_performance,
    } = mixnet_client_config;

    // Disable Poisson rate limiter by default
    info!(
        "mixnet client poisson rate limiting: {}",
        true_to_enabled(*enable_poisson_rate)
    );
    debug_config
        .traffic
        .disable_main_poisson_packet_distribution = !enable_poisson_rate;

    info!(
        "mixnet client background loop cover traffic stream: {}",
        true_to_disabled(*disable_background_cover_traffic)
    );
    debug_config.cover_traffic.disable_loop_cover_traffic_stream =
        *disable_background_cover_traffic;

    if let Some(min_mixnode_performance) = min_mixnode_performance {
        debug_config.topology.minimum_mixnode_performance = *min_mixnode_performance;
    }
    info!(
        "mixnet client minimum mixnode performance: {}",
        debug_config.topology.minimum_mixnode_performance,
    );

    if let Some(min_gateway_performance) = min_gateway_performance {
        debug_config.topology.minimum_gateway_performance = *min_gateway_performance;
    }
    info!(
        "mixnet client minimum gateway performance: {}",
        debug_config.topology.minimum_gateway_performance,
    );
}

pub(crate) async fn setup_mixnet_client(
    mixnet_entry_gateway: &NodeIdentity,
    mixnet_client_key_storage_path: &Option<PathBuf>,
    mut task_client: nym_task::TaskClient,
    enable_wireguard: bool,
    mixnet_client_config: MixnetClientConfig,
) -> Result<SharedMixnetClient> {
    let mut debug_config = nym_client_core::config::DebugConfig::default();
    apply_mixnet_client_config(&mixnet_client_config, &mut debug_config);

    debug!(
        "mixnet client has wireguard_mode: {}",
        true_to_enabled(enable_wireguard)
    );

    let user_agent = nym_bin_common::bin_info_owned!().into();

    let mixnet_client = if let Some(path) = mixnet_client_key_storage_path {
        debug!("Using custom key storage path: {:?}", path);

        let gateway_id = mixnet_entry_gateway.to_base58_string();
        if let Err(err) = check_imported_credential(path.to_path_buf(), &gateway_id).await {
            // UGLY: flow needs to restructured to sort this out, but I don't want to refactor all
            // that just before release.
            task_client.disarm();
            return Err(Error::InvalidCredential {
                reason: err,
                path: path.to_path_buf(),
                gateway_id,
            });
        };

        let key_storage_path = StoragePaths::new_from_dir(path)?;
        MixnetClientBuilder::new_with_default_storage(key_storage_path)
            .await?
            .with_wireguard_mode(enable_wireguard)
            .with_user_agent(user_agent)
            .request_gateway(mixnet_entry_gateway.to_string())
            .network_details(NymNetworkDetails::new_from_env())
            .debug_config(debug_config)
            .custom_shutdown(task_client)
            .credentials_mode(mixnet_client_config.enable_credentials_mode)
            .build()?
            .connect_to_mixnet()
            .await?
    } else {
        debug!("Using ephemeral key storage");
        MixnetClientBuilder::new_ephemeral()
            .with_wireguard_mode(enable_wireguard)
            .with_user_agent(user_agent)
            .request_gateway(mixnet_entry_gateway.to_string())
            .network_details(NymNetworkDetails::new_from_env())
            .debug_config(debug_config)
            .custom_shutdown(task_client)
            .credentials_mode(mixnet_client_config.enable_credentials_mode)
            .build()?
            .connect_to_mixnet()
            .await?
    };

    Ok(SharedMixnetClient::new(mixnet_client))
}
