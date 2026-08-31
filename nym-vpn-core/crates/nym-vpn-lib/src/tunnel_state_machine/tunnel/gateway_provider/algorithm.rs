// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{BlacklistedGateways, Location};
use nym_vpn_store::keys::wireguard::WireguardKeysDb;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    TunnelSettings,
    tunnel::gateway_provider::{
        SelectionResultSender, gateway_cache::GatewayCache, geo_ip::GeoIpProvider,
        selector::select_gateways,
    },
};

#[derive(Clone)]
pub struct SelectAndSend {
    pub tunnel_settings: TunnelSettings,
    pub selection_tx: SelectionResultSender,
}

async fn continuous_select<C: GatewayCache>(
    select_and_send: SelectAndSend,
    gateway_cache: C,
    blacklisted_gateways: &BlacklistedGateways,
    device_location: Option<Location>,
    wg_keys_db: &WireguardKeysDb,
) {
    loop {
        // make sure the buffer is not full before deciding on other possible selections
        let Ok(selection_tx) = select_and_send.selection_tx.reserve().await else {
            tracing::debug!("GatewayProvider shut down during selection algorithm");
            return;
        };
        let selection = select_gateways(
            gateway_cache.clone(),
            blacklisted_gateways,
            &select_and_send.tunnel_settings,
            device_location.clone(),
            wg_keys_db,
        )
        .await;
        selection_tx.send(selection);
    }
}

pub struct SelectionAlgorithm<C: GatewayCache> {
    tunnel_settings_rx: mpsc::Receiver<SelectAndSend>,
    gateway_cache: C,
    geo_ip_provider: GeoIpProvider,
    blacklisted_gateways: BlacklistedGateways,
    wg_keys_db: WireguardKeysDb,
    shutdown_token: CancellationToken,
}

impl<C: GatewayCache> SelectionAlgorithm<C> {
    pub fn new(
        tunnel_settings_rx: mpsc::Receiver<SelectAndSend>,
        gateway_cache: C,
        geo_ip_provider: GeoIpProvider,
        blacklisted_gateways: BlacklistedGateways,
        wg_keys_db: WireguardKeysDb,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            tunnel_settings_rx,
            gateway_cache,
            geo_ip_provider,
            blacklisted_gateways,
            wg_keys_db,
            shutdown_token,
        }
    }

    pub async fn run(
        mut self,
        mut latest_tunnel_settings: SelectAndSend,
        mut latest_location: Option<Location>,
    ) {
        loop {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    tracing::info!("SelectionAlgorithm shut down");
                    break;
                }
                Some(new_settings) = self.tunnel_settings_rx.recv() => {
                    latest_tunnel_settings = new_settings;
                }
                new_location = self.geo_ip_provider.new_location() => {
                    latest_location = new_location;
                }
                _ = continuous_select(
                        latest_tunnel_settings.clone(),
                        self.gateway_cache.clone(),
                        &self.blacklisted_gateways,
                        latest_location.clone(),
                        &self.wg_keys_db,
                    ) => {},
            }
        }
        self.wg_keys_db.close().await;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::RwLock;

    use crate::tunnel_state_machine::tunnel::gateway_provider::{
        SelectionResult,
        gateway_cache::tests::MockGatewayCache,
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
        let (_update_location_tx, update_location_rx) = mpsc::unbounded_channel();
        let (selection_tx, _selection_rx) = mpsc::channel(10);
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
            GeoIpProvider::new(update_location_rx),
            BlacklistedGateways::new(),
            WireguardKeysDb::Ephemeral(Default::default()),
            shutdown_token.clone(),
        );
        let handle = tokio::spawn(algo.run(
            SelectAndSend {
                tunnel_settings: default_tunnel_settings(),
                selection_tx,
            },
            None,
        ));

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
