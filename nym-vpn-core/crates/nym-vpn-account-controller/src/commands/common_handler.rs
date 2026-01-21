// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    ResolverOverrides,
    response::{NymVpnDevice, NymVpnUsage},
};
use nym_vpn_lib_types::{AccountCommandError, DeeplinkKind, VpnAccountSummary};
use url::Url;

use crate::{
    AvailableTicketbooks, SharedAccountState,
    commands::{ReturnSender, dispatch::CommonCommand},
    storage::AccountStorageOp,
};
use nym_vpn_store::account::StorableAccount;

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
        CommonCommand::GetAvailableTickets(result_tx) => {
            result_tx.send(handle_get_available_tickets(shared_state).await);
        }
        CommonCommand::SetResolverOverrides(result_tx, resolver_overrides) => {
            result_tx.send(handle_set_resolver_overrides(shared_state, resolver_overrides).await);
        }
        CommonCommand::GetAccountSummary(result_tx) => {
            result_tx.send(handle_get_account_summary(shared_state).await);
        }
        CommonCommand::GetDeeplink(result_tx, (kind, name, base_url)) => {
            result_tx.send(handle_get_deeplink(shared_state, kind, name, base_url).await)
        }
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

pub(crate) async fn handle_get_available_tickets<C: ConnectivityMonitor>(
    shared_state: &SharedAccountState<C>,
) -> Result<AvailableTicketbooks, AccountCommandError> {
    tracing::debug!("Getting available tickets from local credential storage");

    shared_state
        .credential_storage
        .print_info()
        .await
        .map_err(|err| AccountCommandError::Storage(err.to_string()))?;
    shared_state
        .credential_storage
        .get_available_ticketbooks()
        .await
        .map_err(|err| AccountCommandError::Storage(err.to_string()))
}

pub(crate) async fn handle_set_resolver_overrides<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    resolver_overrides: Option<ResolverOverrides>,
) -> Result<(), AccountCommandError> {
    shared_state
        .vpn_api_client
        .override_resolver(resolver_overrides.as_ref())
        .await
        .map_err(|e| {
            AccountCommandError::internal(format!("Failed to set resolver overrides: {e}"))
        })
}

pub(crate) async fn handle_get_account_summary<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Option<VpnAccountSummary>, AccountCommandError> {
    Ok(shared_state.vpn_account_summary.clone())
}

pub(crate) async fn handle_get_deeplink<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    kind: DeeplinkKind,
    name: String,
    base_url: Url,
) -> Result<String, AccountCommandError> {
    // Housekeeping
    shared_state.deeplinks.remove_expired();

    // Create a new Deeplink for this request
    let deeplink = shared_state
        .deeplinks
        .create_deeplink(kind, &name)
        .map_err(|e| AccountCommandError::DeeplinkError(e.to_string()))?;

    // Pass back the URL to use for this deeplink
    Ok(deeplink.create_url(&base_url))
}
