// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Error, Result};
use std::sync::Arc;

use crate::{SharedAccountState, commands::ReturnSender, storage::AccountStorageOp};
use nym_common::trace_err_chain;
use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    VpnApiClient,
    error::UNREGISTER_NON_EXISTENT_DEVICE_CODE_ID,
    response::extract_error_response,
    types::{Device, DeviceStatus, Platform, VpnAccount},
};
use nym_vpn_lib_types::{
    QrPairingConfirmation, QrUrl, RegisterAccountResponse, StorableAccount, StoredAccountMode,
};
use nym_vpn_store::{
    device_state::{StoredAccountIdentity, StoredDeviceState},
    keys::wireguard::WireguardKeyStore,
};

// The onus of making sure the conditions are right to call these handlers is on the caller

async fn ensure_account_exists_on_chain<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    account: &VpnAccount,
) -> Result<()> {
    // if we're attempting to store a decentralised account, it MUST exist on chain,
    // i.e. it must have proper number and sequence
    let Some(account_response) = shared_state
        .nyxd_client
        .get_account_details(&account.get_mnemonic())
        .await?
    else {
        return Err(Error::AccountDoesntExistOnChain);
    };

    let Ok(base_account) = account_response.try_get_base_account() else {
        return Err(Error::AccountDoesntExistOnChain);
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
) -> Result<VpnAccount> {
    let account_identity = StoredAccountIdentity::new(account.mnemonic.clone(), account.mode)
        .map_err(|e| Error::InvalidMnemonic(e.into()))?;
    let vpn_account =
        VpnAccount::try_from(account.clone()).map_err(|e| Error::InvalidMnemonic(e.into()))?;
    let ecash_keypair = vpn_account
        .create_ecash_keypair()
        .map_err(|err| Error::CreateEcashKeyPair(err.into()))?;

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
        .map_err(Error::internal)?;
    let device = rx.await.map_err(Error::internal)??;

    shared_state.device_state = Some(StoredDeviceState {
        account_identity,
        ecash_keypair,
        summary: None,
    });
    shared_state.device = Some(device);

    tracing::debug!("Account stored!");
    Ok(vpn_account)
}

pub(crate) async fn handle_create_account<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<()> {
    let (vpn_account, mnemonic) = VpnAccount::generate_new().map_err(Error::internal)?;
    let ecash_keypair = vpn_account
        .create_ecash_keypair()
        .map_err(Error::internal)?;
    let account_identity = StoredAccountIdentity::new(
        mnemonic.clone(),
        StoredAccountMode::from(vpn_account.mode()),
    )
    .map_err(Error::internal)?;

    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::StoreAccount(
            tx,
            StorableAccount::new(mnemonic.clone(), vpn_account.mode().into()),
        ))
        .map_err(Error::internal)?;
    let device = rx.await.map_err(Error::internal)??;

    shared_state.device_state = Some(StoredDeviceState {
        account_identity,
        ecash_keypair,
        summary: None,
    });
    shared_state.device = Some(device);
    tracing::debug!("Account created and stored");

    Ok(())
}

pub(crate) async fn handle_register_account<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    account: StorableAccount,
    platform: Platform,
) -> Result<RegisterAccountResponse> {
    let vpn_account =
        VpnAccount::try_from(account.clone()).map_err(|e| Error::InvalidMnemonic(e.into()))?;
    let account_token = shared_state
        .vpn_api_client
        .register_account(&vpn_account, platform)
        .await?
        .account_token;

    tracing::debug!("Account registered with API");

    Ok(RegisterAccountResponse { account_token })
}

pub(crate) async fn handle_redeem_freepass<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    account: StorableAccount,
    code: String,
) -> Result<()> {
    let vpn_account =
        VpnAccount::try_from(account).map_err(|e| Error::InvalidMnemonic(e.into()))?;
    shared_state
        .vpn_api_client
        .redeem_freepass(&vpn_account, code)
        .await?;

    tracing::debug!("Free pass redeemed, account registered with API");

    Ok(())
}

pub(crate) async fn handle_forget_account<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<()> {
    tracing::info!("REMOVING ACCOUNT AND ALL ASSOCIATED DATA");

    // Tunnel state is checked before sending the command here. We're in Disconnected state

    if let Err(err) = handle_unregister_device(shared_state).await {
        tracing::error!("Failed to unregister device: {err}");
    } else {
        tracing::info!("Device has been unregistered");
    }

    clear_local_account_data(shared_state).await
}

