// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, sync::Arc};

use nym_common::{ErrorExt, trace_err_chain};
use nym_platform_metadata::new_user_agent;
use nym_sdk::mixnet::StoragePaths;
use nym_vpn_account_controller::{CreateDeeplinkParams, Deeplink};
use nym_vpn_api_client::{
    VpnApiClient,
    response::NymVpnRegisterAccountResponse,
    types::{Device, DeviceStatus, Platform, VpnAccount, VpnAccountMode},
};
use nym_vpn_lib::storage::VpnClientOnDiskStorage;
use nym_vpn_lib_types::{
    AutologinResponse, DeeplinkClient, DeeplinkKind, GetDeeplinkParams, ParsedAccountLinks,
    RegisterAccountResponse, StorableAccount, StoreAccountRequest, StoredAccountMode,
    VpnAccountSummary,
};
use nym_vpn_store::{
    device_state::{DeviceStateStorage, StoredAccountIdentity, StoredDeviceState},
    keys::{device::DeviceKeyStore, wireguard::DB_NAME},
    mnemonic::{MnemonicStorage, StoredMnemonic},
};

use crate::{NymEnvironment, VpnError, deeplink::NymDeeplinkMnemonic};

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
    pub async fn new(
        data_dir: PathBuf,
        environment: Arc<NymEnvironment>,
    ) -> Result<Self, VpnError> {
        // todo: ensure account controller is not running?
        let storage_path = data_dir.join(environment.network_name());
        let secure_enclave = Arc::new(nym_secure_enclave::SecureEnclave::new());
        let storage = VpnClientOnDiskStorage::new(&storage_path, secure_enclave)
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?;
        Ok(Self {
            storage,
            storage_path,
            environment,
        })
    }

    /// Store the account mnemonic
    /// This is a version that can be called when the account controller is not running.
    pub async fn login(&self, request: StoreAccountRequest) -> Result<(), VpnError> {
        let storable_account = StorableAccount::try_from(request).map_err(VpnError::internal)?;
        let account = VpnAccount::try_from(storable_account.clone()).map_err(VpnError::internal)?;

        let vpn_api_client = self.create_vpn_api_client().await?;
        let _response = vpn_api_client
            .get_account(&account)
            .await
            .map_err(|_err| VpnError::AccountNotRegistered)?;

        self.init_account(storable_account).await
    }

    /// Either store account mnemonic or link the existing API account with Privy depending on the type of deeplink mnemonic.
    /// This is a version that can be called when the account controller is not running.
    pub async fn login_with_deeplink_mnemonic(
        &self,
        deeplink_mnemonic: Arc<NymDeeplinkMnemonic>,
    ) -> Result<(), VpnError> {
        let deeplink_mnemonic = deeplink_mnemonic.inner();

        let account = StorableAccount {
            mnemonic: deeplink_mnemonic.mnemonic.clone(),
            mode: StoredAccountMode::Privy,
        };

        match deeplink_mnemonic.kind {
            DeeplinkKind::Privy | DeeplinkKind::CreateAccount => {
                tracing::info!("Storing Privy account");

                self.init_account(account).await
            }
            DeeplinkKind::PrivyLink => {
                let privy_vpn_account =
                    VpnAccount::try_from(account).map_err(|err| VpnError::InvalidMnemonic {
                        details: err.to_string(),
                    })?;

                let vpn_api_client = self.create_vpn_api_client().await?;

                let current_account = self
                    .load_account()
                    .await?
                    .map(VpnAccount::try_from)
                    .transpose()
                    .map_err(|err| VpnError::InternalError {
                        details: err.to_string(),
                    })?;

                // We can only link the Privy account if we're currently logged-in with an API account
                if privy_vpn_account.mode().is_privy()
                    && let Some(ref current_account) = current_account
                    && current_account.mode().is_api()
                {
                    tracing::info!("Linking Privy account with API account");

                    let _status_ok = vpn_api_client
                        .link_account(current_account, &privy_vpn_account, "Social login")
                        .await
                        .inspect_err(|err| {
                            tracing::error!(
                                "Failed to link Privy account with API account: {err:?}"
                            )
                        })
                        .map_err(|err| VpnError::LinkPrivyAccount {
                            details: err.to_string(),
                        })?;

                    tracing::info!("Successfully linked Privy account with API account");

                    Ok(())
                } else {
                    tracing::error!(
                        "Cannot link Privy account when not logged-in with an API account"
                    );
                    Err(VpnError::internal(
                        "cannot link privy account when logged-in with an API account",
                    ))
                }
            }
            DeeplinkKind::AutologinRenew | DeeplinkKind::AutologinView => {
                Err(VpnError::DeeplinkError {
                    details: "Invalid deeplink kind".to_owned(),
                })
            }
        }
    }

    // Get deeplink for autologin
    pub async fn get_autologin_deeplink(
        &self,
        params: GetDeeplinkParams,
    ) -> Result<AutologinResponse, VpnError> {
        let Some(ref account_management) =
            self.environment.inner().nym_vpn_network.account_management
        else {
            return Err(VpnError::DeeplinkError {
                details: "No account management data is available at this time".to_owned(),
            });
        };

        let base_url = match params.client {
            DeeplinkClient::Mobile => account_management.autologin_mobile_url(&params.locale),
            DeeplinkClient::Desktop => account_management.autologin_desktop_url(&params.locale),
            DeeplinkClient::Web => account_management.autologin_web_url(&params.locale),
        }
        .ok_or(VpnError::DeeplinkError {
            details: "The autologin path could not be determined".to_owned(),
        })?;

        let mnemonic = self.get_stored_mnemonic().await?;

        let deeplink_params = CreateDeeplinkParams {
            kind: params.kind,
            name: params.name,
            base_url: base_url.clone(),
        };
        let deeplink = Deeplink::new(&deeplink_params);
        deeplink
            .create_autologin_url(&base_url, mnemonic)
            .map_err(|err| VpnError::DeeplinkError {
                details: err.to_string(),
            })
    }

    /// Generate the account mnemonic locally and store it.
    /// This is a version that can be called when the account controller is not running.
    pub async fn create_account(&self) -> Result<(), VpnError> {
        let (_, mnemonic) = VpnAccount::generate_new().map_err(VpnError::internal)?;
        let account = StorableAccount::new(mnemonic, StoredAccountMode::Api);
        self.init_account(account).await
    }

    /// Check if the account mnemonic is stored
    /// This is a version that can be called when the account controller is not running.
    pub async fn is_account_mnemonic_stored(&self) -> Result<bool, VpnError> {
        Ok(self.storage.is_mnemonic_stored().await?)
    }

    /// Returns account links for the logged in account or error if not logged in
    pub async fn account_links(&self, locale: String) -> Result<ParsedAccountLinks, VpnError> {
        let account_id = self.get_account_identity().await?;
        self.environment.account_links(&locale, Some(account_id))
    }

    /// Read and return the mnemonic, if there's one stored.
    pub async fn get_stored_mnemonic(&self) -> Result<String, VpnError> {
        Ok(self
            .storage
            .load_mnemonic()
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
        self.remove_device_state().await?;
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
            Ok(_) => tracing::info!("Removed file: {}", db_path.display()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!("File not found: {}", db_path.display())
            }
            Err(e) => {
                trace_err_chain!(e, "Failed to remove file: {}", db_path.display());
                return Err(VpnError::InternalError {
                    details: e.to_string(),
                });
            }
        }

        let mut first_err = None;

        for suffix in ["-wal", "-shm"] {
            let db_path = self.storage_path.join(format!("{DB_NAME}{suffix}"));
            match tokio::fs::remove_file(&db_path).await {
                Ok(_) => tracing::info!("Removed file: {}", db_path.display()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    tracing::debug!("File not found: {}", db_path.display())
                }
                Err(e) => {
                    trace_err_chain!(e, "Failed to remove file: {}", db_path.display());
                    first_err.get_or_insert(e);
                }
            }
        }

        match first_err {
            Some(e) => Err(VpnError::InternalError {
                details: e.to_string(),
            }),
            None => Ok(()),
        }
    }

    /// Get the account identity
    /// This is a version that can be called when the account controller is not running.
    pub async fn get_account_identity(&self) -> Result<String, VpnError> {
        Ok(self.load_account_identity().await?.id)
    }

    pub async fn get_canonical_account_identity(&self) -> Result<String, VpnError> {
        let account_identity = self.load_account_identity().await?;

        match account_identity.mode {
            StoredAccountMode::Api | StoredAccountMode::Decentralised => Ok(account_identity.id),
            StoredAccountMode::Privy => {
                let vpn_account = self.load_vpn_account().await?;
                let vpn_api_client = self.create_vpn_api_client().await?;
                let response = vpn_api_client
                    .get_canonical_account_identity(&vpn_account)
                    .await
                    .map_err(VpnError::internal)?;
                Ok(response.canonical_account_addr)
            }
        }
    }

    /// Get a summary of account usage, syncing from the VPN API first. falling back to cache otherwise
    pub async fn get_account_summary(&self) -> Result<Option<VpnAccountSummary>, VpnError> {
        // Check if we have an account
        let Some(mut device_state) = self.storage.load_device_state().await? else {
            return Ok(None);
        };

        let device = self.load_device().await?;

        match self
            .get_account_summary_from_network(&device_state.account_identity, &device)
            .await
        {
            Ok(account_summary) => {
                // Best effort, let's not fail to return it even though we have it
                device_state.summary = Some(account_summary.clone());
                let _ = self.storage.store_device_state(device_state).await;
                Ok(Some(account_summary))
            }
            Err(e) => {
                tracing::warn!("Account summary network sync failed, trying cache: {e:?}");
                // If we have an account but no cached summary, its absence is indeed an error
                device_state.summary.map(Some).ok_or(e)
            }
        }
    }

    /// Get the type of account the user is logged in with
    pub async fn get_account_mode(&self) -> Result<StoredAccountMode, VpnError> {
        Ok(self.load_account_identity().await?.mode)
    }

    /// Load the account mnemonic stored locally and register it.
    /// This is a version that can be called when the account controller is not running.
    pub async fn register_account(&self) -> Result<RegisterAccountResponse, VpnError> {
        let platform = Platform::Apple;
        let account = self.load_vpn_account().await?;
        let account_token = self
            .register_account_by_account(&account, platform)
            .await?
            .account_token;
        Ok(RegisterAccountResponse { account_token })
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
}

