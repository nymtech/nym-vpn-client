// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_crypto::asymmetric::x25519;
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::types::RawWireguardKeys;

use super::key_store::WireguardKeyStore;

#[derive(Clone, Serialize, Deserialize)]
pub struct WireguardKeys {
    entry_keypair: Arc<x25519::KeyPair>,
    exit_keypair: Arc<x25519::KeyPair>,
}

impl TryFrom<RawWireguardKeys> for WireguardKeys {
    type Error = nym_crypto::asymmetric::x25519::KeyRecoveryError;

    fn try_from(keys: RawWireguardKeys) -> Result<Self, Self::Error> {
        let entry_keypair =
            Arc::new(x25519::PrivateKey::from_base58_string(keys.entry_private_key_bs58)?.into());
        let exit_keypair =
            Arc::new(x25519::PrivateKey::from_base58_string(keys.exit_private_key_bs58)?.into());

        Ok(WireguardKeys {
            entry_keypair,
            exit_keypair,
        })
    }
}

impl WireguardKeys {
    pub fn generate_new<R>(rng: &mut R) -> Self
    where
        R: RngCore + CryptoRng,
    {
        WireguardKeys {
            entry_keypair: Arc::new(x25519::KeyPair::new(rng)),
            exit_keypair: Arc::new(x25519::KeyPair::new(rng)),
        }
    }

    pub async fn load_or_create_keys<S: WireguardKeyStore>(
        store: &S,
        gateway_id: &str,
    ) -> Result<Self, S::StorageError> {
        store.load_or_create_keys(gateway_id).await
    }

    pub fn entry_keypair(&self) -> &Arc<x25519::KeyPair> {
        &self.entry_keypair
    }

    pub fn exit_keypair(&self) -> &Arc<x25519::KeyPair> {
        &self.exit_keypair
    }
}
