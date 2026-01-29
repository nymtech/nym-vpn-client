// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, str::FromStr, time::Duration};

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::{ACCOUNT_CONTROLLER_HANDLE, DEEPLINKS, error::VpnError};
use crate::{environment::current_environment_details, offline_monitor};
use nym_common::trace_err_chain;
use nym_offline_monitor::ConnectivityHandle;
use nym_vpn_account_controller::{AccountCommandSender, AccountStateReceiver, NyxdClient};
use nym_vpn_api_client::types::{Platform, VpnAccount};
use nym_vpn_lib::{new_user_agent, storage::VpnClientOnDiskStorage};
use nym_vpn_lib_types::{
    AccountControllerState, DeeplinkClient, DeeplinkKind, GetDeeplinkParams,
    RegisterAccountResponse, StoreAccountRequest, VpnAccountSummary,
};
use nym_vpn_network_config::Network;
use nym_vpn_store::{
    account::Mnemonic,
    keys::{device::DeviceKeyStore, wireguard::WireguardKeysDb},
};

pub(super) async fn init_account_controller(
    data_dir: PathBuf,
    network: Network,
) -> Result<(), VpnError> {
    let mut guard = ACCOUNT_CONTROLLER_HANDLE.lock().await;

    if guard.is_none() {
        let account_controller_handle = start_account_controller(
            data_dir,
            network,
            offline_monitor::get_connectivity_handle().await?,
        )
        .await?;
        *guard = Some(account_controller_handle);
        Ok(())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Account controller is already running.".to_owned(),
        })
    }
}

pub(super) async fn stop_account_controller() -> Result<(), VpnError> {
    let mut guard = ACCOUNT_CONTROLLER_HANDLE.lock().await;

    match guard.take() {
        Some(account_controller_handle) => {
            account_controller_handle.shutdown_and_wait().await;
            Ok(())
        }
        None => Err(VpnError::InvalidStateError {
            details: "Account controller is not running.".to_owned(),
        }),
    }
}

async fn start_account_controller(
    data_dir: PathBuf,
    network_env: Network,
    connectivity_handle: ConnectivityHandle,
) -> Result<AccountControllerHandle, VpnError> {
    let storage = VpnClientOnDiskStorage::new(data_dir.clone());
    // TODO: pass in as argument
    let user_agent = new_user_agent!();
    let shutdown_token = CancellationToken::new();

    let nym_vpn_api_client = nym_vpn_api_client::VpnApiClient::from_network(
        network_env.nym_network_details(),
        Some(user_agent),
        None,
    )
    .await
    .map_err(|err| VpnError::InternalError {
        details: err.to_string(),
    })?;

    let nyxd_client = NyxdClient::new(&network_env);

    let account_controller_config = nym_vpn_account_controller::AccountControllerConfig {
        data_dir,
        network_env,
    };

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
    let wireguard_key_db = account_controller.get_wireguard_keys_storage();
    let account_controller_handle = tokio::spawn(account_controller.run());

    Ok(AccountControllerHandle {
        command_sender,
        state_receiver,
        wireguard_key_db,
        handle: account_controller_handle,
        shutdown_token,
    })
}

pub(super) struct AccountControllerHandle {
    command_sender: AccountCommandSender,
    state_receiver: AccountStateReceiver,
    wireguard_key_db: WireguardKeysDb,
    handle: JoinHandle<()>,
    shutdown_token: CancellationToken,
}

impl AccountControllerHandle {
    async fn shutdown_and_wait(self) {
        self.shutdown_token.cancel();

        if let Err(e) = self.handle.await {
            tracing::error!("Failed to join on account controller handle: {}", e);
        }
    }
}

async fn is_account_controller_running() -> bool {
    ACCOUNT_CONTROLLER_HANDLE.lock().await.is_some()
}

async fn assert_account_controller_not_running() -> Result<(), VpnError> {
    if is_account_controller_running().await {
        Err(VpnError::InvalidStateError {
            details: "Account controller is running.".to_owned(),
        })
    } else {
        Ok(())
    }
}

pub(super) async fn get_command_sender() -> Result<AccountCommandSender, VpnError> {
    if let Some(guard) = &*ACCOUNT_CONTROLLER_HANDLE.lock().await {
        Ok(guard.command_sender.clone())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Account controller is not running.".to_owned(),
        })
    }
}

pub(super) async fn get_state_receiver() -> Result<AccountStateReceiver, VpnError> {
    if let Some(guard) = &*ACCOUNT_CONTROLLER_HANDLE.lock().await {
        Ok(guard.state_receiver.clone())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Account controller is not running.".to_owned(),
        })
    }
}

