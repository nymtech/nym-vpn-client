// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::convert::Infallible;

use nym_vpn_lib_types::VpnAccountSummary;
use tokio::sync::Mutex;

use super::AccountSummaryStorage;

#[derive(Default)]
pub struct InMemoryAccountSummaryStorage {
    summary: Mutex<Option<VpnAccountSummary>>,
}

#[async_trait::async_trait]
impl AccountSummaryStorage for InMemoryAccountSummaryStorage {
    type StorageError = Infallible;

    async fn load_summary(&self) -> Result<Option<VpnAccountSummary>, Self::StorageError> {
        Ok(self.summary.lock().await.clone())
    }

    async fn store_summary(&self, account: VpnAccountSummary) -> Result<(), Self::StorageError> {
        *self.summary.lock().await = Some(account);
        Ok(())
    }

    async fn remove_summary(&self) -> Result<(), Self::StorageError> {
        *self.summary.lock().await = None;
        Ok(())
    }
}
