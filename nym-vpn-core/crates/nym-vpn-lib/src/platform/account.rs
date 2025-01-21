// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, str::FromStr, sync::Arc, time::Duration};

use nym_vpn_account_controller::{
    shared_state::DeviceState, AccountControllerCommander, SharedAccountState,
};
use nym_vpn_api_client::{response::NymVpnAccountSummaryResponse, types::VpnApiAccount};
use nym_vpn_network_config::Network;
use nym_vpn_store::{
    keys::KeyStore,
    mnemonic::{Mnemonic, MnemonicStorage},
};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::uniffi_custom_impls::AccountStateSummary;

use super::{error::VpnError, ACCOUNT_CONTROLLER_HANDLE};

pub(super) async fn init_account_controller(
    data_dir: PathBuf,
    credential_mode: Option<bool>,
    network: Network,
) -> Result<(), VpnError> {
    let mut guard = ACCOUNT_CONTROLLER_HANDLE.lock().await;

    if guard.is_none() {
        let account_controller_handle =
            start_account_controller(data_dir, credential_mode, network).await?;
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
) -> Result<AccountControllerHandle, VpnError> {
    let storage = Arc::new(tokio::sync::Mutex::new(
        crate::storage::VpnClientOnDiskStorage::new(data_dir.clone()),
    ));
    // TODO: pass in as argument
    let user_agent = crate::util::construct_user_agent();
    let shutdown_token = CancellationToken::new();
    let account_controller = nym_vpn_account_controller::AccountController::new(
        Arc::clone(&storage),
        data_dir.clone(),
        user_agent,
        credential_mode,
        network_env,
        shutdown_token.child_token(),
    )
    .await
    .map_err(|err| VpnError::InternalError {
        details: err.to_string(),
    })?;

    let shared_account_state = account_controller.shared_state();
    let command_sender = account_controller.commander();
    let account_controller_handle = tokio::spawn(account_controller.run());

    Ok(AccountControllerHandle {
        command_sender,
        shared_state: shared_account_state,
        handle: account_controller_handle,
        shutdown_token,
    })
}

pub(super) struct AccountControllerHandle {
    command_sender: AccountControllerCommander,
    shared_state: nym_vpn_account_controller::SharedAccountState,
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

async fn get_shared_account_state() -> Result<SharedAccountState, VpnError> {
    if let Some(guard) = &*ACCOUNT_CONTROLLER_HANDLE.lock().await {
        Ok(guard.shared_state.clone())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Account controller is not running.".to_owned(),
        })
    }
}

pub(super) async fn get_command_sender() -> Result<AccountControllerCommander, VpnError> {
    if let Some(guard) = &*ACCOUNT_CONTROLLER_HANDLE.lock().await {
        Ok(guard.command_sender.clone())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Account controller is not running.".to_owned(),
        })
    }
}

pub(super) async fn wait_for_update_account(
) -> Result<Option<NymVpnAccountSummaryResponse>, VpnError> {
    get_command_sender()
        .await?
        .ensure_update_account()
        .await
        .map_err(VpnError::from)
}

pub(super) async fn wait_for_update_device() -> Result<DeviceState, VpnError> {
    get_command_sender()
        .await?
        .ensure_update_device()
        .await
        .map_err(VpnError::from)
}

pub(super) async fn wait_for_register_device() -> Result<(), VpnError> {
    get_command_sender()
        .await?
        .ensure_register_device()
        .await
        .map_err(VpnError::from)
}

pub(super) async fn wait_for_available_zk_nyms() -> Result<(), VpnError> {
    get_command_sender()
        .await?
        .ensure_available_zk_nyms()
        .await
        .map_err(VpnError::from)
}

pub(super) async fn wait_for_account_ready_to_connect(
    credential_mode: bool,
    timeout: Duration,
) -> Result<(), VpnError> {
    let command_sender = get_command_sender().await?;
    tokio::time::timeout(
        timeout,
        command_sender.wait_for_account_ready_to_connect(credential_mode),
    )
    .await
    .map_err(|_| VpnError::VpnApiTimeout)?
    .map_err(VpnError::from)
}

pub(super) async fn get_account_state() -> Result<AccountStateSummary, VpnError> {
    let shared_account_state = get_shared_account_state().await?;
    let account_state_summary = shared_account_state.lock().await.clone();
    Ok(AccountStateSummary::from(account_state_summary))
}

pub(super) async fn update_account_state() -> Result<(), VpnError> {
    get_command_sender()
        .await?
        .sync_account_state()
        .await
        .map_err(VpnError::from)
        .map(|_| ())
}

