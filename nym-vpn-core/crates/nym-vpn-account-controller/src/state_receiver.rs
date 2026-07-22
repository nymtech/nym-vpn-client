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

    /// Returns Ok() when the AC is ready to connect, returns an Error if it can't reach that state
    pub async fn wait_for_account_ready_to_connect(
        &mut self,
    ) -> Result<(), AccountControllerError> {
        loop {
            let state = self.get_state();
            match state {
                AccountControllerState::Offline => {
                    return Err(AccountControllerError::Offline);
                }
                AccountControllerState::LoggedOut => {
                    return Err(AccountControllerError::NoAccountStored);
                }
                AccountControllerState::Error(reason) => {
                    return Err(AccountControllerError::ErrorState(reason));
                }
                AccountControllerState::Syncing | AccountControllerState::PendingSubscription => {
                    tracing::debug!("Account controller is {state}, waiting for the next state");

                    self.inner.changed().await.map_err(|_| {
                        AccountControllerError::Internal(
                            "Account controller state receiver has closed".into(),
                        )
                    })?;
                }
                AccountControllerState::ReadyToConnect | AccountControllerState::Decentralised => {
                    return Ok(());
                }
            }
        }
    }

    pub fn get_state(&self) -> AccountControllerState {
        self.inner.borrow().to_owned()
    }

    pub fn subscribe(&self) -> watch::Receiver<AccountControllerState> {
        self.inner.clone()
    }
}
