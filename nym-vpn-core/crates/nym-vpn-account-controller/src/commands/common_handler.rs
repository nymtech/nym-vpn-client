// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    response::{NymVpnDevice, NymVpnUsage},
    types::{VpnAccount, VpnAccountMode},
};
use nym_vpn_lib_types::{
    AccountCommandError, AutologinResponse, DeeplinkKind, StorableAccount, StoredAccountMode,
    VpnAccountSummary,
};
use nym_vpn_store::device_state::DeviceState;

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
            result_tx.send(handle_get_account_identity(shared_state).await);
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
        CommonCommand::IsTrialEligible(result_tx) => {
            result_tx.send(handle_is_trial_eligible(shared_state).await);
        }
        CommonCommand::CheckAppleIntroOfferEligibility(result_tx, product_id, transaction_id) => {
            result_tx.send(
                handle_check_apple_intro_offer_eligibility(
                    shared_state,
                    product_id,
                    transaction_id,
                )
                .await,
            );
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

pub(crate) async fn handle_get_account_identity<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Option<String>, AccountCommandError> {
    let (tx, rx) = ReturnSender::new();

    shared_state
        .storage_op_sender
        .send(AccountStorageOp::GetStoredAccount(tx))
        .map_err(AccountCommandError::internal)?;

    let account = rx
        .await
        .map_err(AccountCommandError::internal)? // Channel error
        .map_err(AccountCommandError::storage)?; // Storage error

    match account {
        Some(account) => {
            // FIXME
            let vpn_account =
                VpnAccount::new(account.mnemonic, VpnAccountMode::from(account.mode)).unwrap();

            Ok(Some(vpn_account.id()))
        }
        None => Ok(None),
    }
}

pub(crate) async fn handle_get_canonical_account_identity<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Option<String>, AccountCommandError> {
    let Some(account) = shared_state.device_state.account() else {
        return Err(AccountCommandError::NoAccountStored);
    };

    match account.mode {
        StoredAccountMode::Api | StoredAccountMode::Decentralised => Ok(Some(account.id.clone())),
        StoredAccountMode::Privy => {
            let mnemonic = todo!("obtain mnemonic");
            let vpn_account = VpnAccount::new(mnemonic, VpnAccountMode::from(account.mode))
                .map_err(AccountCommandError::internal)?;
            let response = shared_state
                .vpn_api_client
                .get_canonical_account_identity(&vpn_account)
                .await?;

            Ok(Some(response.canonical_account_addr))
        }
    }
}

pub(crate) fn handle_get_account_mode<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Option<StoredAccountMode>, AccountCommandError> {
    Ok(shared_state
        .device_state
        .account()
        .map(|account| account.mode))
}

async fn handle_get_usage<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Vec<NymVpnUsage>, AccountCommandError> {
    let Some(account) = shared_state.device_state.account() else {
        return Err(AccountCommandError::NoAccountStored);
    };
    let mnemonic = todo!();

    let vpn_account = VpnAccount::new(mnemonic, VpnAccountMode::from(account.mode))
        .map_err(AccountCommandError::internal)?;
    let usage = shared_state.vpn_api_client.get_usage(&vpn_account).await?;

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

    let Some(account) = shared_state.device_state.account() else {
        return Err(AccountCommandError::NoAccountStored);
    };

    let mnemonic = todo!();
    let vpn_account = VpnAccount::new(mnemonic, VpnAccountMode::from(account.mode))
        .map_err(AccountCommandError::internal)?;

    let devices = shared_state
        .vpn_api_client
        .get_devices(&vpn_account)
        .await?;

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

    let Some(account) = shared_state.device_state.account() else {
        return Err(AccountCommandError::NoAccountStored);
    };

    let mnemonic = todo!();
    let vpn_account = VpnAccount::new(mnemonic, VpnAccountMode::from(account.mode))
        .map_err(AccountCommandError::internal)?;

    let devices = shared_state
        .vpn_api_client
        .get_active_devices(&vpn_account)
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
    Ok(match &shared_state.device_state {
        DeviceState::LoggedIn {
            summary: account_summary,
            ..
        } => account_summary.clone(),
        DeviceState::LoggedOut => None,
    })
}

async fn handle_is_trial_eligible<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<bool, AccountCommandError> {
    let Some(account) = shared_state.device_state.account() else {
        return Err(AccountCommandError::NoAccountStored);
    };

    let mnemonic = todo!();
    let vpn_account = VpnAccount::new(mnemonic, VpnAccountMode::from(account.mode))
        .map_err(AccountCommandError::internal)?;

    let eligible = shared_state
        .vpn_api_client
        .get_active_subscriptions(&vpn_account)
        .await
        .map(|response| {
            tracing::debug!("Trial eligibility (raw: {:?})", response.trial_eligible);
            response.trial_eligible == Some(true)
        })
        .or_else(|err| {
            // A brand-new account is registered only after its first purchase, so the API does
            // not know it yet. No account means no history, which makes it eligible.
            if err.is_account_not_found() {
                tracing::debug!("Trial eligibility: account not registered yet");
                Ok(true)
            } else {
                Err(err)
            }
        })?;

    tracing::debug!("Trial eligible: {eligible}");
    Ok(eligible)
}

/// The denying Apple introductory-offer token, or `None` when the account may be offered a free
/// trial and Apple's own rule decides.
async fn handle_check_apple_intro_offer_eligibility<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    product_id: String,
    transaction_id: String,
) -> Result<Option<String>, AccountCommandError> {
    let Some(account) = shared_state.device_state.account() else {
        return Err(AccountCommandError::NoAccountStored);
    };

    let mnemonic = todo!();
    let vpn_account = VpnAccount::new(mnemonic, VpnAccountMode::from(account.mode))
        .map_err(AccountCommandError::internal)?;

    let compact_jws = shared_state
        .vpn_api_client
        .check_apple_intro_offer_eligibility(&vpn_account, product_id, transaction_id)
        .await
        .map(|response| response.compact_jws)
        .or_else(|err| {
            // An account the API has never seen has no history, so it is eligible and gets no
            // denying token.
            if err.is_account_not_found() {
                tracing::debug!("Apple intro offer eligibility: account not registered yet");
                Ok(None)
            } else {
                Err(err)
            }
        })?;

    tracing::debug!(
        "Apple intro offer eligibility: denying token {}",
        if compact_jws.is_some() {
            "present"
        } else {
            "absent"
        }
    );
    Ok(compact_jws)
}

pub(crate) async fn handle_get_deeplink<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    params: CreateDeeplinkParams,
) -> Result<String, AccountCommandError> {
    if params.kind != DeeplinkKind::PrivyLink {
        return Err(AccountCommandError::DeeplinkError(
            "Unexpected deep link kind".to_owned(),
        ));
    }

    // For `DeeplinkKind::PrivyLink`, the user must be logged-in via an API account
    let is_api_account = shared_state
        .device_state
        .account()
        .map(|account| account.mode == StoredAccountMode::Api)
        .unwrap_or(true);

    if is_api_account {
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
    } else {
        Err(AccountCommandError::DeeplinkError(
            "You can only link a Privy account if you are logged in with an API account".to_owned(),
        ))
    }
}

pub(crate) async fn handle_get_autologin_deeplink<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    params: CreateDeeplinkParams,
) -> Result<AutologinResponse, AccountCommandError> {
    let Some(account) = shared_state.device_state.account() else {
        return Err(AccountCommandError::NoAccountStored);
    };
    let mnemonic = todo!();

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
