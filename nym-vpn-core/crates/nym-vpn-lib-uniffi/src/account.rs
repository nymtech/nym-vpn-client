// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, sync::Arc, time::Duration};

use tokio::{sync::Mutex, task::JoinHandle};
use tokio_util::sync::{CancellationToken, DropGuard};

use nym_vpn_account_controller::{AccountCommandSender, AccountStateReceiver, NyxdClient};
use nym_vpn_api_client::types::Platform;
use nym_vpn_lib::storage::VpnClientOnDiskStorage;
use nym_vpn_lib_types::{
    AccountControllerState, DeeplinkClient, DeeplinkKind, GetDeeplinkParams,
    RegisterAccountResponse, StoreAccountRequest, UserAgent, VpnAccountSummary,
};

use crate::{environment::NymEnvironment, error::VpnError, offline_monitor::NymOfflineMonitor};

struct State {
    join_handle: JoinHandle<()>,
    shutdown_drop_guard: DropGuard,
}

#[derive(uniffi::Object)]
pub struct NymAccountController {
    command_sender: AccountCommandSender,
    state_receiver: AccountStateReceiver,
    network_env: Arc<NymEnvironment>,
    state: Arc<Mutex<Option<State>>>,
}

#[uniffi::export(async_runtime = "tokio")]
impl NymAccountController {
    #[uniffi::constructor]
    pub async fn new(
        data_dir: PathBuf,
        user_agent: UserAgent,
        network_env: Arc<NymEnvironment>,
        offline_monitor: Arc<NymOfflineMonitor>,
    ) -> Result<Self, VpnError> {
        let storage = VpnClientOnDiskStorage::new(data_dir.clone());
        let shutdown_token = CancellationToken::new();

        let nym_vpn_api_client = nym_vpn_api_client::VpnApiClient::from_network(
            network_env.inner().nym_network_details(),
            Some(user_agent.into()),
            None,
        )
        .await
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })?;

        let nyxd_client = NyxdClient::new(&network_env.inner());
        let account_controller_config = nym_vpn_account_controller::AccountControllerConfig {
            data_dir,
            network_env: network_env.inner().clone(),
        };

        let connectivity_handle = offline_monitor.inner();
        let account_controller = nym_vpn_account_controller::AccountController::new(
            nym_vpn_api_client,
            nyxd_client,
            account_controller_config,
            storage,
            connectivity_handle,
            shutdown_token.child_token(),
        )
        .await
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })?;

        let command_sender = account_controller.get_command_sender();
        let state_receiver = account_controller.get_state_receiver();
        let join_handle = tokio::spawn(account_controller.run());
        let shutdown_drop_guard = shutdown_token.drop_guard();

        Ok(Self {
            command_sender,
            state_receiver,
            network_env,
            state: Arc::new(Mutex::new(Some(State {
                join_handle,
                shutdown_drop_guard,
            }))),
        })
    }

    pub async fn shutdown_and_wait(&self) {
        let Some(state) = self.state.lock().await.take() else {
            return;
        };

        drop(state.shutdown_drop_guard);
        if let Err(err) = state.join_handle.await {
            tracing::error!("Failed to wait on account controller join handle: {err}");
        }
    }

    pub async fn wait_for_account_ready_to_connect(
        &self,
        timeout: Duration,
    ) -> Result<(), VpnError> {
        let mut cloned_receiver = self.state_receiver.clone();
        tokio::time::timeout(timeout, cloned_receiver.wait_for_account_ready_to_connect())
            .await
            .map_err(|_| VpnError::VpnApiTimeout)?
            .map_err(VpnError::from)
    }

    pub async fn get_deeplink(&self, params: GetDeeplinkParams) -> Result<String, VpnError> {
        let base_url = match params.kind {
            DeeplinkKind::Privy => {
                let Some(ref account_management) =
                    self.network_env.inner().nym_vpn_network.account_management
                else {
                    return Err(VpnError::DeeplinkError {
                        details: "No account management data is available at this time".to_owned(),
                    });
                };

                let opt_url = match params.client {
                    DeeplinkClient::Mobile => account_management.privy_mobile_url(&params.locale),
                    DeeplinkClient::Desktop => account_management.privy_desktop_url(&params.locale),
                    DeeplinkClient::Web => account_management.privy_web_url(&params.locale),
                };

                opt_url.ok_or(VpnError::DeeplinkError {
                    details: "The privy path could not be determined".to_owned(),
                })?
            }
        };

        self.command_sender
            .get_deeplink(params.kind, params.name, base_url)
            .await
            .map_err(VpnError::from)
    }

    pub async fn login_with_deeplink(&self, deeplink_callback_url: String) -> Result<(), VpnError> {
        let mnemonic = self
            .command_sender
            .derive_deeplink_mnemonic(deeplink_callback_url)
            .await?;

        self.command_sender
            .store_account(mnemonic.into())
            .await
            .map_err(VpnError::from)
    }

    pub async fn get_account_summary(&self) -> Result<Option<VpnAccountSummary>, VpnError> {
        self.command_sender
            .get_account_summary()
            .await
            .map_err(VpnError::from)
    }

    /// Get the account state
    pub async fn get_account_state(&self) -> AccountControllerState {
        self.state_receiver.get_state()
    }

    /// This manually syncs the account state with the server. Normally this is done automatically, but
    /// this can be used to manually trigger a sync.
    pub async fn update_account_state(&self) -> Result<(), VpnError> {
        self.command_sender
            .background_refresh_account_state()
            .await
            .map_err(VpnError::from)
    }

    /// Import the account mnemonic
    pub async fn login(&self, request: &StoreAccountRequest) -> Result<(), VpnError> {
        let mnemonic = nym_vpn_lib::login::parse_account_request(request).map_err(|err| {
            VpnError::InvalidSecret {
                details: err.to_string(),
            }
        })?;
        self.command_sender.store_account(mnemonic.into()).await?;
        Ok(())
    }

    /// Generate the account mnemonic locally and store it.
    pub async fn create_account(&self) -> Result<(), VpnError> {
        self.command_sender
            .create_account_command()
            .await
            .map_err(VpnError::from)
    }

