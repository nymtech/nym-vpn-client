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
        .map_err(|err| VpnError::SyncAccount {
            details: err.into(),
        })
}

pub(super) async fn wait_for_update_device() -> Result<DeviceState, VpnError> {
    get_command_sender()
        .await?
        .ensure_update_device()
        .await
        .map_err(|err| VpnError::SyncDevice {
            details: err.into(),
        })
}

pub(super) async fn wait_for_register_device() -> Result<(), VpnError> {
    get_command_sender()
        .await?
        .ensure_register_device()
        .await
        .map_err(|err| VpnError::RegisterDevice {
            details: err.into(),
        })
}

pub(super) async fn wait_for_available_zk_nyms() -> Result<(), VpnError> {
    get_command_sender()
        .await?
        .ensure_available_zk_nyms()
        .await
        .map_err(|err| VpnError::RequestZkNym {
            details: err.into(),
        })
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
        .map_err(|err| VpnError::SyncAccount {
            details: err.into(),
        })
        .map(|_| ())
}

async fn parse_mnemonic(mnemonic: &str) -> Result<Mnemonic, VpnError> {
    Mnemonic::parse(mnemonic).map_err(|err| VpnError::InvalidMnemonic {
        details: err.to_string(),
    })
}

pub(super) async fn login(mnemonic: &str) -> Result<(), VpnError> {
    let mnemonic = parse_mnemonic(mnemonic).await?;
    get_command_sender().await?.login(mnemonic).await?;
    Ok(())
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

// Raw API that directly accesses storage without going through the account controller.
// This API places the responsibility of ensuring the account controller is not running on
// the caller.
//
// WARN: This API was added mostly as a workaround for unblocking the iOS client, and is not a
// sustainable long term solution.
pub(crate) mod raw {
    use std::path::Path;

    use nym_sdk::mixnet::StoragePaths;
    use nym_vpn_api_client::{
        response::NymVpnAccountResponse,
        types::{Device, DeviceStatus},
        VpnApiClient,
    };

    use crate::{platform::environment, storage::VpnClientOnDiskStorage};

    use super::*;

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

    pub(crate) async fn is_account_mnemonic_stored_raw(path: &str) -> Result<bool, VpnError> {
        let storage = setup_account_storage(path).await?;
        storage.is_mnemonic_stored().await.map_err(Into::into)
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
            .map_err(Into::into)
    }

    async fn remove_credential_storage_raw<P: AsRef<Path>>(path: P) -> Result<(), VpnError> {
        let storage_paths = StoragePaths::new_from_dir(&path).map_err(VpnError::internal)?;
        for path in storage_paths.credential_database_paths() {
            tracing::info!("Removing file: {}", path.display());
            match std::fs::remove_file(&path) {
                Ok(_) => tracing::trace!("Removed file: {}", path.display()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    tracing::trace!("File not found, skipping: {}", path.display())
                }
                Err(e) => {
                    tracing::error!("Failed to remove file {}: {e}", path.display());
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
        let user_agent = crate::util::construct_user_agent();
        let vpn_api_client =
            VpnApiClient::new(network_env.vpn_api_url(), user_agent).map_err(VpnError::internal)?;
        Ok(vpn_api_client)
    }

    async fn load_device(path: &str) -> Result<Device, VpnError> {
        let account_storage = setup_account_storage(path).await?;
        account_storage
            .load_keys()
            .await
            .map_err(|_| VpnError::NoDeviceIdentity)
            .map(|d| Device::from(d.device_keypair().clone()))
    }

    async fn unregister_device_raw(path: &str) -> Result<(), VpnError> {
        let account_storage = setup_account_storage(path).await?;
        let device = load_device(path).await?;
        let mnemonic = account_storage
            .load_mnemonic()
            .await
            .map_err(|_| VpnError::NoAccountStored)?;
        let account = VpnApiAccount::from(mnemonic);

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
        let account = VpnApiAccount::from(mnemonic);
        vpn_api_client
            .get_account(&account)
            .await
            .map_err(|_err| VpnError::AccountNotRegistered)
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
        nym_vpn_account_controller::storage_cleanup::remove_files_for_account(&path_buf).map_err(
            |err| VpnError::Storage {
                details: err.to_string(),
            },
        )?;

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
        storage.remove_keys().await.map_err(VpnError::internal)
    }
}
