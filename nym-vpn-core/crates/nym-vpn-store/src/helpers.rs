use std::path::{Path, PathBuf};

use rand::SeedableRng;

use crate::{key_store::KeyStore, DeviceKeys, DeviceKeysPaths, OnDiskKeys, OnDiskKeysError};

#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("failed to load device keys")]
    Load {
        path: PathBuf,
        error: OnDiskKeysError,
    },

    #[error("failed to create device keys")]
    Create {
        path: PathBuf,
        error: OnDiskKeysError,
    },

    #[error("failed to store device keys")]
    Store {
        path: PathBuf,
        error: OnDiskKeysError,
    },

    #[error("invalid key pair, one is missing: {path}")]
    InvalidKeyPair { path: PathBuf },
}

pub fn keypair_exists<P: AsRef<Path>>(path: P) -> Result<bool, KeyStoreError> {
    let device_key_paths = DeviceKeysPaths::new(path);
    let private_key_exists = device_key_paths.private_device_key().exists();
    let public_key_exists = device_key_paths.public_device_key().exists();
    match (private_key_exists, public_key_exists) {
        (true, true) => Ok(true),
        (false, false) => Ok(false),
        (true, false) => Err(KeyStoreError::InvalidKeyPair {
            path: device_key_paths.public_device_key().to_path_buf(),
        }),
        (false, true) => Err(KeyStoreError::InvalidKeyPair {
            path: device_key_paths.private_device_key().to_path_buf(),
        }),
    }
}

pub async fn load_device_keys<P: AsRef<Path> + Clone>(
    path: P,
) -> Result<DeviceKeys, KeyStoreError> {
    let device_key_paths = DeviceKeysPaths::new(path.clone());
    let key_store = OnDiskKeys::new(device_key_paths);

    key_store
        .load_keys()
        .await
        .map_err(|error| KeyStoreError::Load {
            path: path.as_ref().to_path_buf(),
            error,
        })
}

pub async fn create_device_keys<P: AsRef<Path> + Clone>(path: P) -> Result<(), KeyStoreError> {
    let device_key_paths = DeviceKeysPaths::new(path.clone());
    let key_store = OnDiskKeys::new(device_key_paths);

    let mut rng = rand::rngs::OsRng;
    DeviceKeys::generate_new(&mut rng)
        .persist_keys(&key_store)
        .await
        .map_err(|error| KeyStoreError::Create {
            path: path.as_ref().to_path_buf(),
            error,
        })
}

pub async fn create_device_keys_from_seed<P: AsRef<Path> + Clone>(
    path: P,
    seed: [u8; 32],
) -> Result<(), KeyStoreError> {
    let device_key_paths = DeviceKeysPaths::new(path.clone());
    let key_store = OnDiskKeys::new(device_key_paths);

    let mut rng = rand_chacha::ChaCha20Rng::from_seed(seed);

    DeviceKeys::generate_new(&mut rng)
        .persist_keys(&key_store)
        .await
        .map_err(|error| KeyStoreError::Create {
            path: path.as_ref().to_path_buf(),
            error,
        })
}

pub async fn store_device_keys<P: AsRef<Path> + Clone>(
    path: P,
    keys: &DeviceKeys,
) -> Result<(), KeyStoreError> {
    let device_key_paths = DeviceKeysPaths::new(path.clone());
    let key_store = OnDiskKeys::new(device_key_paths);

    keys.persist_keys(&key_store)
        .await
        .map_err(|error| KeyStoreError::Store {
            path: path.as_ref().to_path_buf(),
            error,
        })
}
