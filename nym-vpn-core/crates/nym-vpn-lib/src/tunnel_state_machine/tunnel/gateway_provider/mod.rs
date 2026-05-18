// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod algorithm;
mod error;
mod gateway_cache;
mod geo_ip;
mod independence;
mod selector;

use std::{sync::Arc, task::Poll, time::Duration};

use futures::{FutureExt as _, Stream, StreamExt as FuturesStreamExt};
use nym_gateway_directory::{BlacklistedGateways, GatewayClient, NodeIdentity};
use nym_vpn_lib_types::TentativeGateways;
use nym_vpn_store::keys::wireguard::WireguardKeysDb;
use tokio::{
    sync::{Mutex, RwLock, mpsc, oneshot},
    task::JoinHandle,
};
use tokio_stream::{adapters::Peekable, wrappers::ReceiverStream};
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    TunnelSettings,
    tunnel::gateway_provider::{
        algorithm::{SelectAndSend, SelectionAlgorithm},
        gateway_cache::GatewayCache,
        geo_ip::{FetcherCommand, GeoIpClient, GeoIpFetcher, GeoIpProvider, QueryControl},
    },
};

pub use error::GatewayProviderError;
pub use selector::SelectedGateways;

type SelectionResult = Result<SelectedGateways, GatewayProviderError>;
type SelectionResultSender = mpsc::Sender<SelectionResult>;
type SelectedGatewaysStream =
    Arc<Mutex<Peekable<ReceiverStream<Result<SelectedGateways, GatewayProviderError>>>>>;

#[derive(Clone)]
pub struct GatewayProvider<C: GatewayCache> {
    gateway_cache: C,
    tunnel_settings_tx: mpsc::Sender<SelectAndSend>,
    selected_gateways_stream: SelectedGatewaysStream,
    blacklisted_entry_gateways: BlacklistedGateways,
    query_control: Arc<RwLock<QueryControl>>,
    query_control_tx: mpsc::UnboundedSender<FetcherCommand>,
}

impl<C: GatewayCache> Stream for GatewayProvider<C> {
    type Item = SelectionResult;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let Poll::Ready(mut selected_gateways_stream) =
            Box::pin(self.selected_gateways_stream.lock()).poll_unpin(cx)
        else {
            return Poll::Pending;
        };
        selected_gateways_stream.poll_next_unpin(cx)
    }
}

impl<C: GatewayCache> GatewayProvider<C> {
    pub fn new(
        gateway_cache: C,
        geo_ip_client: impl GeoIpClient,
        tunnel_settings: TunnelSettings,
        wg_keys_db: WireguardKeysDb,
        shutdown_token: CancellationToken,
    ) -> (Self, JoinHandle<()>) {
        let (tunnel_settings_tx, tunnel_settings_rx) = mpsc::channel(1);
        let blacklisted_entry_gateways = BlacklistedGateways::new();
        let (query_control_tx, query_control_rx) = mpsc::unbounded_channel();
        let (update_location_tx, update_location_rx) = mpsc::unbounded_channel();

        let geo_ip_provider = GeoIpProvider::new(update_location_rx);
        let geo_ip_fetcher = GeoIpFetcher::new(
            tunnel_settings
                .gateway_selection_algorithm_config
                .enable_geo_location,
            Box::new(geo_ip_client),
            query_control_rx,
            update_location_tx,
            shutdown_token.child_token(),
        );
        let query_control = geo_ip_fetcher.query_control();
        let geo_ip_fetcher_handle = tokio::spawn(geo_ip_fetcher.run());

        // Pre-compute at most 10 different possibilities of selected gateways
        let (selection_tx, selection_rx) = mpsc::channel(10);
        let selection_algorithm_handle = tokio::spawn(
            SelectionAlgorithm::new(
                tunnel_settings_rx,
                gateway_cache.clone(),
                geo_ip_provider,
                blacklisted_entry_gateways.clone(),
                wg_keys_db,
                shutdown_token,
            )
            .run(SelectAndSend {
                tunnel_settings,
                selection_tx,
            }),
        );
        let selected_gateways_stream = Arc::new(Mutex::new(tokio_stream::StreamExt::peekable(
            ReceiverStream::new(selection_rx),
        )));

        // unify the two local handles into one
        let gateway_provider_handle = tokio::spawn(async {
            let _ = geo_ip_fetcher_handle.await;
            let _ = selection_algorithm_handle.await;
        });

        (
            Self {
                gateway_cache,
                tunnel_settings_tx,
                selected_gateways_stream,
                blacklisted_entry_gateways,
                query_control_tx,
                query_control,
            },
            gateway_provider_handle,
        )
    }

