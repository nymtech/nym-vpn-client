// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    response::{NymVpnDevice, NymVpnUsage},
    types::VpnAccountMode,
};
use nym_vpn_lib_types::{
    AccountCommandError, AutologinResponse, DeeplinkKind, StorableAccount, StoredAccountMode,
    VpnAccountSummary,
};

use crate::{
    SharedAccountState,
    commands::{ReturnSender, dispatch::CommonCommand},
    deeplink::{CreateDeeplinkParams, DeeplinkMnemonic},
    storage::AccountStorageOp,
};

pub(crate) async fn handle_common_command<C: ConnectivityMonitor>(
    command: CommonCommand,
    shared_state: &mut SharedAccountState<C>,
) {
    match command {
        CommonCommand::GetStoredAccount(result_tx) => {
            result_tx.send(handle_get_stored_account(shared_state).await);
        }
        CommonCommand::GetAccountIdentity(result_tx) => {
            result_tx.send(handle_get_account_identity(shared_state));
        }
        CommonCommand::GetCanonicalAccountIdentity(result_tx) => {
            result_tx.send(handle_get_canonical_account_identity(shared_state).await);
        }
        CommonCommand::GetAccountMode(result_tx) => {
            result_tx.send(handle_get_account_mode(shared_state));
        }
        CommonCommand::GetDeviceIdentity(result_tx) => {
            result_tx.send(handle_get_device_identity(shared_state));
        }
        CommonCommand::GetUsage(result_tx) => {
            result_tx.send(handle_get_usage(shared_state).await);
        }
        CommonCommand::GetDevices(result_tx) => {
            result_tx.send(handle_get_devices(shared_state).await);
        }
        CommonCommand::GetActiveDevices(result_tx) => {
            result_tx.send(handle_get_active_devices(shared_state).await);
        }
        CommonCommand::GetAccountSummary(result_tx) => {
            result_tx.send(handle_get_account_summary(shared_state).await);
        }
        CommonCommand::GetDeeplink(result_tx, params) => {
            result_tx.send(handle_get_deeplink(shared_state, params).await)
        }
        CommonCommand::GetAutologinDeeplink(result_tx, params) => {
            result_tx.send(handle_get_autologin_deeplink(shared_state, params).await);
        }
        CommonCommand::DeriveDeeplinkMnemonic(result_tx, deeplink_callback_url) => result_tx
            .send(handle_derive_deeplink_mnemonic(shared_state, deeplink_callback_url).await),
    };
}

// This goes into storage each time, to trigger platform's unlocking mechanism if secure storage is used
pub(crate) async fn handle_get_stored_account<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Option<StorableAccount>, AccountCommandError> {
    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::GetStoredAccount(tx))
        .map_err(AccountCommandError::internal)?;
    rx.await
        .map_err(AccountCommandError::internal)? // Channel error
        .map_err(AccountCommandError::storage) // Storage error
}

pub(crate) fn handle_get_account_identity<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Option<String>, AccountCommandError> {
    Ok(shared_state
        .vpn_api_account
        .as_ref()
        .map(|account| account.id()))
}

pub(crate) async fn handle_get_canonical_account_identity<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Option<String>, AccountCommandError> {
    let Some(account) = shared_state.vpn_api_account.as_ref() else {
        return Err(AccountCommandError::NoAccountStored);
    };

    match account.mode() {
        VpnAccountMode::Api | VpnAccountMode::Decentralised => Ok(Some(account.id().to_string())),
        VpnAccountMode::Privy => {
            let response = shared_state
                .vpn_api_client
                .get_canonical_account_identity(account)
                .await?;

            Ok(Some(response.canonical_account_addr))
        }
    }
}

pub(crate) fn handle_get_account_mode<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Option<StoredAccountMode>, AccountCommandError> {
    Ok(shared_state
        .vpn_api_account
        .as_ref()
        .map(|account| account.mode().into()))
}