pub(crate) async fn handle_delete_account<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<()> {
    tracing::info!("DELETING ACCOUNT AND ALL ASSOCIATED DATA");

    // Tunnel state is checked before sending the command here. We're in Disconnected state

    let account = shared_state.load_vpn_api_account().await?;

    match shared_state.vpn_api_client.delete_account(&account).await {
        Ok(_account_response) => tracing::info!("Account has been deleted"),
        // An account that was never registered only exists on this device
        Err(err) if err.is_account_not_found() => {
            tracing::info!("Account is not registered with the API, nothing to delete there");
        }
        Err(err) => {
            tracing::error!("Failed to delete account: {err:?}");
            return Err(err.into());
        }
    }

    // A deleted account can't unregister its device, so only local data is removed
    clear_local_account_data(shared_state).await.or_else(|err| {
        // Once the stored account is cleared, only leftover files remain: the deletion itself succeeded
        if shared_state.device_state.is_none() {
            tracing::warn!("Account deleted, but local cleanup is incomplete: {err}");
            Ok(())
        } else {
            Err(err)
        }
    })
}

async fn clear_local_account_data<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<()> {
    shared_state
        .wireguard_keys_storage
        .clear_keys()
        .await
        .map_err(|source| {
            tracing::error!("Failed to clear wireguard keys storage: {source:?}");
            Error::Storage(source.into())
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
        .map_err(Error::storage)?;

    // Removing mnemonic and keys in storage
    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::ForgetAccount(tx))
        .map_err(Error::internal)?;

    rx.await
        .map_err(Error::internal)? // Handling channel error
        .map_err(|source| {
            tracing::error!("Failed to remove account: {source:?}");
            Error::Storage(source.into())
        })?; // Handling account removal error

    // Once we have removed or reset all storage, we need to reset the account state
    shared_state.device = None;
    shared_state.device_state = None;
    shared_state.nyxd_client.disconnect();

    if let Err(err) = remove_files_result {
        return Err(Error::Storage(
            format!("Failed to remove files for account: {err}").into(),
        ));
    }

    Ok(())
}

pub(crate) async fn handle_link_account<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    privy_account: StorableAccount,
) -> Result<()> {
    let privy_vpn_account =
        VpnAccount::try_from(privy_account).map_err(|e| Error::InvalidMnemonic(e.into()))?;

    // We can only link the Privy account if we're currently logged-in with an API account
    if privy_vpn_account.mode().is_privy()
        && let Some(ref current_state) = shared_state.device_state
        && current_state.account_identity.mode == StoredAccountMode::Api
    {
        tracing::info!("Linking Privy account with API account");

        let current_account = shared_state.load_vpn_api_account().await?;

        let _status_ok = shared_state
            .vpn_api_client
            .link_account(&current_account, &privy_vpn_account, "Social login")
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to link Privy account with API account: {err:?}")
            })?;

        tracing::info!("Successfully linked Privy account with API account");

        Ok(())
    } else {
        tracing::error!("Cannot link Privy account when not logged-in with an API account");
        Err(Error::NoAccountStored)
    }
}

pub(crate) async fn handle_qr_pair_account<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<QrUrl> {
    // We can only pair a new device to the account if we're currently logged-in with an API account
    if is_logged_in_with_api_account(shared_state) {
        let account = Arc::new(shared_state.load_vpn_api_account().await?);
        shared_state
            .qr_pairing_handler
            .pair_account(
                &account,
                shared_state.event_sender.clone(),
                shared_state.cancel_token.child_token(),
            )
            .await
    } else {
        tracing::error!("Cannot pair another device when not logged-in with an API account");
        Err(Error::NoAccountStored)
    }
}

pub(crate) async fn handle_qr_scanned_key<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    remote_qr_pub_key_str: &str,
) -> Result<QrPairingConfirmation> {
    // We can only pair a new device to the account if we're currently logged-out
    if shared_state.device_state.is_none() {
        shared_state
            .qr_pairing_handler
            .scanned_key(
                remote_qr_pub_key_str,
                shared_state.event_sender.clone(),
                shared_state.cancel_token.child_token(),
            )
            .await
    } else {
        tracing::error!(
            "Cannot pair to another account when already logged-in with an API account"
        );
        Err(Error::ExistingAccount)
    }
}

pub(crate) async fn handle_qr_verified_pairing_pin<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    session_id: u32,
) -> Result<()> {
    // We can only verify PIN if we're currently logged-in with an API account
    if is_logged_in_with_api_account(shared_state) {
        let account = Arc::new(shared_state.load_vpn_api_account().await?);
        shared_state
            .qr_pairing_handler
            .verified_pairing_pin(&account, session_id)
            .await
    } else {
        tracing::error!("Cannot verify a pairing PIN when not logged-in with an API account");
        Err(Error::NoAccountStored)
    }
}

