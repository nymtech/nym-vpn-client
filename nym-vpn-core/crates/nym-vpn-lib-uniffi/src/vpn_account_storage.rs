// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, sync::Arc};

#[cfg(target_os = "ios")]
use nym_common::ErrorExt;
use nym_common::trace_err_chain;
use nym_platform_metadata::new_user_agent;
use nym_sdk::mixnet::StoragePaths;
#[cfg(target_os = "ios")]
use nym_vpn_api_client::{Platform, response::NymVpnRegisterAccountResponse};
use nym_vpn_api_client::{
    VpnApiClient,
    types::{Device, DeviceStatus, VpnAccount, VpnAccountMode},
};
use nym_vpn_lib::storage::VpnClientOnDiskStorage;
#[cfg(target_os = "ios")]
use nym_vpn_lib_types::RegisterAccountResponse;
use nym_vpn_lib_types::StoreAccountRequest;
use nym_vpn_store::{
    account::AccountInformationStorage,
    keys::{device::DeviceKeyStore, wireguard::DB_NAME},
    types::StorableAccount,
};

use crate::{NymEnvironment, VpnError};

/// Raw API that directly accesses storage without going through the account controller.
/// This API places the responsibility of ensuring the account controller is not running on
/// the caller.
///
/// This type is not thread-safe and should not be used concurrently.
///
/// WARN: This API was added mostly as a workaround for unblocking the iOS client, and is not a
/// sustainable long term solution.
#[derive(uniffi::Object)]
pub struct NymVpnAccountStorage {
    storage_path: PathBuf,
    storage: VpnClientOnDiskStorage,
    environment: Arc<NymEnvironment>,
}

#[uniffi::export(async_runtime = "tokio")]
impl NymVpnAccountStorage {
    #[uniffi::constructor]
    pub fn new(storage_path: PathBuf, environment: Arc<NymEnvironment>) -> Self {
        // todo: ensure account controller is not running?
        Self {
            storage: VpnClientOnDiskStorage::new(&storage_path),
            storage_path,
            environment,
        }
    }

    /// Store the account mnemonic
    /// This is a version that can be called when the account controller is not running.
    pub async fn login(&self, request: StoreAccountRequest) -> Result<(), VpnError> {
        let mnemonic = nym_vpn_lib::login::parse_account_request(&request).map_err(|err| {
            VpnError::InvalidSecret {
                details: err.to_string(),
            }
        })?;
        let vpn_api_client = self.create_vpn_api_client().await?;
        let account =
            VpnAccount::new(mnemonic.clone(), VpnAccountMode::Api).map_err(VpnError::internal)?;
        let _response = vpn_api_client
            .get_account(&account)
            .await
            .map_err(|_err| VpnError::AccountNotRegistered)?;

        self.storage
            .store_account(StorableAccount::from(mnemonic))
            .await?;
        self.storage.init_keys(None).await?;
        Ok(())
    }

    /// Generate the account mnemonic locally and store it.
    /// This is a version that can be called when the account controller is not running.
    pub async fn create_account(&self) -> Result<(), VpnError> {
        let (_, mnemonic) = VpnAccount::generate_new().map_err(VpnError::internal)?;
        self.storage.store_account(mnemonic.into()).await?;
        self.storage.init_keys(None).await?;
        Ok(())
    }

    /// Check if the account mnemonic is stored
    /// This is a version that can be called when the account controller is not running.
    pub async fn is_account_mnemonic_stored(&self) -> Result<bool, VpnError> {
        Ok(self.storage.is_account_stored().await?)
    }

    /// Read and return the mnemonic, if there's one stored.
    pub async fn get_stored_mnemonic(&self) -> Result<String, VpnError> {
        Ok(self
            .storage
            .load_account()
            .await?
            .ok_or(VpnError::NoAccountStored)?
            .mnemonic
            .to_string())
    }

    /// Remove the account mnemonic and all associated keys and files.
    /// This is a version that can be called when the account controller is not running.
    pub async fn forget_account(&self) -> Result<(), VpnError> {
        tracing::info!(
            "REMOVING ALL ACCOUNT AND DEVICE DATA IN: {}",
            self.storage_path.display()
        );

        self.unregister_device()
            .await
            .inspect(|_| tracing::info!("Device has been unregistered"))
            .inspect_err(|err| tracing::error!("Failed to unregister device: {err:?}"))
            .ok();

        // First remove the files we own directly
        self.remove_account_mnemonic().await?;
        self.remove_device_identity().await?;
        self.remove_credential_storage().await?;

        // Then remove the rest of the files, that we own indirectly
        nym_vpn_account_controller::remove_files_for_account(&self.storage_path, true)
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?;

        Ok(())
    }

