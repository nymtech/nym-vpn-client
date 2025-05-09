// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::{stream::FuturesUnordered, StreamExt};
use strum::IntoEnumIterator;
use tokio::sync::Mutex;

use crate::{error::Result, Country, GatewayClient, GatewayList, GatewayType};

#[derive(Clone)]
pub struct CachingGatewayClient {
    inner: Arc<Mutex<CachingGatewayClientInner>>,
}

impl CachingGatewayClient {
    pub fn new(gateway_client: GatewayClient) -> Self {
        CachingGatewayClient {
            inner: Arc::new(Mutex::new(CachingGatewayClientInner {
                gateway_client,
                cached_gateways: Default::default(),
                cached_countries: Default::default(),
            })),
        }
    }

    pub async fn refresh_all(&self) {
        self.inner.lock().await.refresh_all().await
    }

    pub async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList> {
        self.inner.lock().await.lookup_gateways(gw_type).await
    }

    pub async fn lookup_countries(&self, gw_type: GatewayType) -> Result<Vec<Country>> {
        self.inner.lock().await.lookup_countries(gw_type).await
    }

    pub async fn lookup_gateway_ip(&self, gateway_identity: &str) -> Result<IpAddr> {
        self.inner
            .lock()
            .await
            .lookup_gateway_ip(gateway_identity)
            .await
    }
}

/// A caching client that wraps around the `GatewayClient` and caches the results of
/// `lookup_gateways` and `lookup_countries` calls.
struct CachingGatewayClientInner {
    // The underlying client that actually does the work
    gateway_client: GatewayClient,

    // The cached gateways and their last updated time
    cached_gateways: HashMap<GatewayType, (GatewayList, Instant)>,

    // The cached countries and their last updated time
    cached_countries: HashMap<GatewayType, (Vec<Country>, Instant)>,
}

impl CachingGatewayClientInner {
    /// The maximum age of the cache before it is considered stale.
    const MAX_CACHE_AGE: Duration = Duration::from_secs(5 * 60);

    pub async fn refresh_all(&mut self) {
        let mut tasks = FuturesUnordered::new();

        for gw_type in GatewayType::iter() {
            let client = self.gateway_client.clone();
            tasks.push(async move {
                let res_countries = client.lookup_countries(gw_type.clone()).await;
                let res_gateways = client.lookup_gateways(gw_type.clone()).await;
                (gw_type, res_countries, res_gateways)
            });
        }

        while let Some((gw_type, res_countries, res_gateways)) = tasks.next().await {
            if let Ok(ref refreshed_gateways) = res_gateways {
                tracing::info!("refreshed gateways for {gw_type:?}");
                self.cached_gateways.insert(
                    gw_type.clone(),
                    (refreshed_gateways.clone(), Instant::now()),
                );
            }

            if let Ok(ref refreshed_countries) = res_countries {
                tracing::info!("refreshed countries for {gw_type:?}");
                self.cached_countries.insert(
                    gw_type.clone(),
                    (refreshed_countries.clone(), Instant::now()),
                );
            }
        }
    }

    pub async fn lookup_gateways(&mut self, gw_type: GatewayType) -> Result<GatewayList> {
        if let Some((gateways, last_updated)) = self.cached_gateways.get(&gw_type) {
            if last_updated.elapsed() < Self::MAX_CACHE_AGE {
                return Ok(gateways.clone());
            }
        }

        let refreshed_gateways = self.gateway_client.lookup_gateways(gw_type.clone()).await;

        if let Ok(ref refreshed_gateways) = refreshed_gateways {
            self.cached_gateways.insert(
                gw_type.clone(),
                (refreshed_gateways.clone(), Instant::now()),
            );
        }

        // Regardless of if we managed to refresh the cache, we return the cached gateways if they
        // exist
        if let Some((gateways, _)) = self.cached_gateways.get(&gw_type) {
            Ok(gateways.clone())
        } else {
            refreshed_gateways
        }
    }

    pub async fn lookup_countries(&mut self, gw_type: GatewayType) -> Result<Vec<Country>> {
        if let Some((countries, last_updated)) = self.cached_countries.get(&gw_type) {
            if last_updated.elapsed() < Self::MAX_CACHE_AGE {
                return Ok(countries.clone());
            }
        }

        let refreshed_countries = self.gateway_client.lookup_countries(gw_type.clone()).await;

        // TODO: don't attempt to refresh if we are offline
        if let Ok(ref refreshed_countries) = refreshed_countries {
            self.cached_countries.insert(
                gw_type.clone(),
                (refreshed_countries.clone(), Instant::now()),
            );
        }

        // Regardless of if we managed to refresh the cache, we return the cached countries if they
        // exist
        if let Some((countries, _)) = self.cached_countries.get(&gw_type) {
            Ok(countries.clone())
        } else {
            refreshed_countries
        }
    }

    pub async fn lookup_gateway_ip(&mut self, gateway_identity: &str) -> Result<IpAddr> {
        // TODO: cache
        self.gateway_client
            .lookup_gateway_ip(gateway_identity)
            .await
    }
}
