// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, str::FromStr, time::Duration};

use nym_common::trace_err_chain;
use nym_offline_monitor::ConnectivityHandle;
use nym_vpn_account_controller::{AccountCommandSender, AccountStateReceiver};
use nym_vpn_api_client::types::{Platform, VpnAccount};
use nym_vpn_lib::storage::VpnClientOnDiskStorage;
use nym_vpn_lib_types_uniffi::{AccountControllerState, RegisterAccountResponse};
use nym_vpn_network_config::Network;
use nym_vpn_store::{
    keys::device::DeviceKeyStore,
    mnemonic::{AccountStorage, Mnemonic},
};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::{ACCOUNT_CONTROLLER_HANDLE, error::VpnError};
use crate::offline_monitor;

pub(super) async fn init_account_controller(
    data_dir: PathBuf,
    credential_mode: Option<bool>,
    network: Network,
) -> Result<(), VpnError> {
    let mut guard = ACCOUNT_CONTROLLER_HANDLE.lock().await;

    if guard.is_none() {
        let account_controller_handle = start_account_controller(
            data_dir,
            credential_mode,
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
    credential_mode: Option<bool>,
    network_env: Network,
    connectivity_handle: ConnectivityHandle,
) -> Result<AccountControllerHandle, VpnError> {
    let storage = VpnClientOnDiskStorage::new(data_dir.clone());
    // TODO: pass in as argument
    let user_agent = crate::user_agent::construct_user_agent();
    let shutdown_token = CancellationToken::new();

    let nym_vpn_api_client = nym_vpn_api_client::VpnApiClient::from_network(
        network_env.nym_network_details(),
        user_agent,
    )
    .map_err(|err| VpnError::InternalError {
        details: err.to_string(),
    })?;

    let account_controller_config = nym_vpn_account_controller::AccountControllerConfig {
        data_dir,
        credentials_mode: credential_mode,
        network_env,
    };

    let account_controller = nym_vpn_account_controller::AccountController::new(
        nym_vpn_api_client,
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
    let account_controller_handle = tokio::spawn(account_controller.run());

    Ok(AccountControllerHandle {
        command_sender,
        state_receiver,
        handle: account_controller_handle,
        shutdown_token,
    })
}

pub(super) struct AccountControllerHandle {
    command_sender: AccountCommandSender,
    state_receiver: AccountStateReceiver,
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

pub(super) async fn wait_for_account_ready_to_connect(timeout: Duration) -> Result<(), VpnError> {
    let mut state_receiver = get_state_receiver().await?;
    tokio::time::timeout(timeout, state_receiver.wait_for_account_ready_to_connect())
        .await
        .map_err(|_| VpnError::VpnApiTimeout)?
        .map_err(VpnError::from)
}

pub(super) async fn get_account_state() -> Result<AccountControllerState, VpnError> {
    let state_receiver = get_state_receiver().await?;
    Ok(state_receiver.get_state().into())
}

pub(super) async fn update_account_state() -> Result<(), VpnError> {
    get_command_sender()
        .await?
        .background_refresh_account_state()
        .await
        .map_err(VpnError::from)
}

async fn parse_mnemonic(mnemonic: &str) -> Result<Mnemonic, VpnError> {
    Mnemonic::parse(mnemonic).map_err(|err| VpnError::InvalidMnemonic {
        details: err.to_string(),
    })
}

pub(super) async fn login(mnemonic: &str) -> Result<(), VpnError> {
    let mnemonic = parse_mnemonic(mnemonic).await?;
    get_command_sender().await?.store_account(mnemonic).await?;
    Ok(())
}

pub(super) async fn create_account() -> Result<(), VpnError> {
    get_command_sender()
        .await?
        .create_account_command()
        .await
        .map_err(VpnError::from)
}

pub(super) async fn register_account() -> Result<RegisterAccountResponse, VpnError> {
    let mnemonic = get_command_sender()
        .await?
        .get_stored_account()
        .await
        .map_err(VpnError::from)?
        .ok_or(VpnError::NoAccountStored)?;
    let platform = if cfg!(target_os = "ios") {
        Platform::Apple
    } else {
        return Err(VpnError::InternalError {
            details: "only iOS supported for now".to_string(),
        });
    };
    get_command_sender()
        .await?
        .register_account(mnemonic, platform)
        .await
        .map(RegisterAccountResponse::from)
        .map_err(VpnError::from)
}

pub(super) async fn forget_account() -> Result<(), VpnError> {
    get_command_sender()
        .await?
        .forget_account()
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
        .to_string())
}

pub(super) async fn get_device_id() -> Result<String, VpnError> {
    get_command_sender()
        .await?
        .get_device_identity()
        .await?
        .ok_or(VpnError::NoAccountStored)
}

// Raw API that directly accesses storage without going through the account controller.
// This API places the responsibility of ensuring the account controller is not running on
// the caller.
//
// WARN: This API was added mostly as a workaround for unblocking the iOS client, and is not a
// sustainable long term solution.
pub(crate) mod raw {
    use std::path::Path;

    use nym_common::ErrorExt;
    use nym_sdk::mixnet::StoragePaths;
    use nym_vpn_api_client::{
        VpnApiClient,
        response::{NymVpnAccountResponse, NymVpnRegisterAccountResponse},
        types::{Device, DeviceStatus},
    };

    use super::*;
    use crate::environment;

    async fn setup_account_storage(path: &str) -> Result<VpnClientOnDiskStorage, VpnError> {
        assert_account_controller_not_running().await?;
        let path = PathBuf::from_str(path).map_err(|err| VpnError::InvalidAccountStoragePath {
            details: err.to_string(),
        })?;
        Ok(VpnClientOnDiskStorage::new(path))
    }

    pub(crate) async fn login_raw(mnemonic: &str, path: &str) -> Result<(), VpnError> {
        let mnemonic = parse_mnemonic(mnemonic).await?;
        get_account_by_mnemonic_raw(mnemonic.clone()).await?;
        let storage = setup_account_storage(path).await?;
        storage.store_mnemonic(mnemonic).await?;
        storage.init_keys(None).await?;
        Ok(())
    }

    pub(crate) async fn create_account_raw(path: &str) -> Result<(), VpnError> {
        let (_, mnemonic) = VpnAccount::random().map_err(VpnError::internal)?;
        let storage = setup_account_storage(path).await?;
        storage.store_mnemonic(mnemonic.clone()).await?;
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
        let mnemonic = storage
            .load_mnemonic()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoAccountStored)?;
        let account = VpnAccount::try_from(mnemonic).map_err(VpnError::internal)?;
        let account_token = register_account_by_account_raw(&account, platform)
            .await?
            .account_token;
        Ok(RegisterAccountResponse { account_token })
    }

    pub(crate) async fn is_account_mnemonic_stored_raw(path: &str) -> Result<bool, VpnError> {
        let storage = setup_account_storage(path).await?;
        storage.is_mnemonic_stored().await.map_err(Into::into)
    }

    pub(crate) async fn get_stored_mnemonic_raw(path: &str) -> Result<String, VpnError> {
        let storage = setup_account_storage(path).await?;
        Ok(storage
            .load_mnemonic()
            .await?
            .ok_or(VpnError::NoAccountStored)?
            .to_string())
    }

    pub(crate) async fn get_account_id_raw(path: &str) -> Result<String, VpnError> {
        let storage = setup_account_storage(path).await?;
        let mnemonic = storage
            .load_mnemonic()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoAccountStored)?;
        VpnAccount::try_from(mnemonic)
            .map_err(VpnError::internal)
            .map(|account| account.id().to_string())
    }

    async fn remove_account_mnemonic_raw(path: &str) -> Result<bool, VpnError> {
        let storage = setup_account_storage(path).await?;
        storage
            .remove_mnemonic()
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

    async fn create_vpn_api_client() -> Result<VpnApiClient, VpnError> {
        let network_env = environment::current_environment_details().await?;
        let user_agent = crate::user_agent::construct_user_agent();
        let vpn_api_client =
            VpnApiClient::from_network(network_env.nym_network_details(), user_agent)
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
        let mnemonic = account_storage
            .load_mnemonic()
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?
            .ok_or(VpnError::NoAccountStored)?;
        let account = VpnAccount::try_from(mnemonic).map_err(VpnError::internal)?;

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
        let account = VpnAccount::try_from(mnemonic).map_err(VpnError::internal)?;
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
            .post_account(account, platform)
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
        nym_vpn_account_controller::remove_files_for_account(&path_buf)
            .await
            .map_err(|err| VpnError::Storage {
                details: err.to_string(),
            })?;

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
}