pub(super) async fn register_account(
    args: crate::AccountRegistrationArgs,
) -> Result<RegisterAccountResponse, VpnError> {
    let stored_accounts = get_command_sender().await?.get_stored_accounts().await?;

    let Some(stored_account) = stored_accounts
        .into_iter()
        .find(|account| account.mode == StoredAccountMode::Api)
    else {
        return Err(VpnError::NoAccountStored);
    };

    let platform = Platform::try_from(args)?;
    get_command_sender()
        .await?
        .register_account(stored_account, platform)
        .await
        .map_err(VpnError::from)
}

pub(super) async fn forget_account() -> Result<(), VpnError> {
    get_command_sender()
        .await?
        .forget_account(Some(StoredAccountMode::Api))
        .await
        .map_err(VpnError::from)
}

    /// Force a rotation of the wireguard keys
    pub async fn rotate_keys(&self) -> Result<(), VpnError> {
        self.command_sender
            .rotate_keys()
            .await
            .map_err(VpnError::from)
    }

    /// Get the account identity
    pub async fn get_account_identity(&self) -> Result<Option<String>, VpnError> {
        Ok(self.command_sender.get_account_id().await?)
    }

    /// Check if the account mnemonic is stored
    pub async fn is_account_mnemonic_stored(&self) -> Result<bool, VpnError> {
        Ok(self.command_sender.get_account_id().await?.is_some())
    }

