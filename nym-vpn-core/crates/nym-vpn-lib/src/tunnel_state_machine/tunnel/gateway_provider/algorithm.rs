// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use futures::future::pending;
use nym_gateway_directory::{BlacklistedGateways, Location};
use nym_vpn_store::keys::wireguard::WireguardKeysDb;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    TunnelSettings,
    tunnel::{
        self,
        gateway_provider::{
            SelectionResultSender, gateway_cache::GatewayCache, geo_ip::GeoIpProvider,
            selector::select_gateways,
        },
    },
};

const GEO_IP_UPDATE_INTERVAL: Duration = Duration::from_hours(1);

#[derive(Clone)]
pub struct SelectAndSend {
    pub tunnel_settings: TunnelSettings,
    pub selection_tx: SelectionResultSender,
}

async fn continuous_select<C: GatewayCache>(
    select_and_send: Option<SelectAndSend>,
    gateway_cache: C,
    blacklisted_entry_gateways: &BlacklistedGateways,
    device_location: Option<Location>,
    wg_keys_db: WireguardKeysDb,
) {
    let Some(SelectAndSend {
        tunnel_settings,
        selection_tx,
    }) = select_and_send
    else {
        // not settings so nothing to send yet
        return pending().await;
    };
    loop {
        // make sure the buffer is not full before deciding on other possible selections
        let Ok(selection_tx) = selection_tx.reserve().await else {
            tracing::debug!("GatewayProvider shut down during selection algorithm");
            return;
        };
        let selection = select_gateways(
            gateway_cache.clone(),
            blacklisted_entry_gateways,
            &tunnel_settings,
            device_location.clone(),
            wg_keys_db.clone(),
        )
        .await
        .map_err(|err| tunnel::Error::SelectGateways(Box::new(err)));
        selection_tx.send(selection);
    }
}

pub struct SelectionAlgorithm<C: GatewayCache> {
    tunnel_settings_rx: mpsc::Receiver<SelectAndSend>,
    gateway_cache: C,
    geo_ip_provider: GeoIpProvider,
    blacklisted_entry_gateways: BlacklistedGateways,
    wg_keys_db: WireguardKeysDb,
    shutdown_token: CancellationToken,
}

impl<C: GatewayCache> SelectionAlgorithm<C> {
    pub fn new(
        tunnel_settings_rx: mpsc::Receiver<SelectAndSend>,
        gateway_cache: C,
        geo_ip_provider: GeoIpProvider,
        blacklisted_entry_gateways: BlacklistedGateways,
        wg_keys_db: WireguardKeysDb,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            tunnel_settings_rx,
            gateway_cache,
            geo_ip_provider,
            blacklisted_entry_gateways,
            wg_keys_db,
            shutdown_token,
        }
    }

    pub async fn run(mut self) {
        let mut latest_tunnel_settings = None;
        let update_timer = tokio::time::sleep(GEO_IP_UPDATE_INTERVAL);
        tokio::pin!(update_timer);
        loop {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    tracing::info!("SelectionAlgorithm shut down");
                    return;
                }
                _ = &mut update_timer => {
                    if let Err(err) = self.geo_ip_provider.update().await {
                        tracing::warn!("Could not update the location on timer for determining gateway proximity: {err}");
                    }
                    update_timer.set(tokio::time::sleep(GEO_IP_UPDATE_INTERVAL));
                }
                settings = self.tunnel_settings_rx.recv() => {
                    if settings.is_none() {
                        tracing::debug!(
                            "GatewayProvider shut down before starting the selection algorithm"
                        );
                        return;
                    };
                    if let Err(err) = self.geo_ip_provider.update().await {
                        tracing::warn!("Could not update the locationon new settings for determining gateway proximity: {err}");
                    }
                    // store the received tunnel settings received for the next loop iteration
                    latest_tunnel_settings = settings;
                }
                // consume the tunnel settings received in the previous loop iteration
                _ = continuous_select(
                        latest_tunnel_settings.take(),
                        self.gateway_cache.clone(),
                        &self.blacklisted_entry_gateways,
                        self.geo_ip_provider.latest_location(),
                        self.wg_keys_db.clone(),
                    ) => {},
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, atomic::AtomicBool};

    use tokio::sync::RwLock;

    use crate::tunnel_state_machine::tunnel::gateway_provider::{
        SelectionResult,
        gateway_cache::tests::MockGatewayCache,
        geo_ip::tests::MockGeoIpClient,
        tests::{default_tunnel_settings, gateway_id_to_gateway},
    };

    use super::*;

    async fn reset_default_settings(
        tunnel_settings_tx: &mpsc::Sender<SelectAndSend>,
    ) -> mpsc::Receiver<SelectionResult> {
        let (selection_tx, selection_rx) = mpsc::channel(1);
        tunnel_settings_tx
            .send(SelectAndSend {
                tunnel_settings: default_tunnel_settings(),
                selection_tx,
            })
            .await
            .unwrap();
        selection_rx
    }

    #[tokio::test]
    async fn run_algo() {
        let (tunnel_settings_tx, tunnel_settings_rx) = mpsc::channel(1);
        let shutdown_token = CancellationToken::new();
        let possible_gateways_ids = [
            "2zHiExNRKiCXVKS35SNKtK4apGfZELMpA1jJ2gVevJoz",
            "38zcSsvjXsAX7C28ko2H3Lt55X4TYxfZYkPADxKXZHUj",
        ];
        let possible_gateways = possible_gateways_ids.map(gateway_id_to_gateway);

        let gateways = Arc::new(RwLock::new(None));
        let algo = SelectionAlgorithm::new(
            tunnel_settings_rx,
            MockGatewayCache::new(gateways.clone()),
            GeoIpProvider::new(
                MockGeoIpClient::new(),
                Arc::new(AtomicBool::new(true)),
                Arc::new(AtomicBool::new(true)),
            )
            .await,
            BlacklistedGateways::new(),
            WireguardKeysDb::Ephemeral(Default::default()),
            shutdown_token.clone(),
        );
        let handle = tokio::spawn(algo.run());

        // set some default settings
        let mut selection_rx = reset_default_settings(&tunnel_settings_tx).await;
        // error when no gateway available
        *gateways.write().await = Some(vec![]);
        assert!(selection_rx.recv().await.unwrap().is_err());

        // re-set some default settings because there are continuous retries with errors in the stream
        let mut selection_rx = reset_default_settings(&tunnel_settings_tx).await;
        // error when not enough gateways
        *gateways.write().await = Some(vec![possible_gateways[0].clone()]);
        assert!(selection_rx.recv().await.unwrap().is_err());

        // re-set some default settings because there are continuous retries with errors in the stream
        let mut selection_rx = reset_default_settings(&tunnel_settings_tx).await;
        // 2 gateways should be minimally functional
        *gateways.write().await = Some(possible_gateways.to_vec());
        // check that the stream is infinite, as long as there are valid available choices
        for _ in 0..100 {
            let selected_gateways = selection_rx.recv().await.unwrap().unwrap();
            assert!(
                possible_gateways_ids.contains(
                    &selected_gateways
                        .entry_gateway()
                        .identity()
                        .to_base58_string()
                        .as_str()
                )
            );
            assert!(
                possible_gateways_ids.contains(
                    &selected_gateways
                        .exit_gateway()
                        .identity()
                        .to_base58_string()
                        .as_str()
                )
            );
        }

        shutdown_token.cancel();
        handle.await.unwrap();
    }
}