pub(super) async fn get_wireguard_key_db() -> Result<WireguardKeysDb, VpnError> {
    if let Some(guard) = &*ACCOUNT_CONTROLLER_HANDLE.lock().await {
        Ok(guard.wireguard_key_db.clone())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Account controller is not running.".to_owned(),
        })
    }
}

pub(super) async fn wait_for_account_ready_to_connect(timeout: Duration) -> Result<(), VpnError> {
    let mut state_receiver = get_state_receiver().await?;
    tokio::time::timeout(timeout, state_receiver.wait_for_account_ready_to_connect())
        .await
        .map_err(|_| VpnError::VpnApiTimeout)?
        .map_err(VpnError::from)
}

pub(super) async fn get_account_state() -> Result<AccountControllerState, VpnError> {
    let state_receiver = get_state_receiver().await?;
    Ok(state_receiver.get_state())
}

pub(super) async fn update_account_state() -> Result<(), VpnError> {
    get_command_sender()
        .await?
        .background_refresh_account_state()
        .await
        .map_err(VpnError::from)
}

pub(super) async fn login(request: &StoreAccountRequest) -> Result<(), VpnError> {
    let mnemonic = nym_vpn_lib::login::parse_account_request(request).map_err(|err| {
        VpnError::InvalidSecret {
            details: err.to_string(),
        }
    })?;
    get_command_sender()
        .await?
        .store_account(mnemonic.into())
        .await?;
    Ok(())
}

pub(super) async fn create_account() -> Result<(), VpnError> {
    get_command_sender()
        .await?
        .create_account_command()
        .await
        .map_err(VpnError::from)
}

pub(super) async fn register_account(
    args: crate::AccountRegistrationArgs,
) -> Result<RegisterAccountResponse, VpnError> {
    let mnemonic = get_command_sender()
        .await?
        .get_stored_account()
        .await
        .map_err(VpnError::from)?
        .ok_or(VpnError::NoAccountStored)?;
    let platform = Platform::try_from(args)?;
    get_command_sender()
        .await?
        .register_account(mnemonic, platform)
        .await
        .map_err(VpnError::from)
}

pub(super) async fn forget_account() -> Result<(), VpnError> {
    get_command_sender()
        .await?
        .forget_account()
        .await
        .map_err(VpnError::from)
}

pub(super) async fn rotate_keys() -> Result<(), VpnError> {
    get_command_sender()
        .await?
        .rotate_keys()
        .await
        .map_err(VpnError::from)
}

pub(super) async fn get_account_id() -> Result<Option<String>, VpnError> {
    Ok(get_command_sender().await?.get_account_id().await?)
}

pub(super) async fn is_account_mnemonic_stored() -> Result<bool, VpnError> {
    Ok(get_command_sender()
        .await?
        .get_account_id()
        .await?
        .is_some())
}

