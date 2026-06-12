// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use nym_vpn_lib_types::VpnAccountSummary;

pub mod ephemeral;
pub mod on_disk;

/// Storage for the account summary cache, kept alongside the mnemonic and device keys.
#[async_trait::async_trait]
pub trait AccountSummaryStorage {
    type StorageError: Error + Send + Sync + 'static;

    async fn load_summary(&self) -> Result<Option<VpnAccountSummary>, Self::StorageError>; // None means no error, but nothing
    async fn store_summary(&self, account: VpnAccountSummary) -> Result<(), Self::StorageError>;
    async fn remove_summary(&self) -> Result<(), Self::StorageError>;
}
