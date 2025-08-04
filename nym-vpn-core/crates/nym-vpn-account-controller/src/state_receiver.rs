// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{AccountCommandError, AccountControllerState};
use tokio::sync::watch;

// Channel to keep track of the account controller state
#[derive(Clone)]
pub struct AccountStateReceiver {
    inner: watch::Receiver<AccountControllerState>,
}

impl AccountStateReceiver {
    pub fn new(inner: watch::Receiver<AccountControllerState>) -> Self {
        Self { inner }
    }

    pub async fn wait_for_account_ready_to_connect(&mut self) -> Result<(), AccountCommandError> {
        //Make sure we're not stuck there
        self.inner.mark_changed();

        while (self.inner.changed().await).is_ok() {
            match *self.inner.borrow() {
                AccountControllerState::Offline => {
                    return Err(AccountCommandError::Offline);
                }
                AccountControllerState::LoggedOut => {
                    return Err(AccountCommandError::NoAccountStored);
                }
                AccountControllerState::Error => {
                    return Err(AccountCommandError::Internal("Error state".into())); // SW better error
                }
                AccountControllerState::Syncing => {
                    tracing::debug!("Account controller is syncing, waiting for the next state");
                }
                AccountControllerState::ReadyToConnect => return Ok(()),
            }
        }
        Err(AccountCommandError::internal(
            "Account controller state receiver has closed",
        ))
    }
}