pub(super) async fn get_stored_mnemonic() -> Result<String, VpnError> {
    let stored_accounts = get_command_sender().await?.get_stored_accounts().await?;

    let Some(stored_account) = stored_accounts
        .into_iter()
        .find(|account| account.mode == StoredAccountMode::Api)
    else {
        return Err(VpnError::NoAccountStored);
    };

    Ok(stored_account.mnemonic.to_string())
}
    /// Read and return the mnemonic, if there's one stored.
    pub async fn get_stored_mnemonic(&self) -> Result<String, VpnError> {
        Ok(self
            .command_sender
            .get_stored_account()
            .await
            .map_err(VpnError::from)?
            .ok_or(VpnError::NoAccountStored)?
            .mnemonic
            .to_string())
    }

    /// Get the device identity
    pub async fn get_device_identity(&self) -> Result<String, VpnError> {
        self.command_sender
            .get_device_identity()
            .await?
            .ok_or(VpnError::NoAccountStored)
    }
pub(super) async fn get_device_id() -> Result<String, VpnError> {
    get_command_sender()
        .await?
        .get_device_identity()
        .await?
        .ok_or(VpnError::NoAccountStored)
}

pub(super) async fn get_deeplink(params: GetDeeplinkParams) -> Result<String, VpnError> {
    let base_url = match params.kind {
        DeeplinkKind::Privy | DeeplinkKind::PrivyAssociate => {
            let Some(ref account_management) = current_environment_details()
                .await
                .ok()
                .and_then(|network| network.nym_vpn_network.account_management)
            else {
                return Err(VpnError::DeeplinkError {
                    details: "No account management data is available at this time".to_string(),
                });
            };

            let opt_url = match params.client {
                DeeplinkClient::Mobile => account_management.privy_mobile_url(&params.locale),
                DeeplinkClient::Desktop => account_management.privy_desktop_url(&params.locale),
                DeeplinkClient::Web => account_management.privy_web_url(&params.locale),
            };

            opt_url.ok_or(VpnError::DeeplinkError {
                details: "The privy path could not be determined".to_string(),
            })?
        }
    };

    get_command_sender()
        .await?
        .get_deeplink(params.kind, params.name, base_url)
        .await
        .map_err(VpnError::from)
}

#[derive(uniffi::Record)]
pub struct AccountRegistrationArgs {
    #[cfg(target_os = "android")]
    pub purchase_token: String,
}

impl TryFrom<AccountRegistrationArgs> for nym_vpn_api_client::types::Platform {
    type Error = VpnError;

    fn try_from(_value: AccountRegistrationArgs) -> Result<Self, Self::Error> {
        #[cfg(target_os = "ios")]
        return Ok(nym_vpn_api_client::types::Platform::Apple);
        #[cfg(target_os = "android")]
        return Ok(nym_vpn_api_client::types::Platform::Android {
            purchase_token: _value.purchase_token,
        });
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        Err(VpnError::InternalError {
            details: "only iOS and Android supported for now".to_string(),
        })
    }

    async fn get_account_by_mnemonic_raw(
        mnemonic: Mnemonic,
    ) -> Result<NymVpnAccountResponse, VpnError> {
        let vpn_api_client = create_vpn_api_client().await?;
        let account = VpnAccount::new(mnemonic, VpnAccountMode::Api).map_err(VpnError::internal)?;
        vpn_api_client
            .get_account(&account)
            .await
            .map_err(|_err| VpnError::AccountNotRegistered)
    }

    async fn register_account_by_account_raw(
        account: &VpnAccount,
        platform: Platform,
    ) -> Result<NymVpnRegisterAccountResponse, VpnError> {
        let vpn_api_client = create_vpn_api_client().await?;
        vpn_api_client
            .register_account(account, platform)
            .await
            .map_err(|err| VpnError::FailedAccountRegistration {
                details: err.display_chain(),
            })
    }

    pub(crate) async fn forget_account_raw(path: &str) -> Result<(), VpnError> {
        tracing::info!("REMOVING ALL ACCOUNT AND DEVICE DATA IN: {path}");

        let path_buf =
            PathBuf::from_str(path).map_err(|err| VpnError::InvalidAccountStoragePath {
                details: err.to_string(),
            })?;

        unregister_device_raw(path)
            .await
            .inspect(|_| tracing::info!("Device has been unregistered"))
            .inspect_err(|err| tracing::error!("Failed to unregister device: {err:?}"))
            .ok();

        // First remove the files we own directly
        remove_account_mnemonic_raw(path).await?;
        remove_device_identity_raw(path).await?;
        remove_credential_storage_raw(&path_buf).await?;

        // Then remove the rest of the files, that we own indirectly
        nym_vpn_account_controller::remove_files_for_account(&path_buf, true)
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?;

        Ok(())
    }