pub(super) async fn get_stored_mnemonic() -> Result<String, VpnError> {
    Ok(get_command_sender()
        .await?
        .get_stored_account()
        .await
        .map_err(VpnError::from)?
        .ok_or(VpnError::NoAccountStored)?
        .mnemonic
        .to_string())
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
        DeeplinkKind::Privy => {
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

pub(super) async fn deeplink_store_account(deeplink_callback_url: String) -> Result<(), VpnError> {
    let mnemonic = get_command_sender()
        .await?
        .derive_deeplink_mnemonic(deeplink_callback_url)
        .await?;

    get_command_sender()
        .await?
        .store_account(mnemonic.into())
        .await?;
    Ok(())
}

pub(super) async fn get_account_summary() -> Result<Option<VpnAccountSummary>, VpnError> {
    get_command_sender()
        .await?
        .get_account_summary()
        .await
        .map_err(VpnError::from)
}

// Raw API that directly accesses storage without going through the account controller.
// This API places the responsibility of ensuring the account controller is not running on
// the caller.
//
// WARN: This API was added mostly as a workaround for unblocking the iOS client, and is not a
// sustainable long term solution.
pub(crate) mod raw {
    use std::path::Path;

    use super::*;
    use nym_common::ErrorExt;
    use nym_sdk::mixnet::StoragePaths;
    use nym_vpn_account_controller::{CreateDeeplinkParams, Deeplinks};
    use nym_vpn_api_client::{
        VpnApiClient,
        response::{NymVpnAccountResponse, NymVpnRegisterAccountResponse},
        types::{Device, DeviceStatus, VpnAccountMode},
    };
    use nym_vpn_store::{account::AccountInformationStorage, keys::wireguard::DB_NAME};

    async fn setup_account_storage(path: &str) -> Result<VpnClientOnDiskStorage, VpnError> {
        assert_account_controller_not_running().await?;
        let path = PathBuf::from_str(path).map_err(|err| VpnError::InvalidAccountStoragePath {
            details: err.to_string(),
        })?;
        Ok(VpnClientOnDiskStorage::new(path))
    }

    async fn login_inner(mnemonic: Mnemonic, path: &str) -> Result<(), VpnError> {
        get_account_by_mnemonic_raw(mnemonic.clone()).await?;
        let storage = setup_account_storage(path).await?;
        storage.store_account(mnemonic.into()).await?;
        storage.init_keys(None).await?;
        Ok(())
    }

    pub(crate) async fn login_raw(
        request: &StoreAccountRequest,
        path: &str,
    ) -> Result<(), VpnError> {
        let mnemonic = nym_vpn_lib::login::parse_account_request(request).map_err(|err| {
            VpnError::InvalidSecret {
                details: err.to_string(),
            }
        })?;
        login_inner(mnemonic, path).await
    }

    pub(crate) async fn create_account_raw(path: &str) -> Result<(), VpnError> {
        let (_, mnemonic) = VpnAccount::generate_new().map_err(VpnError::internal)?;
        let storage = setup_account_storage(path).await?;
        storage.store_account(mnemonic.into()).await?;
        storage.init_keys(None).await?;
        Ok(())
    }

    pub(crate) async fn register_account_raw(
        path: &str,
    ) -> Result<RegisterAccountResponse, VpnError> {
        let platform = if cfg!(target_os = "ios") {
            Platform::Apple
        } else {
            return Err(VpnError::InternalError {
                details: "only iOS supported for now".to_string(),
            });
        };
        let storage = setup_account_storage(path).await?;
        let account = storage
            .load_account()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoAccountStored)?;
        let account = VpnAccount::try_from(account).map_err(VpnError::internal)?;
        let account_token = register_account_by_account_raw(&account, platform)
            .await?
            .account_token;
        Ok(RegisterAccountResponse { account_token })
    }

    pub(crate) async fn is_account_mnemonic_stored_raw(path: &str) -> Result<bool, VpnError> {
        let storage = setup_account_storage(path).await?;
        storage.is_account_stored().await.map_err(Into::into)
    }

    pub(crate) async fn get_stored_mnemonic_raw(path: &str) -> Result<String, VpnError> {
        let storage = setup_account_storage(path).await?;
        Ok(storage
            .load_account()
            .await?
            .ok_or(VpnError::NoAccountStored)?
            .mnemonic
            .to_string())
    }

    pub(crate) async fn get_account_id_raw(path: &str) -> Result<String, VpnError> {
        let storage = setup_account_storage(path).await?;
        let account = storage
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

    async fn remove_account_mnemonic_raw(path: &str) -> Result<bool, VpnError> {
        let storage = setup_account_storage(path).await?;
        storage
            .remove_account()
            .await
            .map(|_| true)
            .map_err(Into::into)
    }

    async fn remove_credential_storage_raw<P: AsRef<Path>>(path: P) -> Result<(), VpnError> {
        let storage_paths = StoragePaths::new_from_dir(&path).map_err(VpnError::internal)?;
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

    async fn remove_wireguard_keys_storage_raw(data_dir: &Path) -> Result<(), VpnError> {
        let db_path = data_dir.join(DB_NAME);
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

    async fn create_vpn_api_client() -> Result<VpnApiClient, VpnError> {
        let network_env = current_environment_details().await?;
        let user_agent = new_user_agent!();
        let vpn_api_client =
            VpnApiClient::from_network(network_env.nym_network_details(), Some(user_agent), None)
                .await
                .map_err(VpnError::internal)?;
        Ok(vpn_api_client)
    }

    async fn load_device(path: &str) -> Result<Device, VpnError> {
        let account_storage = setup_account_storage(path).await?;
        let device_id = account_storage
            .load_keys()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoDeviceIdentity)?;
        Ok(Device::from(device_id.device_keypair().clone()))
    }

    async fn unregister_device_raw(path: &str) -> Result<(), VpnError> {
        let account_storage = setup_account_storage(path).await?;
        let device = load_device(path).await?;
        let account = account_storage
            .load_account()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoAccountStored)?;
        let account = VpnAccount::try_from(account).map_err(VpnError::internal)?;

        let vpn_api_client = create_vpn_api_client().await?;

        vpn_api_client
            .update_device(&account, &device, DeviceStatus::DeleteMe)
            .await
            .map(|_| ())
            .map_err(|err| VpnError::UnregisterDevice {
                details: err.to_string(),
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
            DeeplinkKind::Privy => {
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
