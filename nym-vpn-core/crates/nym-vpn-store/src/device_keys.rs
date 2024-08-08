// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_crypto::asymmetric::ed25519;
use rand::{CryptoRng, RngCore};
use zeroize::ZeroizeOnDrop;

use super::key_store::KeyStore;

#[derive(Clone)]
pub struct DeviceKeys {
    device_keypair: Arc<ed25519::KeyPair>,
}

impl DeviceKeys {
    pub fn generate_new<R>(rng: &mut R) -> Self
    where
        R: RngCore + CryptoRng,
    {
        DeviceKeys {
            device_keypair: Arc::new(ed25519::KeyPair::new(rng)),
        }
    }

    pub fn from_keys(device_keypair: ed25519::KeyPair) -> Self {
        DeviceKeys {
            device_keypair: Arc::new(device_keypair),
        }
    }

    pub async fn load_keys<S: KeyStore>(store: &S) -> Result<Self, S::StorageError> {
        store.load_keys().await
    }

    pub async fn persist_keys<S: KeyStore>(&self, store: &S) -> Result<(), S::StorageError> {
        store.store_keys(self).await
    }

    pub fn device_keypair(&self) -> Arc<ed25519::KeyPair> {
        Arc::clone(&self.device_keypair)
    }
}

fn _assert_keys_zeroize_on_drop() {
    fn _assert_zeroize_on_drop<T: ZeroizeOnDrop>() {}

    _assert_zeroize_on_drop::<ed25519::KeyPair>();
}