impl NymVpnAccountStorage {
    /// Stores the mnemonic and the device state derived from it, creating device keys if missing.
    async fn init_account(&self, account: StorableAccount) -> Result<(), VpnError> {
        let vpn_account =
            VpnAccount::try_from(account.clone()).map_err(|err| VpnError::InvalidMnemonic {
                details: err.to_string(),
            })?;
        let ecash_keypair = vpn_account
            .create_ecash_keypair()
            .map_err(VpnError::internal)?;
        let account_identity = StoredAccountIdentity::new(account.mnemonic.clone(), account.mode)
            .map_err(VpnError::internal)?;

        self.storage.init_keys(None).await?;
        self.storage
            .store_mnemonic(StoredMnemonic {
                mnemonic: account.mnemonic,
            })
            .await?;
        self.storage
            .store_device_state(StoredDeviceState {
                account_identity,
                ecash_keypair,
                summary: None,
            })
            .await?;
        Ok(())
    }

    /// Loads the account identity, which does not require step-up authentication.
    async fn load_account_identity(&self) -> Result<StoredAccountIdentity, VpnError> {
        self.storage
            .load_device_state()
            .await?
            .map(|device_state| device_state.account_identity)
            .ok_or(VpnError::NoAccountStored)
    }

    /// Loads the stored account, which requires step-up authentication.
    async fn load_account(&self) -> Result<Option<StorableAccount>, VpnError> {
        let Some(device_state) = self.storage.load_device_state().await? else {
            return Ok(None);
        };
        Ok(self.storage.load_mnemonic().await?.map(|stored| {
            StorableAccount::new(stored.mnemonic, device_state.account_identity.mode)
        }))
    }

