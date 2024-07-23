// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_crypto::asymmetric::ed25519;
use nym_pemstore::traits::{PemStorableKey, PemStorableKeyPair};
use rand::{CryptoRng, RngCore};
use zeroize::ZeroizeOnDrop;

use super::key_store::KeyStore;

pub trait DeviceKeyPair: PemStorableKeyPair {
    type PrivatePemKey: PemStorableKey;
    type PublicPemKey: PemStorableKey;
}

impl DeviceKeyPair for ed25519::KeyPair {
    type PrivatePemKey = ed25519::PrivateKey;
    type PublicPemKey = ed25519::PublicKey;
}

#[derive(Clone)]
pub struct DeviceKeys<T: DeviceKeyPair> {
    device_keypair: Arc<T>,
}

impl<T: DeviceKeyPair> DeviceKeys<T> {
    pub fn from_keys(device_keypair: T) -> Self {
        DeviceKeys {
            device_keypair: Arc::new(device_keypair),
        }
    }

    pub async fn load_keys<S: KeyStore<T>>(store: &S) -> Result<Self, S::StorageError> {
        store.load_keys().await
    }

    pub async fn persist_keys<S: KeyStore<T>>(&self, store: &S) -> Result<(), S::StorageError> {
        store.store_keys(self).await
    }

    pub fn device_keypair(&self) -> Arc<T> {
        Arc::clone(&self.device_keypair)
    }
}

impl DeviceKeys<ed25519::KeyPair> {
    pub fn generate_new_ed25519<R>(rng: &mut R) -> Self
    where
        R: RngCore + CryptoRng,
    {
        DeviceKeys {
            device_keypair: Arc::new(ed25519::KeyPair::new(rng)),
        }
    }
}

fn _assert_keys_zeroize_on_drop() {
    fn _assert_zeroize_on_drop<T: ZeroizeOnDrop>() {}

    _assert_zeroize_on_drop::<ed25519::KeyPair>();
}
