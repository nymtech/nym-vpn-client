// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashMap,
    net::IpAddr,
    time::{Duration, Instant},
};

use nym_offline_monitor::ConnectivityHandle;
use nym_sdk::mixnet::NodeIdentity;
use strum::IntoEnumIterator;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    Error, Gateway, GatewayClient, GatewayList, GatewayType, LookupGatewayFilters, NymNode,
    NymNodeList, error::Result,
};

/// The maximum age of the cache before it is considered stale.
const MAX_CACHE_AGE: Duration = Duration::from_secs(5 * 60);

#[derive(Clone)]
pub struct GatewayCacheHandle {
    tx: tokio::sync::mpsc::UnboundedSender<Command>,
}

impl GatewayCacheHandle {
    fn new(tx: tokio::sync::mpsc::UnboundedSender<Command>) -> Self {
        Self { tx }
    }

    /// Refresh all gateways and countries without blocking until the operation is complete.
    pub async fn refresh_all(&self) -> Result<()> {
        self.tx.send(Command::RefreshAll).map_err(|_| {
            tracing::error!("Gateway cache command channel closed (RefreshAll)");
            Error::Cancelled
        })
    }

    /// Lookup gateways waiting for any pending fetch request or initiating one if needed.
    pub async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Command::LookupGateways(gw_type, tx))
            .map_err(|_| {
                tracing::error!(
                    "Gateway cache command channel closed (LookupGateways: {:?})",
                    gw_type
                );
                Error::Cancelled
            })?;
        rx.await.map_err(|_| {
            tracing::error!(
                "Gateway cache response channel closed (LookupGateways: {:?})",
                gw_type
            );
            Error::Cancelled
        })?
    }

    pub async fn lookup_filtered_gateways(
        &self,
        filters: LookupGatewayFilters,
    ) -> Result<Vec<Gateway>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Command::LookupFilteredGateways(filters.clone(), tx))
            .map_err(|_| {
                tracing::error!(
                    "Gateway cache command channel closed (LookupFilteredGateways: {:?})",
                    filters.gw_type
                );
                Error::Cancelled
            })?;
        rx.await.map_err(|_| {
            tracing::error!(
                "Gateway cache response channel closed (LookupFilteredGateways: {:?})",
                filters.gw_type
            );
            Error::Cancelled
        })?
    }

    /// Lookup gateway IP address waiting for any pending fetch request or initiating one if needed.
    pub async fn lookup_gateway_ip(&self, gateway_identity: String) -> Result<IpAddr> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let identity_clone = gateway_identity.clone();
        self.tx
            .send(Command::LookupGatewayIp(gateway_identity, tx))
            .map_err(|_| {
                tracing::error!(
                    "Gateway cache command channel closed (LookupGatewayIp: {})",
                    identity_clone
                );
                Error::Cancelled
            })?;
        rx.await.map_err(|_| {
            tracing::error!(
                "Gateway cache response channel closed (LookupGatewayIp: {})",
                identity_clone
            );
            Error::Cancelled
        })?
    }

    pub fn replace_gateway_client(&self, gateway_client: GatewayClient) -> Result<()> {
        self.tx
            .send(Command::ReplaceGatewayClient(Box::new(gateway_client)))
            .map_err(|_| Error::Cancelled)
    }

    /// Clear all cached gateway data. This should be called when the network environment changes.
    pub fn clear_cache(&self) -> Result<()> {
        self.tx
            .send(Command::ClearCache)
            .map_err(|_| Error::Cancelled)
    }

    /// Pause or resume the background gateway cache refresh.
    ///
    /// While paused, connectivity-triggered fetches are held until [`set_paused(false)`] is called.
    /// On resume, if the initial refresh has not yet completed, it fires immediately (if online).
    pub fn set_paused(&self, paused: bool) -> Result<()> {
        self.tx
            .send(Command::Pause(paused))
            .map_err(|_| Error::Cancelled)
    }

    /// Lookup a NymNode by identity, using cached data if available.
    /// This is specifically for SOCKS5 which needs the nr_address field.
    pub async fn lookup_nymnode_by_identity(&self, identity: NodeIdentity) -> Result<NymNode> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let identity_str = identity.to_string();
        self.tx
            .send(Command::LookupNymNodeByIdentity(identity, tx))
            .map_err(|_| {
                tracing::error!(
                    "Gateway cache command channel closed (LookupNymNodeByIdentity: {})",
                    identity_str
                );
                Error::Cancelled
            })?;
        rx.await.map_err(|_| {
            tracing::error!(
                "Gateway cache response channel closed (LookupNymNodeByIdentity: {})",
                identity_str
            );
            Error::Cancelled
        })?
    }

    /// Lookup all NymNodes with network requester addresses.
    /// This is specifically for SOCKS5 Network Requester rotation.
    pub async fn lookup_all_nymnodes(&self) -> Result<NymNodeList> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Command::LookupAllNymNodes(tx)).map_err(|_| {
            tracing::error!("Gateway cache command channel closed (LookupAllNymNodes)");
            Error::Cancelled
        })?;
        rx.await.map_err(|_| {
            tracing::error!("Gateway cache response channel closed (LookupAllNymNodes)");
            Error::Cancelled
        })?
    }

    /// Lookup NymNodes with SOCKS5 probe data from VPN API
    /// This is specifically for SOCKS5 Network Requester selection and includes:
    /// - Probe data with SOCKS5 scores from VPN API
    /// - Network Requester addresses from node descriptions
    ///
    /// This method is separate from lookup_all_nymnodes() to avoid breaking existing code
    /// that depends on skimmed nodes data and append_performance()
    pub async fn lookup_nymnodes_for_socks5(&self) -> Result<NymNodeList> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Command::LookupNymNodesForSocks5(tx))
            .map_err(|_| {
                tracing::error!("Gateway cache command channel closed (LookupNymNodesForSocks5)");
                Error::Cancelled
            })?;
        rx.await.map_err(|_| {
            tracing::error!("Gateway cache response channel closed (LookupNymNodesForSocks5)");
            Error::Cancelled
        })?
    }
}

