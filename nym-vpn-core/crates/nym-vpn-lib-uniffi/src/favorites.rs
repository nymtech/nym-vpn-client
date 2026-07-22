// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, sync::Arc};

use nym_vpn_lib::{FavoritesManager, RecentsError, RecentsManager};
use nym_vpn_lib_types::{FavoriteSelector, FavoriteSelectors, RecentGateways, TunnelType};
use tokio::sync::RwLock;

use crate::gateway_cache::NymGatewayCache;

#[derive(Debug, thiserror::Error)]
enum FavoritesInnerError {
    #[error("failed to get recent gateways ({0})")]
    Recents(RecentsError),
}

impl FavoritesInnerError {
    pub fn error_chain(&self) -> String {
        match self {
            Self::Recents(err) => err.to_string(),
        }
    }
}

#[derive(Debug, uniffi::Object)]
#[uniffi::export(Display)]
pub struct FavoritesError {
    inner: FavoritesInnerError,
}

impl FavoritesError {
    fn new(inner: FavoritesInnerError) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl FavoritesError {
    /// Returns formatted error chain
    pub fn error_chain(&self) -> String {
        self.inner.error_chain()
    }
}

impl std::fmt::Display for FavoritesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner.to_string())
    }
}

impl From<FavoritesInnerError> for FavoritesError {
    fn from(value: FavoritesInnerError) -> Self {
        Self::new(value)
    }
}

type Result<T, E = FavoritesError> = std::result::Result<T, E>;

#[uniffi::export]
pub async fn get_recent_gateways_no_service(
    data_dir: PathBuf,
    gateway_cache: &NymGatewayCache,
    tunnel_type: TunnelType,
) -> Result<RecentGateways> {
    let recent_gateway_cache = RecentsManager::new(data_dir, gateway_cache).await;
    Ok(recent_gateway_cache
        .get_recent(tunnel_type)
        .await
        .map_err(FavoritesInnerError::Recents)?)
}

#[derive(uniffi::Object)]
pub struct FavoritesController {
    manager: Arc<RwLock<FavoritesManager>>,
}

#[uniffi::export(async_runtime = "tokio")]
impl FavoritesController {
    #[uniffi::constructor]
    pub async fn new(data_dir: PathBuf) -> Self {
        let manager = Arc::new(RwLock::new(FavoritesManager::new(data_dir).await));
        Self { manager }
    }

    pub async fn add_favorite_entry(&self, selector: FavoriteSelector) {
        self.manager
            .write()
            .await
            .add_favorite_entry(selector)
            .await;
    }

    pub async fn add_favorite_exit(&self, selector: FavoriteSelector) {
        self.manager
            .write()
            .await
            .add_favorite_entry(selector)
            .await;
    }

    pub async fn remove_favorite_entry(&self, selector: FavoriteSelector) {
        self.manager
            .write()
            .await
            .remove_favorite_entry(selector)
            .await;
    }

    pub async fn remove_favorite_exit(&self, selector: FavoriteSelector) {
        self.manager
            .write()
            .await
            .remove_favorite_exit(selector)
            .await;
    }

    pub async fn get_favorites(&self) -> FavoriteSelectors {
        self.manager.read().await.get_favorites()
    }
}