    /// Force a rotation of the wireguard keys
    /// This is a version that can be called when the account controller is not running.
    pub async fn rotate_keys(&self) -> Result<(), VpnError> {
        let db_path = self.storage_path.join(DB_NAME);
        match tokio::fs::remove_file(&db_path).await {
            Ok(_) => tracing::trace!("Removed file: {}", db_path.display()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::trace!("File not found: {}", db_path.display())
            }
            Err(e) => {
                trace_err_chain!(e, "Failed to remove file: {}", db_path.display());

                return Err(VpnError::InternalError {
                    details: e.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Get the device identity
    /// This is a version that can be called when the account controller is not running.
    pub async fn get_device_identity(&self) -> Result<String, VpnError> {
        let device_id = self
            .storage
            .load_keys()
            .await
            .map_err(|_err| VpnError::NoDeviceIdentity)?
            .ok_or(VpnError::NoDeviceIdentity)?;
        Ok(device_id.device_keypair().public_key().to_string())
    }

    /// Get the account identity
    /// This is a version that can be called when the account controller is not running.
    pub async fn get_account_identity(&self) -> Result<String, VpnError> {
        let account = self
            .storage
            .load_account()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoAccountStored)?;
        VpnAccount::try_from(account)
            .map_err(VpnError::internal)
            .map(|account| account.id().to_string())
    }
}

#[cfg(target_os = "ios")]
#[uniffi::export(async_runtime = "tokio")]
impl NymVpnAccountStorage {
    /// Load the account mnemonic stored locally and register it.
    /// This is a version that can be called when the account controller is not running.
    pub async fn register_account(&self) -> Result<RegisterAccountResponse, VpnError> {
        let platform = Platform::Apple;
        let account = self
            .load_account()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoAccountStored)?;
        let account = VpnAccount::try_from(account).map_err(VpnError::internal)?;
        let account_token = self
            .register_account_by_account(&account, platform)
            .await?
            .account_token;
        Ok(RegisterAccountResponse { account_token })
    }
}

impl NymVpnAccountStorage {
    #[cfg(target_os = "ios")]
    async fn register_account_by_account(
        &self,
        account: &VpnAccount,
        platform: Platform,
    ) -> Result<NymVpnRegisterAccountResponse, VpnError> {
        let vpn_api_client = self.create_vpn_api_client().await?;
        vpn_api_client
            .register_account(account, platform)
            .await
            .map_err(|err| VpnError::FailedAccountRegistration {
                details: err.display_chain(),
            })
    }

    async fn create_vpn_api_client(&self) -> Result<VpnApiClient, VpnError> {
        let user_agent = new_user_agent!();
        let vpn_api_client = VpnApiClient::from_network(
            self.environment.inner().nym_network_details(),
            Some(user_agent),
            None,
        )
        .await
        .map_err(VpnError::internal)?;
        Ok(vpn_api_client)
    }

    async fn load_device(&self) -> Result<Device, VpnError> {
        let device_id = self
            .storage
            .load_keys()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoDeviceIdentity)?;
        Ok(Device::from(device_id.device_keypair().clone()))
    }

    async fn unregister_device(&self) -> Result<(), VpnError> {
        let device = self.load_device().await?;
        let account = self
            .storage
            .load_account()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoAccountStored)?;
        let account = VpnAccount::try_from(account).map_err(VpnError::internal)?;
        let vpn_api_client = self.create_vpn_api_client().await?;
        vpn_api_client
            .update_device(&account, &device, DeviceStatus::DeleteMe)
            .await
            .map(|_| ())
            .map_err(|err| VpnError::UnregisterDevice {
                details: err.to_string(),
            })
    }

    async fn remove_account_mnemonic(&self) -> Result<bool, VpnError> {
        self.storage
            .remove_account()
            .await
            .map(|_| true)
            .map_err(Into::into)
    }

    async fn remove_credential_storage(&self) -> Result<(), VpnError> {
        let storage_paths =
            StoragePaths::new_from_dir(&self.storage_path).map_err(VpnError::internal)?;
        for path in storage_paths.credential_database_paths() {
            tracing::info!("Removing file: {}", path.display());
            match tokio::fs::remove_file(&path).await {
                Ok(_) => tracing::trace!("Removed file: {}", path.display()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    tracing::trace!("File not found, skipping: {}", path.display())
                }
                Err(e) => {
                    trace_err_chain!(e, "Failed to remove file: {}", path.display());

                    return Err(VpnError::InternalError {
                        details: e.to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    async fn remove_device_identity(&self) -> Result<(), VpnError> {
        self.storage.remove_keys().await.map_err(VpnError::internal)
    }
}