enum Command {
    RefreshAll,
    LookupGateways(
        GatewayType,
        tokio::sync::oneshot::Sender<Result<GatewayList>>,
    ),
    LookupFilteredGateways(
        LookupGatewayFilters,
        tokio::sync::oneshot::Sender<Result<Vec<Gateway>>>,
    ),
    LookupGatewayIp(
        String, // gateway_identity
        tokio::sync::oneshot::Sender<Result<IpAddr>>,
    ),
    LookupNymNodeByIdentity(NodeIdentity, tokio::sync::oneshot::Sender<Result<NymNode>>),
    LookupAllNymNodes(tokio::sync::oneshot::Sender<Result<NymNodeList>>),
    LookupNymNodesForSocks5(tokio::sync::oneshot::Sender<Result<NymNodeList>>),
    ReplaceGatewayClient(Box<GatewayClient>),
    ClearCache,
    Pause(bool),
}

pub struct GatewayCache {
    // The channel for receiving commands
    command_rx: tokio::sync::mpsc::UnboundedReceiver<Command>,

    // The underlying client that actually does the work
    gateway_client: GatewayClient,

    // The cached gateways and their last updated time
    cached_gateways: HashMap<GatewayType, (GatewayList, Instant)>,

    // The cached full node list (with nr_address) for SOCKS5
    cached_nymnodes: Option<(NymNodeList, Instant)>,

    // The connectivity handle to check if we are online
    connectivity_handle: ConnectivityHandle,

    /// Whether the initial refresh has been performed
    is_performed_initial_refresh: bool,

    /// When true, connectivity-triggered fetches are deferred until unpaused.
    paused: bool,

    // Shutdown token
    shutdown_token: CancellationToken,
}

impl GatewayCache {
    pub fn spawn(
        gateway_client: GatewayClient,
        connectivity_handle: ConnectivityHandle,
        shutdown_token: CancellationToken,
    ) -> (GatewayCacheHandle, JoinHandle<()>) {
        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();

        let inner = Self {
            gateway_client,
            connectivity_handle,
            command_rx,
            cached_gateways: HashMap::default(),
            cached_nymnodes: None,
            is_performed_initial_refresh: false,
            paused: false,
            shutdown_token,
        };
        let join_handle = tokio::spawn(inner.run());
        (GatewayCacheHandle::new(command_tx), join_handle)
    }

    async fn run(mut self) {
        if !self.paused && self.connectivity_handle.connectivity().await.is_online() {
            self.perform_initial_fetch_once().await;
        }

        loop {
            tokio::select! {
                Some(cmd) = self.command_rx.recv() => {
                    match cmd {
                        Command::RefreshAll => {
                            self.refresh_all().await;
                        }
                        Command::LookupGateways(gw_type, tx) => {
                            tx.send(self.lookup_gateways(gw_type).await).ok();
                        }
                        Command::LookupFilteredGateways(filters, tx) => {
                            let gw_vec = self.lookup_filtered_gateways(filters).await;
                            tx.send(gw_vec).ok();
                        }
                        Command::LookupGatewayIp(gateway_identity, tx) => {
                            tx.send(self.lookup_gateway_ip(&gateway_identity).await).ok();
                        }
                        Command::LookupNymNodeByIdentity(identity, tx) => {
                            tx.send(self.lookup_nymnode_by_identity(&identity).await).ok();
                        }
                        Command::LookupAllNymNodes(tx) => {
                            tx.send(self.refresh_nymnodes().await).ok();
                        }
                        Command::LookupNymNodesForSocks5(tx) => {
                            tx.send(self.refresh_nymnodes_for_socks5().await).ok();
                        }
                        Command::ReplaceGatewayClient(gateway_client) => {
                            self.replace_gateway_client(*gateway_client)
                        }
                        Command::ClearCache => {
                            self.clear_cache();
                        }
                        Command::Pause(paused) => {
                            tracing::info!("Gateway caching is {}", if paused { "Paused" } else { "Resumed" });
                            self.paused = paused;
                        }
                    }
                }
                Some(status) = self.connectivity_handle.next() => {
                    if status.is_online() && !self.paused {
                        self.perform_initial_fetch_once().await;
                    }
                }
                _ = self.shutdown_token.cancelled() => {
                    break;
                }
            }
        }
    }

