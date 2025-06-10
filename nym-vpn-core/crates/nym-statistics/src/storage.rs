// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::Error;
use std::sync::Arc;

use nym_vpn_store::VpnStorage;

#[derive(Debug)]
pub(crate) struct StatsStorage<S>
where
    S: VpnStorage,
{
    storage: Arc<tokio::sync::Mutex<S>>,
}

impl<S> StatsStorage<S>
where
    S: VpnStorage,
{
    pub(crate) fn from(storage: Arc<tokio::sync::Mutex<S>>) -> Self {
        Self { storage }
    }

    pub(crate) async fn maybe_init_and_load_seed(&self) -> Result<String, Error> {
        self.storage
            .lock()
            .await
            .maybe_init_and_load_stats_seed()
            .await
            .map_err(|err| Error::StatsStorage {
                source: Box::new(err),
            })
    }
}

// S is not clone so we have to do that manually
impl<S> Clone for StatsStorage<S>
where
    S: VpnStorage,
{
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}
