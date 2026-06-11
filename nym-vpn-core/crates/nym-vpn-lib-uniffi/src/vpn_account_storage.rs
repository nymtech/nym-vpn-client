// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, sync::Arc};

use nym_common::{ErrorExt, trace_err_chain};
use nym_platform_metadata::new_user_agent;
use nym_sdk::mixnet::StoragePaths;
use nym_vpn_account_controller::{CreateDeeplinkParams, Deeplink};
use nym_vpn_account_controller::{
    PrefetchZkNymOutcome, SUMMARY_STALE_AFTER, register_device_if_needed, verify_time_synced,
};
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
    account::AccountInformationStorage,
    account_summary::AccountSummaryStorage,
    keys::{device::DeviceKeyStore, wireguard::DB_NAME},
};

use crate::{NymEnvironment, VpnError, deeplink::NymDeeplinkMnemonic};

/// Outcome returned by a one-shot zk-nym prefetch.
///
/// Exposed via UniFFI so Swift callers can distinguish the three meaningful
/// cases without parsing strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum VpnPrefetchZkNymOutcome {
    /// Local storage already had enough tickets; no API call was made.
    SufficientBandwidth,
    /// New ticketbooks were successfully fetched and stored.
    FetchedTickets,
    /// Upgrade mode is active; tickets are not issued in this mode.
    UpgradeMode,
    /// Prefetch skipped because another process holds the credential store lock.
    SkippedStoreBusy,
}

