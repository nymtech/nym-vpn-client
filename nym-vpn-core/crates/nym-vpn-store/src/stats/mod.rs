// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

pub mod on_disk;

pub trait StatsStorageError: Error + Send + Sync + 'static {}

#[async_trait::async_trait]
pub trait StatsStorage {
    type StorageError: StatsStorageError;

    async fn maybe_init_and_load_stats_seed(&self) -> Result<String, Self::StorageError>;
    async fn reset_stats_seed(&self) -> Result<String, Self::StorageError>;
    async fn remove_stats_seed(&self) -> Result<(), Self::StorageError>;
}
