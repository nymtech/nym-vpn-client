// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::SocketAddr;

use nym_vpn_api_client::response::{NymVpnDevice, NymVpnUsage};
use nym_vpn_lib_types::{AccountCommandError, VpnApiError};
use nym_vpn_store::mnemonic::Mnemonic;

use crate::{
    AvailableTicketbooks, SharedAccountState,
    commands::{ReturnSender, dispatch::CommonCommand},
    storage::AccountStorageOp,
};

pub(crate) async fn handle_common_command(
    command: CommonCommand,
    shared_state: &mut SharedAccountState,
) {
    match command {
        CommonCommand::GetStoredMnemonic(result_tx) => {
            result_tx.send(handle_get_stored_mnemonic(shared_state).await);
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
        CommonCommand::SetStaticApiAddresses(result_tx, static_api_addresses) => {
            result_tx.send(handle_set_static_api_addresses(
                shared_state,
                static_api_addresses,
            ));
        }
    };
}

// This goes into storage each time, to trigger platform's unlocking mechanism if secure storage is used
pub(crate) async fn handle_get_stored_mnemonic(
    shared_state: &mut SharedAccountState,
) -> Result<Option<Mnemonic>, AccountCommandError> {
    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::GetStoredMnemonic(tx))
        .map_err(AccountCommandError::internal)?;
    rx.await
        .map_err(AccountCommandError::internal)? // Channel error
        .map_err(AccountCommandError::storage) // Storage error
}

pub(crate) fn handle_get_account_identity(
    shared_state: &mut SharedAccountState,
) -> Result<Option<String>, AccountCommandError> {
    Ok(shared_state
        .vpn_api_account
        .as_ref()
        .map(|account| account.id().into()))
}

async fn handle_get_usage(
    shared_state: &mut SharedAccountState,
) -> Result<Vec<NymVpnUsage>, AccountCommandError> {
    let account = shared_state
        .vpn_api_account
        .as_ref()
        .ok_or(AccountCommandError::NoAccountStored)?;

    let usage = shared_state
        .vpn_api_client
        .get_usage(account)
        .await
        .map_err(|err| {
            VpnApiError::try_from(err)
                .map(AccountCommandError::from)
                .unwrap_or_else(AccountCommandError::internal)
        })?;
    // SW Better error handling?
    tracing::debug!("Usage: {:#?}", usage);
    Ok(usage.items)
}

pub(crate) fn handle_get_device_identity(
    shared_state: &SharedAccountState,
) -> Result<String, AccountCommandError> {
    let device = shared_state
        .device
        .as_ref()
        .ok_or(AccountCommandError::NoDeviceStored)?
        .identity_key()
        .to_string();

    tracing::debug!("Device identity: {device:?}");
    Ok(device)
}

async fn handle_get_devices(
    shared_state: &mut SharedAccountState,
) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
    tracing::debug!("Getting devices from API");

    let account = shared_state
        .vpn_api_account
        .as_ref()
        .ok_or(AccountCommandError::NoAccountStored)?;

    let devices = shared_state
        .vpn_api_client
        .get_devices(account)
        .await
        .map_err(|err| {
            VpnApiError::try_from(err)
                .map(AccountCommandError::from)
                .unwrap_or_else(AccountCommandError::internal)
        })?;
    // SW Better error handling?

    tracing::debug!("The account has the following devices associated to it:");
    // TODO: pagination
    for device in &devices.items {
        tracing::debug!("{:?}", device);
    }
    Ok(devices.items)
}

async fn handle_get_active_devices(
    shared_state: &mut SharedAccountState,
) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
    tracing::debug!("Getting active devices from API");

    let account = shared_state
        .vpn_api_account
        .as_ref()
        .ok_or(AccountCommandError::NoAccountStored)?;

    let devices = shared_state
        .vpn_api_client
        .get_active_devices(account)
        .await
        .map_err(|err| {
            VpnApiError::try_from(err)
                .map(AccountCommandError::from)
                .unwrap_or_else(AccountCommandError::internal)
        })?;
    // SW Better error handling?

    tracing::debug!("The account has the following active devices associated to it:");
    // TODO: pagination
    for device in &devices.items {
        tracing::debug!("{:?}", device);
    }
    Ok(devices.items)
}

async fn handle_get_available_tickets(
    shared_state: &SharedAccountState,
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

pub(crate) fn handle_set_static_api_addresses(
    shared_state: &mut SharedAccountState,
    static_api_addresses: Option<Vec<SocketAddr>>,
) -> Result<(), AccountCommandError> {
    nym_vpn_api_client::VpnApiClient::new_with_resolver_overrides(
        shared_state.vpn_api_client.current_url().clone(),
        shared_state.config.user_agent.clone(),
        static_api_addresses.as_deref(),
    )
    .map(|new_vpn_api_client| {
        shared_state
            .vpn_api_client
            .swap_inner_client(new_vpn_api_client.clone());
    })
    .map_err(|e| AccountCommandError::internal(format!("Failed to set static addresses: {e}")))
}