pub(super) async fn store_account_mnemonic(mnemonic: &str) -> Result<(), VpnError> {
    let mnemonic = Mnemonic::parse(mnemonic).map_err(|err| VpnError::InternalError {
        details: err.to_string(),
    })?;

    get_command_sender()
        .await?
        .store_account(mnemonic)
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

pub(super) async fn get_account_id() -> Result<Option<String>, VpnError> {
    Ok(get_shared_account_state().await?.get_account_id().await)
}

pub(super) async fn is_account_mnemonic_stored() -> Result<bool, VpnError> {
    Ok(get_shared_account_state().await?.is_account_stored().await)
}

pub(super) async fn get_device_id() -> Result<String, VpnError> {
    get_command_sender()
        .await?
        .get_device_identity()
        .await
        .map_err(VpnError::from)
}

// Raw API that does not interact with the account controller
pub(crate) mod raw {
    use std::path::Path;

    use super::*;
    use crate::platform::environment;
    use nym_sdk::mixnet::StoragePaths;
    use nym_vpn_api_client::types::{Device, DeviceStatus};
    use nym_vpn_api_client::VpnApiClient;

    async fn setup_account_storage(
        path: &str,
    ) -> Result<crate::storage::VpnClientOnDiskStorage, VpnError> {
        assert_account_controller_not_running().await?;
        let path = PathBuf::from_str(path).map_err(|err| VpnError::InvalidAccountStoragePath {
            details: err.to_string(),
        })?;
        Ok(crate::storage::VpnClientOnDiskStorage::new(path))
    }

    pub(crate) async fn store_account_mnemonic_raw(
        mnemonic: &str,
        path: &str,
    ) -> Result<(), VpnError> {
        let storage = setup_account_storage(path).await?;

        let mnemonic = Mnemonic::parse(mnemonic).map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })?;

        storage
            .store_mnemonic(mnemonic)
            .await
            .map_err(|err| VpnError::InternalError {
                details: err.to_string(),
            })?;

        storage
            .init_keys(None)
            .await
            .map_err(|err| VpnError::InternalError {
                details: err.to_string(),
            })?;

        Ok(())
    }

    pub(crate) async fn is_account_mnemonic_stored_raw(path: &str) -> Result<bool, VpnError> {
        let storage = setup_account_storage(path).await?;
        storage
            .is_mnemonic_stored()
            .await
            .map_err(|err| VpnError::InternalError {
                details: err.to_string(),
            })
    }

    pub(crate) async fn get_account_id_raw(path: &str) -> Result<String, VpnError> {
        let storage = setup_account_storage(path).await?;
        storage
            .load_mnemonic()
            .await
            .map(VpnApiAccount::from)
            .map(|account| account.id())
            .map_err(|_err| VpnError::NoAccountStored)
    }

    async fn remove_account_mnemonic_raw(path: &str) -> Result<bool, VpnError> {
        let storage = setup_account_storage(path).await?;
        storage
            .remove_mnemonic()
            .await
            .map(|_| true)
            .map_err(|err| VpnError::InternalError {
                details: err.to_string(),
            })
    }

    async fn remove_credential_storage_raw<P: AsRef<Path>>(path: P) -> Result<(), VpnError> {
        let storage_paths =
            StoragePaths::new_from_dir(&path).map_err(|err| VpnError::InternalError {
                details: err.to_string(),
            })?;

        std::fs::remove_file(storage_paths.credential_database_path).map_err(|err| {
            VpnError::InternalError {
                details: err.to_string(),
            }
        })
    }

    async fn create_vpn_api_client() -> Result<VpnApiClient, VpnError> {
        let network_env = environment::current_environment_details().await?;
        let user_agent = crate::util::construct_user_agent();
        VpnApiClient::new(network_env.vpn_api_url(), user_agent).map_err(|e| {
            VpnError::InternalError {
                details: e.to_string(),
            }
        })
    }

    async fn unregister_device_from_api_raw(path: &str) -> Result<(), VpnError> {
        let account_storage = setup_account_storage(path).await?;
        let device_keys = account_storage
            .load_keys()
            .await
            .map_err(|_| VpnError::NoDeviceIdentity)?;
        let device = Device::from(device_keys.device_keypair().clone());
        let mnemonic = account_storage
            .load_mnemonic()
            .await
            .map_err(|_| VpnError::NoAccountStored)?;
        let account = VpnApiAccount::from(mnemonic);

        let vpn_api_client = create_vpn_api_client().await?;

        vpn_api_client
            .update_device(&account, &device, DeviceStatus::DeleteMe)
            .await
            .map_err(|err| VpnError::UnregisterDeviceApiClientFailure {
                details: err.to_string(),
            })?;
        Ok(())
    }

    pub(crate) async fn forget_account_raw(path: &str) -> Result<(), VpnError> {
        tracing::info!("REMOVING ALL ACCOUNT AND DEVICE DATA IN: {path}");

        let path_buf =
            PathBuf::from_str(path).map_err(|err| VpnError::InvalidAccountStoragePath {
                details: err.to_string(),
            })?;

        unregister_device_from_api_raw(path).await?;

        // First remove the files we own directly
        remove_account_mnemonic_raw(path).await?;
        remove_device_identity_raw(path).await?;
        remove_credential_storage_raw(&path_buf).await?;

        // Then remove the rest of the files, that we own indirectly
        nym_vpn_account_controller::util::remove_files_for_account(&path_buf).map_err(|err| {
            VpnError::InternalError {
                details: err.to_string(),
            }
        })?;

        Ok(())
    }

    pub(crate) async fn get_device_id_raw(path: &str) -> Result<String, VpnError> {
        let storage = setup_account_storage(path).await?;
        storage
            .load_keys()
            .await
            .map(|keys| keys.device_keypair().public_key().to_string())
            .map_err(|_err| VpnError::NoDeviceIdentity)
    }

    pub(crate) async fn remove_device_identity_raw(path: &str) -> Result<(), VpnError> {
        let storage = setup_account_storage(path).await?;
        storage
            .remove_keys()
            .await
            .map_err(|err| VpnError::InternalError {
                details: err.to_string(),
            })
    }
}