    pub async fn tentative_gateways(&self) -> TentativeGateways {
        // use a very small timeout because we actually expect the stream to always have a value ready
        // if it doesn't, there's probably some error, but we shouldn't block the RPC call anyway
        match tokio::time::timeout(
            Duration::from_millis(10),
            self.selected_gateways_stream.lock().await.peek(),
        )
        .await
        {
            Ok(Some(Ok(selected_gateways))) => TentativeGateways::Selected {
                entry: Box::new(selected_gateways.entry_gateway().clone().into()),
                exit: Box::new(selected_gateways.exit_gateway().clone().into()),
            },
            Ok(Some(Err(GatewayProviderError::NeedsRelaxedIndependenceCriteria))) => {
                TentativeGateways::NeedsRelaxedIndependenceCriteria
            }
            Ok(Some(Err(_))) | Ok(None) => TentativeGateways::NoGatewaysAvailable,
            Err(_) => TentativeGateways::NoGatewaysAvailable,
        }
    }

    async fn inner_set_tunnel_settings(
        tunnel_settings_tx: &mpsc::Sender<SelectAndSend>,
        tunnel_settings: TunnelSettings,
    ) -> Result<
        Peekable<ReceiverStream<Result<SelectedGateways, GatewayProviderError>>>,
        crate::tunnel_state_machine::Error,
    > {
        // Pre-compute at most 10 different possibilities of selected gateways
        let (selection_tx, selection_rx) = mpsc::channel(10);
        tunnel_settings_tx
            .send(SelectAndSend {
                tunnel_settings,
                selection_tx,
            })
            .await
            .map_err(|_| crate::tunnel_state_machine::Error::GatewayProviderDown)?;
        Ok(tokio_stream::StreamExt::peekable(ReceiverStream::new(
            selection_rx,
        )))
    }

    async fn start_fetching_again(&self) {
        if self.query_control_tx.send(FetcherCommand::Fetch).is_err() {
            tracing::warn!("Could send fetch control message to geo ip fetcher");
        }
    }

    async fn abort_any_fetch(&self) {
        let (done_tx, done_rx) = oneshot::channel();
        if self
            .query_control_tx
            .send(FetcherCommand::Abort(done_tx))
            .is_err()
        {
            tracing::warn!("Could send abort control message to geo ip fetcher");
        }
        if done_rx.await.is_err() {
            tracing::warn!("Geo ip fetcher did not respond to abort command");
        }
    }

    /// Set if geo-location should be used by the algorithm, based on user's
    /// preference
    async fn set_enabled_geo_location(&self, enabled: bool) {
        let do_not_query = {
            let mut query_control = self.query_control.write().await;
            query_control.set_enabled(enabled);
            query_control.do_not_query()
        };
        if do_not_query {
            self.abort_any_fetch().await;
        } else {
            self.start_fetching_again().await;
        }
    }

    /// Set the activation for geo-location based on connection status, to avoid
    /// false locations being used.
    /// Being active means we try to query the location from the API. We want to
    /// deactivate this in certain states of TSM, when the queries are proxied.
    pub async fn set_active_geo_location(&self, active: bool) {
        let do_not_query = {
            let mut query_control = self.query_control.write().await;
            query_control.set_active(active);
            query_control.do_not_query()
        };
        if do_not_query {
            self.abort_any_fetch().await;
        } else {
            self.start_fetching_again().await;
        }
    }

    pub async fn set_tunnel_settings(
        &self,
        tunnel_settings: TunnelSettings,
    ) -> Result<(), crate::tunnel_state_machine::Error> {
        self.set_enabled_geo_location(
            tunnel_settings
                .gateway_selection_algorithm_config
                .enable_geo_location,
        )
        .await;
        *self.selected_gateways_stream.lock().await =
            Self::inner_set_tunnel_settings(&self.tunnel_settings_tx, tunnel_settings).await?;
        Ok(())
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
    use nym_vpn_lib_types::{EntryPoint, ExitPoint, GatewayIndependence, TunnelType};
    use tokio::sync::RwLock;

    use crate::tunnel_state_machine::tunnel::gateway_provider::{
        gateway_cache::tests::MockGatewayCache, geo_ip::tests::MockGeoIpClient,
    };

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
            gateway_selection_algorithm_config: Default::default(),
            geo_exclusion_settings: Default::default(),
            gateway_independence: GatewayIndependence {
                different_asn: false,
                different_node_family: false,
            },
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
    async fn error_stream() {
        let shutdown_token = CancellationToken::new();
        let gateways = Arc::new(RwLock::new(None));
        let mut tunnel_settings = default_tunnel_settings();
        tunnel_settings
            .gateway_selection_algorithm_config
            .enable_geo_location = false;
        let (mut gw_provider, handle) = GatewayProvider::new(
            MockGatewayCache::new(gateways),
            MockGeoIpClient::new(),
            tunnel_settings,
            WireguardKeysDb::Ephemeral(Default::default()),
            shutdown_token.child_token(),
        );
        // No gateways come out of the stream when there are no gateways to select from
        assert!(
            tokio::time::timeout(Duration::from_millis(100), gw_provider.next())
                .await
                .unwrap()
                .unwrap()
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
        let mut tunnel_settings = default_tunnel_settings();
        tunnel_settings
            .gateway_selection_algorithm_config
            .enable_geo_location = false;
        let (mut gw_provider, handle) = GatewayProvider::new(
            MockGatewayCache::new(gateways),
            MockGeoIpClient::new(),
            tunnel_settings,
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