    async fn perform_initial_fetch_once(&mut self) {
        if !self.is_performed_initial_refresh {
            self.is_performed_initial_refresh = self.refresh_all().await;
            if self.is_performed_initial_refresh {
                tracing::debug!("Initial gateway refresh completed successfully");
            } else {
                tracing::warn!("Initial gateway refresh failed");
            }
        }
    }

    fn replace_gateway_client(&mut self, gateway_client: GatewayClient) {
        let old_config = self.gateway_client.get_config();
        let new_config = gateway_client.get_config();

        self.gateway_client = gateway_client;

        // Invalidate cache immediately if gateway performance change
        if new_config.min_gateway_performance() != old_config.min_gateway_performance() {
            self.cached_gateways.clear();
            self.cached_nymnodes = None;
        }
    }

    fn clear_cache(&mut self) {
        tracing::debug!("Clearing gateway cache due to environment change");
        self.cached_gateways.clear();
        self.cached_nymnodes = None;
        // Reset the initial refresh flag so we fetch fresh data
        self.is_performed_initial_refresh = false;
    }

    async fn refresh_all(&mut self) -> bool {
        let gw_types = self.get_stale_gateway_list_types();

        if !gw_types.is_empty() {
            tracing::debug!("Refreshing gateways: {:?}", gw_types,);
            self.refresh(gw_types).await
        } else {
            false
        }
    }

    fn get_stale_gateway_list_types(&self) -> Vec<GatewayType> {
        GatewayType::iter()
            .filter(|gw_type| !self.is_gateways_current(gw_type))
            .collect()
    }

    async fn refresh(&mut self, gw_list_types: Vec<GatewayType>) -> bool {
        if self.connectivity_handle.connectivity().await.is_offline() {
            tracing::debug!("Not refreshing gateways because we are not connected");
            return false;
        }

        tracing::debug!("Refreshing gateway lists: {gw_list_types:?}");

        let mut tasks = tokio::task::JoinSet::new();

        for gw_type in gw_list_types {
            let client = self.gateway_client.clone();
            tasks.spawn(async move {
                let res = client.lookup_gateways(gw_type).await;
                (gw_type, res)
            });
        }

        let mut ok = false;

        while let Some(res) = tasks.join_next().await {
            match res {
                Ok((gw_type, r)) => match r {
                    Ok(refreshed_gateways) => {
                        tracing::debug!("Refreshed gateways for {gw_type:?}");
                        self.cached_gateways
                            .insert(gw_type, (refreshed_gateways, Instant::now()));
                        ok = true;
                    }
                    Err(err) => {
                        tracing::debug!("Failed to refresh gateways for {gw_type:?}: {err}");
                    }
                },
                Err(err) => {
                    tracing::error!("Failed to join on refresh task: {err}");
                }
            }
        }

        ok
    }

    fn is_gateways_current(&self, gw_type: &GatewayType) -> bool {
        self.cached_gateways
            .get(gw_type)
            .as_ref()
            .map(|(_, last_updated)| last_updated.elapsed() < MAX_CACHE_AGE)
            .unwrap_or_default()
    }

    async fn refresh_gateways(&mut self, gw_type: GatewayType) -> Result<GatewayList> {
        if let Some((gw_list, last_updated)) = self.cached_gateways.get(&gw_type)
            && last_updated.elapsed() < MAX_CACHE_AGE
        {
            Ok(gw_list.clone())
        } else {
            if self.connectivity_handle.connectivity().await.is_offline() {
                tracing::warn!("Not refreshing countries because we are not connected");
                return Err(Error::Offline);
            }

            let refreshed_gateways = self.gateway_client.lookup_gateways(gw_type).await?;

            self.cached_gateways
                .insert(gw_type, (refreshed_gateways.clone(), Instant::now()));

            Ok(refreshed_gateways)
        }
    }

