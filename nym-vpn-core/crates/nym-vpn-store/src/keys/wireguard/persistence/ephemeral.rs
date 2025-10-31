// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, sync::Arc};

use time::OffsetDateTime;
use tokio::sync::Mutex;

use crate::keys::wireguard::{
    WireguardKeyStore, WireguardKeys,
    persistence::{is_expired, random_keys},
};

struct KeysWithExpiration {
    keys: WireguardKeys,
    expiration_time: OffsetDateTime,
}

impl KeysWithExpiration {
    fn new(keys: WireguardKeys) -> Self {
        KeysWithExpiration {
            keys,
            expiration_time: OffsetDateTime::now_utc(),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct InMemEphemeralKeys {
    keys: Arc<Mutex<HashMap<String, KeysWithExpiration>>>,
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
        if let Some(keys_with_expiration) = guard.get(gateway_id)
            && !is_expired(keys_with_expiration.expiration_time)
        {
            Ok(keys_with_expiration.keys.clone())
        } else {
            let keys = random_keys();
            guard.insert(
                gateway_id.to_string(),
                KeysWithExpiration::new(keys.clone()),
            );

            Ok(keys)
        }
    }

    async fn clear_keys(&self) -> Result<(), Self::StorageError> {
        self.keys.lock().await.clear();
        Ok(())
    }
}
