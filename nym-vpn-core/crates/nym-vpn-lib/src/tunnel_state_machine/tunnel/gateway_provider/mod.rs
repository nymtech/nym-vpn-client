// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod algorithm;
mod gateway_cache;
mod selector;

use std::{sync::Arc, task::Poll};

use futures::{FutureExt, Stream, StreamExt};
use nym_gateway_directory::{BlacklistedGateways, GatewayClient, NodeIdentity};
use nym_vpn_store::keys::wireguard::WireguardKeysDb;
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    TunnelSettings,
    tunnel::{
        self,
        gateway_provider::{
            algorithm::{SelectAndSend, SelectionAlgorithm},
            gateway_cache::GatewayCache,
        },
    },
};

pub use selector::SelectedGateways;

type SelectionResult = Result<SelectedGateways, tunnel::Error>;
type SelectionResultSender = mpsc::Sender<SelectionResult>;
type SelectedGatewaysStream = Arc<Mutex<ReceiverStream<Result<SelectedGateways, tunnel::Error>>>>;

#[derive(Clone)]
pub struct GatewayProvider<C: GatewayCache> {
    gateway_cache: C,
    tunnel_settings_tx: mpsc::Sender<SelectAndSend>,
    selected_gateways_stream: Option<SelectedGatewaysStream>,
    blacklisted_entry_gateways: BlacklistedGateways,
}

impl<C: GatewayCache> Stream for GatewayProvider<C> {
    type Item = SelectionResult;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let Some(selected_gateways_stream) = self.selected_gateways_stream.as_ref() else {
            return Poll::Pending;
        };
        let Poll::Ready(mut selected_gateways_stream) =
            Box::pin(selected_gateways_stream.lock()).poll_unpin(cx)
        else {
            return Poll::Pending;
        };
        selected_gateways_stream.poll_next_unpin(cx)
    }
}

impl<C: GatewayCache> GatewayProvider<C> {
    pub fn new(
        gateway_cache: C,
        wg_keys_db: WireguardKeysDb,
        shutdown_token: CancellationToken,
    ) -> (Self, JoinHandle<()>) {
        let (tunnel_settings_tx, tunnel_settings_rx) = mpsc::channel(1);
        let blacklisted_entry_gateways = BlacklistedGateways::new();
        let selection_algorithm_handle = tokio::spawn(
            SelectionAlgorithm::new(
                tunnel_settings_rx,
                gateway_cache.clone(),
                blacklisted_entry_gateways.clone(),
                wg_keys_db,
                shutdown_token,
            )
            .run(),
        );

        (
            Self {
                gateway_cache,
                tunnel_settings_tx,
                selected_gateways_stream: None,
                blacklisted_entry_gateways,
            },
            selection_algorithm_handle,
        )
    }

    pub async fn set_tunnel_settings(
        &mut self,
        tunnel_settings: TunnelSettings,
    ) -> Result<(), crate::tunnel_state_machine::Error> {
        // Pre-compute at most 10 different possibilities of selected gateways
        let (selection_tx, selection_rx) = mpsc::channel(10);
        self.selected_gateways_stream =
            Some(Arc::new(Mutex::new(ReceiverStream::new(selection_rx))));
        self.tunnel_settings_tx
            .send(SelectAndSend {
                tunnel_settings,
                selection_tx,
            })
            .await
            .map_err(|_| crate::tunnel_state_machine::Error::GatewayProviderDown)
    }

    pub async fn replace_gateway_client(&self, gateway_client: GatewayClient) {
        self.gateway_cache
            .replace_gateway_client(gateway_client)
            .ok();
        self.gateway_cache.refresh_all().await.ok();
    }