    /// Materializes the stored account as [`VpnAccount`], which requires step-up authentication.
    async fn load_vpn_account(&self) -> Result<VpnAccount, VpnError> {
        let account = self
            .load_account()
            .await?
            .ok_or(VpnError::NoAccountStored)?;
        VpnAccount::try_from(account).map_err(VpnError::internal)
    }

    async fn get_account_summary_from_network(
        &self,
        account_identity: &StoredAccountIdentity,
        device: &Device,
    ) -> Result<VpnAccountSummary, VpnError> {
        tracing::info!(
            "Fetching account summary from VPN API for account {}",
            account_identity.id
        );

        let vpn_api_client = self.create_vpn_api_client().await?;

        let api_summary = vpn_api_client
            .get_account_summary_with_device(&account_identity.id, device)
            .await
            .map_err(|err| VpnError::InternalError {
                details: format!("Failed to get account summary: {err}"),
            })?;

        let summary = VpnAccountSummary::from_parts(
            &api_summary,
            VpnAccountMode::from(account_identity.mode),
        )
        .map_err(|err| VpnError::InternalError {
            details: format!("Failed to parse account summary: {err}"),
        })?;

        Ok(summary)
    }

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
        let account_identity = self.load_account_identity().await?;
        let vpn_api_client = self.create_vpn_api_client().await?;
        vpn_api_client
            .update_device(&account_identity.id, &device, DeviceStatus::DeleteMe)
            .await
            .map(|_| ())
            .map_err(|err| VpnError::UnregisterDevice {
                details: err.to_string(),
            })
    }

    async fn remove_account_mnemonic(&self) -> Result<(), VpnError> {
        Ok(self.storage.remove_mnemonic().await?)
    }

    async fn remove_device_state(&self) -> Result<(), VpnError> {
        Ok(self.storage.remove_device_state().await?)
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
