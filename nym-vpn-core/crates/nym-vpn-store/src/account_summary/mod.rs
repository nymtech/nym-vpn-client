// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use nym_vpn_lib_types::VpnAccountSummary;

pub mod ephemeral;
pub mod on_disk;

/// Cached account summary, kept separately from the account information as it is refreshed
/// frequently and holds no secrets.
#[async_trait::async_trait]
pub trait AccountSummaryStorage {
    type StorageError: Error + Send + Sync + 'static;

    /// Returns `None` when no summary is stored.
    async fn load_summary(&self) -> Result<Option<VpnAccountSummary>, Self::StorageError>;

    async fn store_summary(&self, summary: &VpnAccountSummary) -> Result<(), Self::StorageError>;

    async fn remove_summary(&self) -> Result<(), Self::StorageError>;
}