    pub(crate) async fn rotate_keys_raw(path: &str) -> Result<(), VpnError> {
        let path_buf =
            PathBuf::from_str(path).map_err(|err| VpnError::InvalidAccountStoragePath {
                details: err.to_string(),
            })?;
        remove_wireguard_keys_storage_raw(&path_buf).await?;

        Ok(())
    }

    pub(crate) async fn get_device_id_raw(path: &str) -> Result<String, VpnError> {
        let storage = setup_account_storage(path).await?;
        let device_id = storage
            .load_keys()
            .await
            .map_err(|_err| VpnError::NoDeviceIdentity)?
            .ok_or(VpnError::NoDeviceIdentity)?;
        Ok(device_id.device_keypair().public_key().to_string())
    }

    pub(crate) async fn remove_device_identity_raw(path: &str) -> Result<(), VpnError> {
        let storage = setup_account_storage(path).await?;
        storage.remove_keys().await.map_err(VpnError::internal)
    }

    pub(crate) async fn get_deeplink(params: GetDeeplinkParams) -> Result<String, VpnError> {
        let base_url = match params.kind {
            DeeplinkKind::Privy | DeeplinkKind::PrivyAssociate => {
                let Some(ref account_management) = current_environment_details()
                    .await
                    .ok()
                    .and_then(|network| network.nym_vpn_network.account_management)
                else {
                    return Err(VpnError::DeeplinkError {
                        details: "No account management data is available at this time".to_string(),
                    });
                };

                let opt_url = match params.client {
                    DeeplinkClient::Mobile => account_management.privy_mobile_url(&params.locale),
                    DeeplinkClient::Desktop => account_management.privy_desktop_url(&params.locale),
                    DeeplinkClient::Web => account_management.privy_web_url(&params.locale),
                };

                opt_url.ok_or(VpnError::DeeplinkError {
                    details: "The privy path could not be determined".to_string(),
                })?
            }
        };

        let mut deeplink_guard = DEEPLINKS.lock().await;

        if deeplink_guard.is_none() {
            let deeplinks = Deeplinks::default();
            *deeplink_guard = Some(deeplinks);
        }

        let deeplinks = deeplink_guard.as_mut().ok_or(VpnError::DeeplinkError {
            details: "Failed to access deeplinks storage".to_string(),
        })?;

        let params = CreateDeeplinkParams {
            kind: params.kind,
            name: params.name,
            base_url,
        };

        // Create a new Deeplink for this request
        let deeplink = deeplinks
            .create_deeplink(&params)
            .map_err(|e| VpnError::DeeplinkError {
                details: e.to_string(),
            })?;

        // Create the deeplink URL
        let url = deeplink.create_url(&params.base_url);

        // Housekeeping
        deeplinks.remove_expired();

        Ok(url.to_string())
    }

    pub(crate) async fn deeplink_store_account(
        path: &str,
        deeplink_callback_url: &str,
    ) -> Result<(), VpnError> {
        let mut deeplink_guard = DEEPLINKS.lock().await;
        let deeplinks = deeplink_guard.as_mut().ok_or(VpnError::DeeplinkError {
            details: "Failed to access deeplinks storage".to_string(),
        })?;

        // Derive the mnemonic from the provided deeplink URL
        let mnemonic = deeplinks
            .derive_mnemonic(deeplink_callback_url)
            .map_err(|e| VpnError::DeeplinkError {
                details: e.to_string(),
            })?;

        // Housekeeping
        deeplinks.remove_expired();

        login_inner(mnemonic, path).await
    }
}