    async fn lookup_gateways(&mut self, gw_type: GatewayType) -> Result<GatewayList> {
        let refresh_result = self.refresh_gateways(gw_type).await;

        // Regardless of if we managed to refresh the cache, we return the cached gateways if they
        // exist. They should be the most recent one we can muster
        if let Some((gateways, _)) = self.cached_gateways.get(&gw_type) {
            tracing::debug!(
                "Gateway cache returning {} cached gateways for {:?}",
                gateways.len(),
                gw_type
            );
            Ok(gateways.clone())
        } else {
            tracing::debug!(
                "No cached gateways for {:?}, returning refresh result",
                gw_type
            );
            refresh_result
        }
    }

    async fn lookup_filtered_gateways(
        &mut self,
        filters: LookupGatewayFilters,
    ) -> Result<Vec<Gateway>> {
        let gw_list = self.lookup_gateways(filters.gw_type).await?;
        Ok(gw_list.filter(&filters.filters))
    }

    async fn lookup_gateway_ip(&mut self, gateway_identity: &str) -> Result<IpAddr> {
        // If we have a populated list of gateways, we should always be able to find the IP there.
        if let Ok(identity) = NodeIdentity::from_base58_string(gateway_identity) {
            for (gateways, _) in self.cached_gateways.values() {
                if let Some(ip) = gateways
                    .node_with_identity(&identity)
                    .and_then(Gateway::lookup_ip)
                {
                    return Ok(ip);
                }
            }
        } else {
            tracing::warn!("Failed to parse gateway identity: {gateway_identity}");
        }

        // Fallback
        tracing::warn!("Using fallback to lookup gateway IP");
        self.gateway_client
            .lookup_gateway_ip(gateway_identity)
            .await
    }

    async fn refresh_nymnodes(&mut self) -> Result<NymNodeList> {
        if let Some((node_list, last_updated)) = &self.cached_nymnodes
            && last_updated.elapsed() < MAX_CACHE_AGE
        {
            tracing::debug!(
                "Using cached NymNode list (age: {:?}, {} nodes)",
                last_updated.elapsed(),
                node_list.len()
            );
            return Ok(node_list.clone());
        }

        if self.connectivity_handle.connectivity().await.is_offline() {
            tracing::warn!("Not refreshing NymNodes because we are not connected");
            // Return cached nodes if available, even if stale
            if let Some((node_list, _)) = &self.cached_nymnodes {
                tracing::info!("Returning stale cached nodes due to offline status");
                return Ok(node_list.clone());
            }
            return Err(Error::Offline);
        }

        tracing::debug!("Fetching fresh NymNode list from nym-api...");
        let start = std::time::Instant::now();
        let refreshed_nodes = self.gateway_client.lookup_all_nymnodes().await?;
        let fetch_duration = start.elapsed();

        let node_count = refreshed_nodes.len();
        tracing::info!(
            "Fetched {} NymNodes in {:?} (avg: {:?}/node)",
            node_count,
            fetch_duration,
            fetch_duration
                .checked_div(node_count as u32)
                .unwrap_or_default()
        );

        if node_count > 300 {
            tracing::warn!(
                "NymNode directory is big: ({} nodes) - consider filtering.",
                node_count
            );
        }

        self.cached_nymnodes = Some((refreshed_nodes.clone(), Instant::now()));
        Ok(refreshed_nodes)
    }

    async fn refresh_nymnodes_for_socks5(&mut self) -> Result<NymNodeList> {
        // This method uses VPN API directly (not cached) to get fresh SOCKS5 probe data
        // It's separate from refresh_nymnodes() to avoid breaking existing code
        let start = Instant::now();
        let refreshed_nodes = self.gateway_client.lookup_nymnodes_for_socks5().await?;
        let fetch_duration = start.elapsed();

        let node_count = refreshed_nodes.len();
        tracing::info!(
            "Fetched {} NymNodes for SOCKS5 in {:?} (avg: {:?}/node)",
            node_count,
            fetch_duration,
            fetch_duration
                .checked_div(node_count as u32)
                .unwrap_or_default()
        );

        Ok(refreshed_nodes)
    }

    async fn lookup_nymnode_by_identity(&mut self, identity: &NodeIdentity) -> Result<NymNode> {
        let refresh_result = self.refresh_nymnodes().await;

        // Try to find the node in cache first, regardless of refresh result
        if let Some((node_list, _)) = &self.cached_nymnodes
            && let Some(node) = node_list.node_with_identity(identity)
        {
            tracing::debug!(
                "Found NymNode {} in cache (has nr_address: {})",
                identity,
                node.nr_address.is_some()
            );
            return Ok(node.clone());
        }

        // If not in cache and refresh failed, return the error
        refresh_result?;

        // If refresh succeeded but node not found, return error
        Err(Error::RequestedGatewayIdNotFound(
            identity.to_base58_string(),
        ))
    }
}
