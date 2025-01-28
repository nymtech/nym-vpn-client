// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_account_controller::{AccountCommandError, AccountControllerCommander};
use tokio_util::sync::CancellationToken;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("timeout")]
    Cancelled,

    #[error("account error: {0}")]
    Account(#[from] AccountCommandError),
}

async fn wait_for_sync_inner(
    account_controller_tx: AccountControllerCommander,
) -> Result<(), AccountCommandError> {
    account_controller_tx
        .ensure_update_account()
        .await
        .map(|_| ())?;
    account_controller_tx
        .ensure_update_device()
        .await
        .map(|_| ())
}

pub async fn wait_for_sync(
    account_controller_tx: AccountControllerCommander,
    cancel_token: CancellationToken,
) -> Result<(), Error> {
    cancel_token
        .run_until_cancelled(wait_for_sync_inner(account_controller_tx))
        .await
        .ok_or(Error::Cancelled)?
        .map_err(Error::Account)
}

pub async fn wait_for_device_register(
    account_controller_tx: AccountControllerCommander,
    cancel_token: CancellationToken,
) -> Result<(), Error> {
    cancel_token
        .run_until_cancelled(account_controller_tx.ensure_register_device())
        .await
        .ok_or(Error::Cancelled)?
        .map_err(Error::Account)
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
        .map_err(Error::Account)
}
