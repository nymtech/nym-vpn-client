// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_store::VpnStorage;

#[derive(Debug, Clone)]
pub(crate) struct StatsStorage<S>
where
    S: VpnStorage,
{
    _storage: Arc<tokio::sync::Mutex<S>>,
}

impl<S> StatsStorage<S>
where
    S: VpnStorage,
{
    pub(crate) fn from(storage: Arc<tokio::sync::Mutex<S>>) -> Self {
        Self { _storage: storage }
    }
}