    pub fn clear_blacklisted_entry_gateways(&self) {
        match self.blacklisted_entry_gateways.is_empty() {
            Ok(is_empty) => {
                if !is_empty {
                    tracing::info!("Clearing blacklisted entry gateways");
                    if let Err(e) = self.blacklisted_entry_gateways.clear() {
                        tracing::error!("Failed to clear blacklisted entry gateway list: {e}");
                    }
                }
            }
            Err(e) => tracing::error!("Failed to read blacklisted entry gateway list: {e}"),
        }
    }

    pub fn add_blacklisted_entry_gateway(&self, entry_gateway_identifier: NodeIdentity) {
        if let Err(e) = self
            .blacklisted_entry_gateways
            .add(entry_gateway_identifier)
        {
            tracing::error!(
                "Failed to add gateway {} to blacklisted entry gateway list: {e}",
                entry_gateway_identifier
            );
        } else {
            tracing::warn!(
                "Blacklisted entry gateway {} due to repeated connection failure",
                entry_gateway_identifier
            );
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::time::Duration;

    use nym_gateway_directory::{Gateway, Performance, ScoreValue};
    use nym_vpn_lib_types::{EntryPoint, ExitPoint, TunnelType};
    use tokio::sync::RwLock;

    use crate::tunnel_state_machine::tunnel::gateway_provider::gateway_cache::tests::MockGatewayCache;

    use super::*;

    pub fn default_tunnel_settings() -> TunnelSettings {
        TunnelSettings {
            enable_ipv6: false,
            tunnel_type: TunnelType::Wireguard,
            allow_lan: false,
            enable_ad_blocking: false,
            residential_exit: false,
            enable_lewes_protocol: false,
            mixnet_tunnel_options: Default::default(),
            wireguard_tunnel_options: Default::default(),
            gateway_performance_options: Default::default(),
            mixnet_client_config: None,
            entry_point: Box::new(EntryPoint::Random),
            exit_point: Box::new(ExitPoint::Random),
            dns: Default::default(),
            split_tunnel: Default::default(),
            socks5_proxy_settings: Default::default(),
        }
    }

    pub fn gateway_id_to_gateway(id: &str) -> Gateway {
        Gateway::builder()
            .identity(id.parse().unwrap())
            .performance(Performance {
                last_updated_utc: Default::default(),
                score: ScoreValue::High,
                mixnet_score: ScoreValue::High,
                load: ScoreValue::Low,
                uptime_percentage_last_24_hours: Default::default(),
            })
            .build()
    }

    #[tokio::test]
    async fn empty_stream() {
        let shutdown_token = CancellationToken::new();
        let gateways = Arc::new(RwLock::new(None));
        let (mut gw_provider, handle) = GatewayProvider::new(
            MockGatewayCache::new(gateways),
            WireguardKeysDb::Ephemeral(Default::default()),
            shutdown_token.child_token(),
        );
        // No gateways come out of the stream when there are no tunnel settings
        assert!(
            tokio::time::timeout(Duration::from_millis(100), gw_provider.next())
                .await
                .is_err()
        );
        shutdown_token.cancel();
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn set_and_stream() {
        let shutdown_token = CancellationToken::new();
        let possible_gateways = [
            "2zHiExNRKiCXVKS35SNKtK4apGfZELMpA1jJ2gVevJoz",
            "38zcSsvjXsAX7C28ko2H3Lt55X4TYxfZYkPADxKXZHUj",
        ]
        .map(gateway_id_to_gateway);
        let gateways = Arc::new(RwLock::new(Some(possible_gateways.to_vec())));
        let (mut gw_provider, handle) = GatewayProvider::new(
            MockGatewayCache::new(gateways),
            WireguardKeysDb::Ephemeral(Default::default()),
            shutdown_token.child_token(),
        );
        gw_provider
            .set_tunnel_settings(default_tunnel_settings())
            .await
            .unwrap();
        // check we have "infinite" stream
        for _ in 0..100 {
            gw_provider.next().await.unwrap().unwrap();
        }

        shutdown_token.cancel();
        handle.await.unwrap();
    }
}
