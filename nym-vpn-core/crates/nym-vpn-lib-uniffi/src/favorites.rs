// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_vpn_lib::{RecentsError, RecentsManager};
use nym_vpn_lib_types::{RecentGateways, TunnelType};

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
