// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use crate::{SharedAccountState, commands::ReturnSender, storage::AccountStorageOp};
use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    error::UNREGISTER_NON_EXISTENT_DEVICE_CODE_ID,
    response::NymErrorResponse,
    types::{DeviceStatus, Platform, VpnAccount},
};
use nym_vpn_lib_types::{
    AccountCommandError, RegisterAccountResponse, StorableAccount, VpnApiError,
};
use nym_vpn_store::keys::wireguard::WireguardKeyStore;

// The onus of making sure the conditions are right to call these handlers is on the caller

async fn ensure_account_exists_on_chain<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    account: &VpnAccount,
) -> Result<(), AccountCommandError> {
    // if we're attempting to store a decentralised account, it MUST exist on chain,
    // i.e. it must have proper number and sequence
    let Some(account_response) = shared_state
        .nyxd_client
        .get_account_details(&account.get_mnemonic())
        .await?
    else {
        return Err(AccountCommandError::AccountDoesntExistOnChain);
    };

    let Ok(base_account) = account_response.try_get_base_account() else {
        return Err(AccountCommandError::AccountDoesntExistOnChain);
    };
    tracing::info!(
        "importing decentralised account '{}' with account number: {} and sequence: {}",
        base_account.address,
        base_account.account_number,
        base_account.sequence
    );

    Ok(())
}

pub(crate) async fn handle_store_account<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    account: StorableAccount,
) -> Result<(), AccountCommandError> {
    let vpn_account = VpnAccount::try_from(account.clone())
        .map_err(|e| AccountCommandError::InvalidMnemonic(e.to_string()))?;

    // if the account is decentralised, it must exist on the chain
    if vpn_account.mode().is_decentralised() {
        ensure_account_exists_on_chain(shared_state, &vpn_account).await?;
    }

    // We don't check the account status here. The check was bypassed when offline anyway. We defer that job to the syncing state

    // at this point, we know the mnemonic is valid
    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::StoreAccount(tx, account))
        .map_err(AccountCommandError::internal)?;
    let device = rx
        .await
        .map_err(AccountCommandError::internal)? // Channel error
        .map_err(AccountCommandError::storage)?; // Storage error

    shared_state.vpn_api_account = Some(Arc::new(vpn_account));
    shared_state.device = Some(device);

    tracing::debug!("Account stored!");
    Ok(())
}

pub(crate) async fn handle_create_account<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<(), AccountCommandError> {
    let (vpn_account, mnemonic) =
        VpnAccount::generate_new().map_err(AccountCommandError::internal)?;

    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::StoreAccount(
            tx,
            StorableAccount::new(mnemonic, vpn_account.mode().into()),
        ))
        .map_err(AccountCommandError::internal)?;
    let device = rx
        .await
        .map_err(AccountCommandError::internal)? // Channel error
        .map_err(AccountCommandError::storage)?; // Storage error

    shared_state.vpn_api_account = Some(Arc::new(vpn_account));
    shared_state.device = Some(device);
    tracing::debug!("Account created and stored");

    Ok(())
}

pub(crate) async fn handle_register_account<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    account: StorableAccount,
    platform: Platform,
) -> Result<RegisterAccountResponse, AccountCommandError> {
    let vpn_account = VpnAccount::try_from(account.clone())
        .map_err(|e| AccountCommandError::InvalidMnemonic(e.to_string()))?;
    let account_token = shared_state
        .vpn_api_client
        .register_account(&vpn_account, platform)
        .await?
        .account_token;

    tracing::debug!("Account registered with API");

    Ok(RegisterAccountResponse { account_token })
}

