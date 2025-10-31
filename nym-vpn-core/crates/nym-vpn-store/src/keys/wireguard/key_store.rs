// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use super::keys::WireguardKeys;

#[async_trait::async_trait]
pub trait WireguardKeyStore {
    type StorageError: Error + Send + Sync + 'static;

    async fn load_or_create_keys(
        &self,
        gateway_id: &str,
    ) -> Result<WireguardKeys, Self::StorageError>;

    async fn clear_keys(&self) -> Result<(), Self::StorageError>;
}
