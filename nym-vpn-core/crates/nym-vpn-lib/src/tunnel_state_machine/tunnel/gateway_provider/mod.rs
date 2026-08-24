// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod algorithm;
mod error;
mod gateway_cache;
mod geo_ip;
mod independence;
mod selector;
#[cfg(test)]
mod tests;

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
    latest_tunnel_settings: Arc<Mutex<TunnelSettings>>,
    tunnel_settings_tx: mpsc::Sender<SelectAndSend>,
    selected_gateways_stream: SelectedGatewaysStream,
    blacklisted_gateways: BlacklistedGateways,
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
        let latest_tunnel_settings = Arc::new(Mutex::new(tunnel_settings.clone()));
        let (tunnel_settings_tx, tunnel_settings_rx) = mpsc::channel(1);
        let blacklisted_gateways = BlacklistedGateways::new();
        let (query_control_tx, query_control_rx) = mpsc::unbounded_channel();
        let (update_location_tx, update_location_rx) = mpsc::unbounded_channel();

        let mut geo_ip_provider = GeoIpProvider::new(update_location_rx);
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
        let gateway_cache_clone = gateway_cache.clone();
        let blacklisted_gateways_clone = blacklisted_gateways.clone();
        let selection_algorithm_handle = tokio::spawn(async move {
            let latest_location = if tunnel_settings
                .gateway_selection_algorithm_config
                .enable_geo_location
            {
                shutdown_token
                    .run_until_cancelled(geo_ip_provider.initial_location())
                    .await
                    .flatten()
            } else {
                None
            };
            SelectionAlgorithm::new(
                tunnel_settings_rx,
                gateway_cache_clone,
                geo_ip_provider,
                blacklisted_gateways_clone,
                wg_keys_db,
                shutdown_token,
            )
            .run(
                SelectAndSend {
                    tunnel_settings,
                    selection_tx,
                },
                latest_location,
            )
            .await
        });
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
                latest_tunnel_settings,
                tunnel_settings_tx,
                selected_gateways_stream,
                blacklisted_gateways,
                query_control_tx,
                query_control,
            },
            gateway_provider_handle,
        )
    }

    pub async fn tentative_gateways(&self) -> TentativeGateways {
        // In steady state the stream always has a value buffered, so this peek
        // returns immediately. However, `set_tunnel_settings` (triggered on every
        // connect press via `set_gateway_independence`) swaps in a brand-new,
        // empty stream and asks the algorithm to recompute. When this RPC is
        // queried in that window we must wait for the first fresh selection to
        // land rather than reporting `NoGatewaysAvailable` against an empty
        // stream. The wait is still bounded: a genuinely failing selection (e.g.
        // an empty gateway pool) yields an `Err` quickly, so this only ever
        // blocks while a real selection is in flight.
        match tokio::time::timeout(
            Duration::from_millis(100),
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

    /// Discard gateway selections pre-computed with state that has since gone
    /// stale (e.g. while offline) and start computing fresh ones with the
    /// latest tunnel settings.
    pub async fn reset_selection_stream(&self) -> Result<(), crate::tunnel_state_machine::Error> {
        let (latest_tunnel_settings, mut selected_gateways_stream) = (
            self.latest_tunnel_settings.lock().await,
            self.selected_gateways_stream.lock().await,
        );
        *selected_gateways_stream = Self::inner_set_tunnel_settings(
            &self.tunnel_settings_tx,
            latest_tunnel_settings.clone(),
        )
        .await?;
        Ok(())
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

        let (mut latest_tunnel_settings, mut selected_gateways_stream) = (
            self.latest_tunnel_settings.lock().await,
            self.selected_gateways_stream.lock().await,
        );
        *latest_tunnel_settings = tunnel_settings.clone();
        *selected_gateways_stream =
            Self::inner_set_tunnel_settings(&self.tunnel_settings_tx, tunnel_settings).await?;
        Ok(())
    }

    pub async fn replace_gateway_client(&self, gateway_client: GatewayClient) {
        self.gateway_cache
            .replace_gateway_client(gateway_client)
            .ok();
        self.gateway_cache.refresh_all().await.ok();
    }

    pub fn set_gateway_cache_paused(&self, paused: bool) {
        if let Err(e) = self.gateway_cache.set_paused(paused) {
            tracing::warn!(
                "Failed to {} the gateway cache: {e}",
                if paused { "pause" } else { "resume" }
            );
        }
    }

    pub fn blacklisted_gateways(&self) -> BlacklistedGateways {
        self.blacklisted_gateways.clone()
    }

    pub async fn add_blacklisted_gateway(&self, gateway_identifier: NodeIdentity) {
        if let Err(e) = self.blacklisted_gateways.add(gateway_identifier) {
            tracing::error!(
                "Failed to add gateway {} to blacklisted gateway list: {e}",
                gateway_identifier
            );
        } else {
            tracing::warn!(
                "Blacklisted gateway {} due to connection or registration failure",
                gateway_identifier
            );
            // Re-create gateway selection stream to reflect the addition to the blacklist.
            let latest_tunnel_settings = self.latest_tunnel_settings.lock().await.clone();
            let _ = self.set_tunnel_settings(latest_tunnel_settings)
                .await
                .inspect_err(|err| {
                    tracing::warn!(
                        "Could not re-create gateway selection stream after blacklisting a gateway: {err:?}"
                    )
                });
        }
    }
}