async fn handle_get_usage<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Vec<NymVpnUsage>, AccountCommandError> {
    let account = shared_state
        .vpn_api_account
        .as_ref()
        .ok_or(AccountCommandError::NoAccountStored)?;

    let usage = shared_state.vpn_api_client.get_usage(account).await?;

    tracing::debug!("Usage: {:#?}", usage);
    Ok(usage.items)
}

pub(crate) fn handle_get_device_identity<C: ConnectivityMonitor>(
    shared_state: &SharedAccountState<C>,
) -> Result<Option<String>, AccountCommandError> {
    let device = shared_state
        .device
        .as_ref()
        .map(|device| device.identity_key().to_string());

    tracing::debug!("Device identity: {device:?}");
    Ok(device)
}

async fn handle_get_devices<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
    tracing::debug!("Getting devices from API");

    let account = shared_state
        .vpn_api_account
        .as_ref()
        .ok_or(AccountCommandError::NoAccountStored)?;

    let devices = shared_state.vpn_api_client.get_devices(account).await?;

    tracing::debug!("The account has the following devices associated to it:");
    // TODO: pagination
    for device in &devices.items {
        tracing::debug!("{:?}", device);
    }
    Ok(devices.items)
}

async fn handle_get_active_devices<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
    tracing::debug!("Getting active devices from API");

    let account = shared_state
        .vpn_api_account
        .as_ref()
        .ok_or(AccountCommandError::NoAccountStored)?;

    let devices = shared_state
        .vpn_api_client
        .get_active_devices(account)
        .await?;

    tracing::debug!("The account has the following active devices associated to it:");
    // TODO: pagination
    for device in &devices.items {
        tracing::debug!("{:?}", device);
    }
    Ok(devices.items)
}

pub(crate) async fn handle_get_account_summary<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Option<VpnAccountSummary>, AccountCommandError> {
    Ok(shared_state.vpn_account_summary.clone())
}

pub(crate) async fn handle_get_deeplink<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    params: CreateDeeplinkParams,
) -> Result<String, AccountCommandError> {
    // For `DeeplinkKind::PrivyLink`, the user must be logged-in via an API account
    if params.kind == DeeplinkKind::PrivyLink
        && shared_state
            .vpn_api_account
            .as_ref()
            .map(|vpn_account| !vpn_account.mode().is_api())
            .unwrap_or(true)
    {
        return Err(AccountCommandError::DeeplinkError(
            "You can only link a Privy account if you are logged in with an API account"
                .to_string(),
        ));
    }

    // Create a new Deeplink for this request
    let deeplink = shared_state
        .deeplinks
        .create_deeplink(&params)
        .map_err(|e| AccountCommandError::DeeplinkError(e.to_string()))?;

    // Create the deeplink URL
    let url = deeplink.create_url(&params.base_url);

    // Housekeeping
    shared_state.deeplinks.remove_expired();

    Ok(url.to_string())
}

pub(crate) async fn handle_get_autologin_deeplink<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    params: CreateDeeplinkParams,
) -> Result<AutologinResponse, AccountCommandError> {
    let account = shared_state
        .vpn_api_account
        .as_ref()
        .ok_or(AccountCommandError::NoAccountStored)?;
    let mnemonic = account.get_mnemonic();

    let deeplink = shared_state
        .deeplinks
        .create_deeplink(&params)
        .map_err(|e| AccountCommandError::DeeplinkError(e.to_string()))?;

    let autologin = deeplink
        .create_autologin_url(&params.base_url, mnemonic.to_string())
        .map_err(|e| AccountCommandError::DeeplinkError(e.to_string()))?;

    shared_state.deeplinks.remove_expired();

    Ok(autologin)
}

pub(crate) async fn handle_derive_deeplink_mnemonic<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    deeplink_callback_url: String,
) -> Result<DeeplinkMnemonic, AccountCommandError> {
    // Derive the mnemonic from the provided deeplink URL
    let deeplink_mnemonic = shared_state
        .deeplinks
        .derive_mnemonic(&deeplink_callback_url)
        .map_err(|e| AccountCommandError::DeeplinkError(e.to_string()))?;

    // Housekeeping
    shared_state.deeplinks.remove_expired();

    Ok(deeplink_mnemonic)
}
