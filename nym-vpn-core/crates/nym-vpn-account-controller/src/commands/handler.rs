// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::types::{DeviceStatus, VpnApiAccount};
use nym_vpn_lib_types::{AccountCommandError, VpnApiError};
use nym_vpn_store::mnemonic::Mnemonic;

use crate::{SharedAccountState, commands::ReturnSender, storage::AccountStorageOp};

// The onus of making sure the conditions are right to call these handlers is on the caller

pub(crate) async fn handle_store_account(
    shared_state: &mut SharedAccountState,
    mnemonic: Mnemonic,
    offline_mode: bool,
) -> Result<(), AccountCommandError> {
    let vpn_account = VpnApiAccount::try_from(mnemonic.clone())
        .map_err(|e| AccountCommandError::InvalidMnemonic(e.to_string()))?;

    // SW This looks so bad tbh
    if !offline_mode {
        shared_state
            .vpn_api_client
            .check_account_exists_on_api(&vpn_account)
            .await?;
    } else {
        tracing::warn!("Not checking if account exists on vpn-api as we are offline");
    }

    // at this point, we know the mnemonic is valid
    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::StoreAccount(tx, mnemonic.clone()))
        .map_err(AccountCommandError::internal)?;
    let device = rx
        .await
        .map_err(AccountCommandError::internal)? // Channel error
        .map_err(AccountCommandError::storage)?; // Storage error
    // SW better error handling

    shared_state.vpn_api_account = Some(vpn_account);
    shared_state.device = Some(device);

    tracing::debug!("Account stored!");
    Ok(())
}

pub(crate) async fn handle_create_account(
    shared_state: &mut SharedAccountState,
) -> Result<(), AccountCommandError> {
    let (vpn_account, mnemonic) = VpnApiAccount::random().map_err(AccountCommandError::internal)?;

    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::StoreAccount(tx, mnemonic.clone()))
        .map_err(AccountCommandError::internal)?;
    let device = rx
        .await
        .map_err(AccountCommandError::internal)? // Channel error
        .map_err(AccountCommandError::storage)?; // Storage error

    // SW better error handling

    shared_state.vpn_api_account = Some(vpn_account);
    shared_state.device = Some(device);
    tracing::debug!("Account created and stored");

    Ok(())
}

pub(crate) async fn handle_forget_account(
    shared_state: &mut SharedAccountState,
) -> Result<(), AccountCommandError> {
    tracing::info!("REMOVING ACCOUNT AND ALL ASSOCIATED DATA");

    // Tunnel state is checked before sending the command here. We're in Disconnected state

    if let Err(err) = handle_unregister_device(shared_state).await {
        tracing::error!("Failed to unregister device: {err}");
    } else {
        tracing::info!("Device has been unregistered");
    }

    shared_state
        .credential_storage
        .reset()
        .await
        .map_err(|source| {
            tracing::error!("Failed to reset credential storage: {source:?}");
            AccountCommandError::ResetCredentialStorage(source.to_string())
        })?;

    // Purge all files in the data directory that we are not explicitly deleting through it's
    // owner. Ideally we should strive for this to be removed.
    // If this fails, we still need to continue with the remaining steps
    let remove_files_result =
        crate::storage::remove_files_for_account(&shared_state.config.data_dir)
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to remove files for account: {err:?}");
            });

    // Removing mnemonic and keys in storage
    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::ForgetAccount(tx))
        .map_err(AccountCommandError::internal)?;

    rx.await
        .map_err(AccountCommandError::internal)? // Handling channel error
        .map_err(|source| {
            tracing::error!("Failed to remove account: {source:?}");
            AccountCommandError::RemoveAccount(source.to_string())
        })?; // Handling account removal error

    // Once we have removed or reset all storage, we need to reset the account state
    shared_state.vpn_api_account = None;
    shared_state.device = None;

    if let Err(err) = remove_files_result {
        return Err(AccountCommandError::RemoveAccountFiles(format!(
            "Failed to remove files for account: {err}"
        )));
    }

    Ok(())
}

pub(crate) async fn handle_unregister_device(
    shared_state: &mut SharedAccountState,
) -> Result<(), AccountCommandError> {
    tracing::info!("Unregistering device from API");

    let device = shared_state
        .device
        .as_ref()
        .ok_or(AccountCommandError::NoDeviceStored)?;

    let account = shared_state
        .vpn_api_account
        .as_ref()
        .ok_or(AccountCommandError::NoAccountStored)?;

    // SW better error handling if the device didn't exist in the first place
    shared_state
        .vpn_api_client
        .update_device(account, device, DeviceStatus::DeleteMe)
        .await
        .map_err(|err| {
            VpnApiError::try_from(err)
                .map(AccountCommandError::UpdateDeviceErrorResponse)
                .unwrap_or_else(AccountCommandError::unexpected_response)
        })?;
    Ok(())
}

pub(crate) async fn handle_reset_device_identity(
    shared_state: &mut SharedAccountState,
    seed: Option<[u8; 32]>,
) -> Result<(), AccountCommandError> {
    tracing::info!("Resetting device key");

    // Reset keys in storage
    // SW Better error handling
    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::ResetKeys(tx, seed))
        .map_err(AccountCommandError::internal)?;
    let device = rx
        .await
        .map_err(AccountCommandError::internal)? // Channel error
        .map_err(AccountCommandError::storage)?; // Storage error

    shared_state.device = Some(device);

    Ok(())
}
