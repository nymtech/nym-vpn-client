// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{AccountControllerError, AccountControllerState};
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

    pub async fn wait_for_account_ready_to_connect(
        &mut self,
    ) -> Result<(), AccountControllerError> {
        //Make sure we're not stuck there
        self.inner.mark_changed();

        while (self.inner.changed().await).is_ok() {
            match self.inner.borrow().clone() {
                AccountControllerState::Offline => {
                    return Err(AccountControllerError::Offline);
                }
                AccountControllerState::LoggedOut => {
                    return Err(AccountControllerError::NoAccountStored);
                }
                AccountControllerState::Error(reason) => {
                    return Err(AccountControllerError::ErrorState(reason));
                }
                AccountControllerState::Syncing => {
                    tracing::debug!("Account controller is syncing, waiting for the next state");
                }
                AccountControllerState::ReadyToConnect => return Ok(()),
            }
        }
        Err(AccountControllerError::Internal(
            "Account controller state receiver has closed".into(),
        ))
    }
}
