// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use tokio::sync::Mutex;

use crate::keys::wireguard::{WireguardKeyStore, WireguardKeys, persistence::random_keys};

#[allow(dead_code)]
#[derive(Default)]
pub struct InMemEphemeralKeys {
    keys: Mutex<HashMap<String, WireguardKeys>>,
}

#[derive(Debug, thiserror::Error)]
pub enum EphemeralKeysError {}

#[async_trait::async_trait]
impl WireguardKeyStore for InMemEphemeralKeys {
    type StorageError = EphemeralKeysError;

    async fn load_or_create_keys(
        &self,
        gateway_id: &str,
    ) -> Result<WireguardKeys, EphemeralKeysError> {
        let mut guard = self.keys.lock().await;
        if let Some(keys) = guard.get(gateway_id) {
            Ok(keys.clone())
        } else {
            let keys = random_keys();
            guard.insert(gateway_id.to_string(), keys.clone());

            Ok(keys)
        }
    }
}
