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

pub async fn load_device_keys<P: AsRef<Path> + Clone>(
    path: P,
) -> Result<DeviceKeys, KeyStoreError> {
    let vpn_storage = VpnClientOnDiskStorage::new(path.clone());

    vpn_storage
        .load_keys()
        .await
        .map_err(|error| KeyStoreError::Load {
            path: path.as_ref().to_path_buf(),
            error,
        })
}
