// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_account_controller::AccountControllerCommander;
use nym_vpn_lib_types::{
    RegisterDeviceError, RequestZkNymError, SyncAccountError, SyncDeviceError,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("timeout")]
    Cancelled,

    #[error(transparent)]
    SyncAccount(#[from] SyncAccountError),

    #[error(transparent)]
    SyncDevice(#[from] SyncDeviceError),

    #[error(transparent)]
    RegisterDevice(#[from] RegisterDeviceError),

    #[error(transparent)]
    RequestZkNym(#[from] RequestZkNymError),
}

pub async fn wait_for_account_sync(
    account_controller_tx: AccountControllerCommander,
    cancel_token: CancellationToken,
) -> Result<(), Error> {
    cancel_token
        .run_until_cancelled(account_controller_tx.ensure_update_account())
        .await
        .ok_or(Error::Cancelled)?
        .map_err(Error::SyncAccount)
        .map(|_| ())
}

pub async fn wait_for_device_sync(
    account_controller_tx: AccountControllerCommander,
    cancel_token: CancellationToken,
) -> Result<(), Error> {
    cancel_token
        .run_until_cancelled(account_controller_tx.ensure_update_device())
        .await
        .ok_or(Error::Cancelled)?
        .map_err(Error::SyncDevice)
        .map(|_| ())
}

pub async fn wait_for_device_register(
    account_controller_tx: AccountControllerCommander,
    cancel_token: CancellationToken,
) -> Result<(), Error> {
    cancel_token
        .run_until_cancelled(account_controller_tx.ensure_register_device())
        .await
        .ok_or(Error::Cancelled)?
        .map_err(Error::RegisterDevice)
}

// Waiting for credentials to be ready can take a while if it's from scratch, in the order of 30
// seconds at least.
pub async fn wait_for_credentials_ready(
    account_controller_tx: AccountControllerCommander,
    cancel_token: CancellationToken,
) -> Result<(), Error> {
    cancel_token
        .run_until_cancelled(account_controller_tx.ensure_available_zk_nyms())
        .await
        .ok_or(Error::Cancelled)?
        .map_err(Error::RequestZkNym)
}