impl From<PrefetchZkNymOutcome> for VpnPrefetchZkNymOutcome {
    fn from(value: PrefetchZkNymOutcome) -> Self {
        match value {
            PrefetchZkNymOutcome::SufficientBandwidth => Self::SufficientBandwidth,
            PrefetchZkNymOutcome::FetchedTickets => Self::FetchedTickets,
            PrefetchZkNymOutcome::UpgradeMode => Self::UpgradeMode,
            PrefetchZkNymOutcome::SkippedStoreBusy => Self::SkippedStoreBusy,
        }
    }
}

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
    pub fn new(data_dir: PathBuf, environment: Arc<NymEnvironment>) -> Self {
        // todo: ensure account controller is not running?
        let storage_path = data_dir.join(environment.network_name());
        Self {
            storage: VpnClientOnDiskStorage::new(&storage_path),
            storage_path,
            environment,
        }
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

        self.storage.store_account(storable_account).await?;
        self.storage.init_keys(None).await?;
        Ok(())
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

                self.storage.store_account(account).await?;
                self.storage.init_keys(None).await?;
                Ok(())
            }
            DeeplinkKind::PrivyLink => {
                let privy_vpn_account =
                    VpnAccount::try_from(account).map_err(|err| VpnError::InvalidMnemonic {
                        details: err.to_string(),
                    })?;

                let vpn_api_client = self.create_vpn_api_client().await?;

                let current_account = self
                    .storage
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
        self.storage.store_account(account).await?;
        self.storage.init_keys(None).await?;
        Ok(())
    }

    /// Check if the account mnemonic is stored
    /// This is a version that can be called when the account controller is not running.
    pub async fn is_account_mnemonic_stored(&self) -> Result<bool, VpnError> {
        Ok(self.storage.is_account_stored().await?)
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

    pub async fn get_canonical_account_identity(&self) -> Result<String, VpnError> {
        let account = self
            .storage
            .load_account()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoAccountStored)?;

        let vpn_account = VpnAccount::try_from(account).map_err(VpnError::internal)?;

        match vpn_account.mode() {
            VpnAccountMode::Api | VpnAccountMode::Decentralised => Ok(vpn_account.id().to_string()),
            VpnAccountMode::Privy => {
                let vpn_api_client = self.create_vpn_api_client().await?;
                let response = vpn_api_client
                    .get_canonical_account_identity(&vpn_account)
                    .await
                    .map_err(VpnError::internal)?;
                Ok(response.canonical_account_addr)
            }
        }
    }

    /// Get a summary of account usage, syncing from the VPN API first.
    ///
    /// On iOS the account controller is not running in-process, so a cache-only read
    /// would stay empty after login until the tunnel starts. This path fetches
    /// `/account/{id}/device/{device}/summary`, persists the result, and returns it.
    /// On transient network failure it falls back to the last cached summary.
    pub async fn get_account_summary(&self) -> Result<Option<VpnAccountSummary>, VpnError> {
        match self.sync_account_summary_from_network().await {
            Ok(summary) => Ok(Some(summary)),
            Err(err @ (VpnError::NoAccountStored | VpnError::NoDeviceIdentity)) => Err(err),
            Err(err) => {
                tracing::warn!("Account summary network sync failed, trying cache: {err:?}");
                match self.storage.load_summary().await.map_err(|storage_err| {
                    VpnError::Storage {
                        details: storage_err.to_string(),
                    }
                })? {
                    Some(cached) => Ok(Some(cached)),
                    None => Err(err),
                }
            }
        }
    }

    /// Get the type of account the user is logged in with
    pub async fn get_account_mode(&self) -> Result<nym_vpn_lib_types::StoredAccountMode, VpnError> {
        let account = self
            .storage
            .load_account()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoAccountStored)?;

        Ok(account.mode)
    }

    /// Load the account mnemonic stored locally and register it.
    /// This is a version that can be called when the account controller is not running.
    pub async fn register_account(&self) -> Result<RegisterAccountResponse, VpnError> {
        let platform = Platform::Apple;
        let account = self
            .storage
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

    /// Sync account summary from the VPN API and register the device when the
    /// subscription is active. Intended immediately after [`Self::register_account`]
    /// on iOS login paths so zk-nym prefetch and connect assume a registered device.
    pub async fn prepare_registered_account(&self) -> Result<(), VpnError> {
        tracing::info!("Starting post-login account setup (summary sync and device registration)");
        let vpn_api_client = self.create_vpn_api_client().await?;
        let account = self
            .storage
            .load_account()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoAccountStored)?;
        let account = VpnAccount::try_from(account).map_err(VpnError::internal)?;
        let device = self.load_device().await?;

        let mut summary = self
            .sync_account_summary_from_network_with_client(&vpn_api_client)
            .await?;

        if summary.is_account_active()
            && !summary.is_subscription_pending()
            && summary.is_subscription_active()
        {
            let registered =
                register_device_if_needed(&vpn_api_client, &account, &device, &mut summary)
                    .await
                    .map_err(|err| VpnError::InternalError {
                        details: err.to_string(),
                    })?;

            if summary.is_device_active {
                verify_time_synced(&summary).map_err(|err| VpnError::InternalError {
                    details: err.to_string(),
                })?;
            }

            if registered {
                tracing::info!(
                    "Device registered for account {} during post-login setup",
                    account.id()
                );
            }
        }

        self.storage
            .store_summary(summary)
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?;

        tracing::info!("Post-login account setup completed");
        Ok(())
    }

    /// Prefetch zk-nyms into the local credential store without a running
    /// account controller, so the next connect can skip the zk-nym fetch
    /// during `AwaitingAccountReadiness`.
    ///
    /// Returns [`VpnPrefetchZkNymOutcome`] so callers can distinguish whether
    /// tickets were already sufficient, newly fetched, or unavailable (upgrade mode).
    ///
    /// Caller invariant: must not run while a controller owns the same data
    /// dir (the network extension at connect, or an in-process controller).
    pub async fn prefetch_zk_nyms(&self) -> Result<VpnPrefetchZkNymOutcome, VpnError> {
        tracing::info!("Starting zk-nym prefetch from app storage API");
        let account = self
            .storage
            .load_account()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoAccountStored)?;
        let account = VpnAccount::try_from(account).map_err(VpnError::internal)?;

        let device = self.load_device().await?;
        let vpn_api_client = self.create_vpn_api_client().await?;

        let mut summary = self.load_cached_summary_or_sync(&vpn_api_client).await?;

        if !summary.is_device_active {
            tracing::warn!(
                "prefetch_zk_nyms: device not registered; attempting repair registration before zk-nym fetch"
            );
            register_device_if_needed(&vpn_api_client, &account, &device, &mut summary)
                .await
                .map_err(|err| VpnError::InternalError {
                    details: err.to_string(),
                })?;
            self.storage
                .store_summary(summary.clone())
                .await
                .map_err(|err| VpnError::Storage {
                    details: err.to_string(),
                })?;
        }

        let fair_usage_left = summary.fair_usage_left();

        nym_vpn_account_controller::prefetch_zk_nyms(
            self.storage_path.clone(),
            vpn_api_client,
            Arc::new(account),
            device,
            fair_usage_left,
        )
        .await
        .map(VpnPrefetchZkNymOutcome::from)
        .map_err(|err| VpnError::ZkNymAcquisitionFailure {
            details: err.to_string(),
        })
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
    async fn sync_account_summary_from_network(&self) -> Result<VpnAccountSummary, VpnError> {
        let vpn_api_client = self.create_vpn_api_client().await?;
        self.sync_account_summary_from_network_with_client(&vpn_api_client)
            .await
    }

    async fn sync_account_summary_from_network_with_client(
        &self,
        vpn_api_client: &VpnApiClient,
    ) -> Result<VpnAccountSummary, VpnError> {
        let account = self
            .storage
            .load_account()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoAccountStored)?;
        let account = VpnAccount::try_from(account).map_err(VpnError::internal)?;
        let device = self.load_device().await?;

        tracing::info!(
            "Fetching account summary from VPN API for account {}",
            account.id()
        );

        let remote_time =
            vpn_api_client
                .get_remote_time()
                .await
                .map_err(|err| VpnError::InternalError {
                    details: format!("Failed to get remote time: {err}"),
                })?;

        let api_summary = vpn_api_client
            .get_account_summary_with_device(&account, &device)
            .await
            .map_err(|err| VpnError::InternalError {
                details: format!("Failed to get account summary: {err}"),
            })?;

        let summary = VpnAccountSummary::from_parts(&api_summary, account.mode(), remote_time)
            .map_err(|err| VpnError::InternalError {
                details: format!("Failed to parse account summary: {err}"),
            })?;

        self.storage
            .store_summary(summary.clone())
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?;

        tracing::info!(
            "Account summary synced: subscription_active={}, is_device_active={}",
            summary.is_subscription_active(),
            summary.is_device_active
        );

        Ok(summary)
    }

    async fn load_cached_summary_or_sync(
        &self,
        vpn_api_client: &VpnApiClient,
    ) -> Result<VpnAccountSummary, VpnError> {
        if let Some(summary) =
            self.storage
                .load_summary()
                .await
                .map_err(|err| VpnError::Storage {
                    details: err.to_string(),
                })?
            && !summary.is_stale(SUMMARY_STALE_AFTER)
        {
            tracing::debug!("Using cached account summary for zk-nym prefetch");
            return Ok(summary);
        }

        self.sync_account_summary_from_network_with_client(vpn_api_client)
            .await
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