pub(crate) fn handle_qr_cancel_pairing_process<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<()> {
    // Cancelling is local and always allowed, so the UI can close the pairing window at any time
    shared_state.qr_pairing_handler.cancel_pairing_process();
    Ok(())
}

pub(crate) async fn handle_rotate_keys<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<()> {
    shared_state
        .wireguard_keys_storage
        .clear_keys()
        .await
        .map_err(|source| {
            tracing::error!("Failed to clear wireguard keys storage: {source:?}");
            Error::Storage(source.into())
        })
}

// TODO: With the erorr rework we are now losing information about the original error cause, this needs to be addressed, but errors rework is a pretty gnarly thing to do, so we need to plan it a bit more carefully
pub(crate) async fn handle_unregister_device<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<()> {
    tracing::info!("Unregistering device from API");

    let device = shared_state.device.as_ref().ok_or(Error::NoDeviceStored)?;

    let account = shared_state
        .device_state
        .as_ref()
        .map(|state| &state.account_identity)
        .ok_or(Error::NoAccountStored)?;

    shared_state
        .vpn_api_client
        .update_device(&account.id, device, DeviceStatus::DeleteMe)
        .await
        .map(|_| ())
        .or_else(|err| {
            let device_never_existed = extract_error_response(&err)
                .and_then(|response| response.code_reference_id)
                .is_some_and(|code| code == UNREGISTER_NON_EXISTENT_DEVICE_CODE_ID);

            if device_never_existed {
                Ok(())
            } else {
                Err(err.into())
            }
        })
}

/// Registers the device with the account unless the API already knows it.
///
/// The lookup is signed by the device, so step-up authentication is only triggered when the
/// device actually needs to be registered.
pub(crate) async fn handle_register_device<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<()> {
    let device = shared_state.device.as_ref().ok_or(Error::NoDeviceStored)?;
    let account = shared_state
        .device_state
        .as_ref()
        .map(|state| &state.account_identity)
        .ok_or(Error::NoAccountStored)?;

    if is_device_registered(&shared_state.vpn_api_client, &account.id, device).await? {
        tracing::debug!("Device is already registered");
    } else {
        tracing::info!("Registering device with the account");
        let vpn_account = shared_state.load_vpn_api_account().await?;
        shared_state
            .vpn_api_client
            .register_device(&vpn_account, device)
            .await
            .inspect_err(|err| {
                trace_err_chain!(err, "failed to register device");
            })?;
    }
    Ok(())
}

/// Registers the device right after log-in, while the account is still in memory.
///
/// Failures are only logged, so that log-in succeeds regardless.
pub(crate) async fn handle_register_device_on_login<C: ConnectivityMonitor>(
    shared_state: &SharedAccountState<C>,
    vpn_account: &VpnAccount,
) {
    let Some(device) = shared_state
        .device
        .as_ref()
        .filter(|_| !vpn_account.mode().is_decentralised() && !shared_state.firewall_active)
    else {
        return;
    };

    // Registering an already registered device succeeds, so there is no need to check first.
    tracing::info!("Registering device with the account");
    if let Err(err) = shared_state
        .vpn_api_client
        .register_device(vpn_account, device)
        .await
    {
        trace_err_chain!(err, "failed to register device during log-in");
    }
}

async fn is_device_registered(
    vpn_api_client: &VpnApiClient,
    account_id: &str,
    device: &Device,
) -> Result<bool> {
    match vpn_api_client.get_device_by_id(account_id, device).await {
        Ok(_) => Ok(true),
        Err(err) if extract_error_response(&err).is_some_and(|r| r.is_device_not_found()) => {
            Ok(false)
        }
        Err(err) => Err(err.into()),
    }
}

pub(crate) async fn handle_reset_device_identity<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    seed: Option<[u8; 32]>,
) -> Result<()> {
    tracing::info!("Resetting device key");

    // Reset keys in storage
    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::ResetKeys(tx, seed))
        .map_err(Error::internal)?;
    let device = rx.await.map_err(Error::internal)??;

    // summary is now stale
    if let Err(err) = shared_state.mark_summary_as_stale().await {
        trace_err_chain!(err, "failed to mark account summary as stale");
    }

    shared_state.device = Some(device);

    // The device identity changed, so any installed credential fetcher is bound to a stale device.
    // Drop it; the next time we reach a state that needs one it will be rebuilt with the new device.
    shared_state.clear_credential_fetcher().await?;
    shared_state.qr_pairing_handler.cancel_pairing_check();

    Ok(())
}

fn is_logged_in_with_api_account<C: ConnectivityMonitor>(
    shared_state: &SharedAccountState<C>,
) -> bool {
    shared_state
        .device_state
        .as_ref()
        .is_some_and(|state| state.account_identity.mode == StoredAccountMode::Api)
}
