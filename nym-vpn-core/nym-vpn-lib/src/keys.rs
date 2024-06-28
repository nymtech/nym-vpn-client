// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{error::Error, sync::Arc};

use nym_crypto::{asymmetric::identity, ctr::cipher::zeroize::ZeroizeOnDrop};
use rand::{CryptoRng, RngCore};

struct DeviceKeys {
    device_keypair: Arc<identity::KeyPair>,
}

impl DeviceKeys {
    pub fn generate_new_device_keys<R>(rng: &mut R) -> Self
    where
        R: RngCore + CryptoRng,
    {
        DeviceKeys {
            device_keypair: Arc::new(identity::KeyPair::new(rng)),
        }
    }

    pub fn from_keys(device_keypair: identity::KeyPair) -> Self {
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
}

fn _assert_keys_zeroize_on_drop() {
    fn _assert_zeroize_on_drop<T: ZeroizeOnDrop>() {}

    _assert_zeroize_on_drop::<identity::KeyPair>();
}

pub trait KeyStore {
    type StorageError: Error;

    async fn load_keys(&self) -> Result<DeviceKeys, Self::StorageError>;
    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), Self::StorageError>;
}

#[derive(Default)]
pub struct InMemEphemeralKeys {
    keys: Mutex<Option<DeviceKeys>>,
}