pub(crate) async fn handle_forget_account<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<(), AccountCommandError> {
    tracing::info!("REMOVING ACCOUNT AND ALL ASSOCIATED DATA");

    if let Err(err) = handle_unregister_device(shared_state).await {
        tracing::error!("Failed to unregister device: {err}");
    } else {
        tracing::info!("Device has been unregistered");
    }

    shared_state
        .wireguard_keys_storage
        .clear_keys()
        .await
        .map_err(|source| {
            tracing::error!("Failed to clear wireguard keys storage: {source:?}");
            AccountCommandError::Storage(source.to_string())
        })?;

    // Purge all files in the data directory that we are not explicitly deleting through it's
    // owner. Ideally we should strive for this to be removed.
    // If this fails, we still need to continue with the remaining steps
    let remove_files_result =
        crate::storage::remove_files_for_account(&shared_state.config.data_dir, false)
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to remove files for account: {err:?}");
            });

    shared_state
        .bandwidth_control_command_tx
        .reset()
        .await
        .map_err(AccountCommandError::storage)?;

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
            AccountCommandError::Storage(source.to_string())
        })?; // Handling account removal error

    // Removing account summary
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::RemoveAccountSummary)
        .map_err(AccountCommandError::internal)?;

    // Once we have removed or reset all storage, we need to reset the account state
    shared_state.vpn_api_account = None;
    shared_state.device = None;
    shared_state.vpn_account_summary = None;
    shared_state.nyxd_client.disconnect();

    if let Err(err) = remove_files_result {
        return Err(AccountCommandError::Storage(format!(
            "Failed to remove files for account: {err}"
        )));
    }

    Ok(())
}

pub(crate) async fn handle_link_account<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    privy_account: StorableAccount,
) -> Result<(), AccountCommandError> {
    let privy_vpn_account = VpnAccount::try_from(privy_account)
        .map_err(|e| AccountCommandError::InvalidMnemonic(e.to_string()))?;

    // We can only link the Privy account if we're currently logged-in with an API account
    if privy_vpn_account.mode().is_privy()
        && let Some(ref current_account) = shared_state.vpn_api_account
        && current_account.mode().is_api()
    {
        tracing::info!("Linking Privy account with API account");

        let _status_ok = shared_state
            .vpn_api_client
            .link_account(current_account, &privy_vpn_account, "Social login")
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to link Privy account with API account: {err:?}")
            })?;

        tracing::info!("Successfully linked Privy account with API account");

        Ok(())
    } else {
        tracing::error!("Cannot link Privy account when not logged-in with an API account");
        Err(AccountCommandError::NoAccountStored)
    }
}

pub(crate) async fn handle_rotate_keys<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<(), AccountCommandError> {
    shared_state
        .wireguard_keys_storage
        .clear_keys()
        .await
        .map_err(|source| {
            tracing::error!("Failed to clear wireguard keys storage: {source:?}");
            AccountCommandError::Storage(source.to_string())
        })
}

// TODO: With the erorr rework we are now losing information about the original error cause, this needs to be addressed, but errors rework is a pretty gnarly thing to do, so we need to plan it a bit more carefully
pub(crate) async fn handle_unregister_device<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
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

    if let Err(e) = shared_state
        .vpn_api_client
        .update_device(account, device, DeviceStatus::DeleteMe)
        .await
    {
        let nym_error = NymErrorResponse::try_from(e)?;
        if nym_error.code_reference_id == Some(UNREGISTER_NON_EXISTENT_DEVICE_CODE_ID.to_string()) {
            // Device didn't exist in the first place so we're good
            Ok(())
        } else {
            Err(VpnApiError::Response(nym_error.into()))?
        }
    } else {
        Ok(())
    }
}

pub(crate) async fn handle_reset_device_identity<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    seed: Option<[u8; 32]>,
) -> Result<(), AccountCommandError> {
    tracing::info!("Resetting device key");

    // Reset keys in storage
    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::ResetKeys(tx, seed))
        .map_err(AccountCommandError::internal)?;
    let device = rx
        .await
        .map_err(AccountCommandError::internal)? // Channel error
        .map_err(AccountCommandError::storage)?; // Storage error

    // summary is now stale
    shared_state.mark_summary_as_stale();

    shared_state.device = Some(device);

    // The device identity changed, so any installed credential fetcher is bound to a stale device.
    // Drop it; the next time we reach a state that needs one it will be rebuilt with the new device.
    shared_state.clear_credential_fetcher().await?;

    Ok(())
}
