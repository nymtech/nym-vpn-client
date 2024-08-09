use std::path::{Path, PathBuf};

use nym_vpn_store::keys::{persistence::OnDiskKeysError, DeviceKeys, KeyStore as _};

use super::VpnClientOnDiskStorage;

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
}

// Set of helpers to load, create and store device keys

// #[allow(unused)]
// pub async fn load_device_keys<P: AsRef<Path> + Clone>(
//     path: P,
// ) -> Result<DeviceKeys, KeyStoreError> {
//     VpnClientOnDiskStorage::new(path.clone())
//         .load_keys()
//         .await
//         .map_err(|error| KeyStoreError::Load {
//             path: path.as_ref().to_path_buf(),
//             error,
//         })
// }
//
// #[allow(unused)]
// pub async fn create_device_keys<P: AsRef<Path> + Clone>(path: P) -> Result<(), KeyStoreError> {
//     let vpn_storage = VpnClientOnDiskStorage::new(path.clone());
//     let mut rng = rand::rngs::OsRng;
//     DeviceKeys::generate_new(&mut rng)
//         .persist_keys(&vpn_storage)
//         .await
//         .map_err(|error| KeyStoreError::Create {
//             path: path.as_ref().to_path_buf(),
//             error,
//         })
// }
//
// #[allow(unused)]
// pub async fn store_device_keys<P: AsRef<Path> + Clone>(
//     path: P,
//     keys: &DeviceKeys,
// ) -> Result<(), KeyStoreError> {
//     let vpn_storage = VpnClientOnDiskStorage::new(path.clone());
//     keys.persist_keys(&vpn_storage)
//         .await
//         .map_err(|error| KeyStoreError::Store {
//             path: path.as_ref().to_path_buf(),
//             error,
//         })
// }
