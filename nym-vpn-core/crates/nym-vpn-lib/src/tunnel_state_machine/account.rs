// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_account_controller::{AccountCommandError, AccountControllerCommander};
use tokio_util::sync::CancellationToken;

// use super::{tunnel::Error, tunnel::Result};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("timeout")]
    Timeout,

    #[error("account error: {0}")]
    Account(#[from] AccountCommandError),
}

pub async fn wait_for_account_ready(
    account_controller_tx: AccountControllerCommander,
    credentials_mode: bool,
    cancel_token: CancellationToken,
) -> Result<(), Error> {
    let fut = account_controller_tx.wait_for_account_ready_to_connect(credentials_mode);
    cancel_token
        .run_until_cancelled(fut)
        .await
        .ok_or(Error::Timeout)?
        .map_err(Error::Account)
}
