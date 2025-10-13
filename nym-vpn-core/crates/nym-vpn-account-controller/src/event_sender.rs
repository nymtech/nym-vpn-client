// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::AccountControllerEvent;
use tokio::sync::{
    broadcast,
    broadcast::error::{RecvError, TryRecvError},
};
use tracing::debug;

pub struct AccountControllerEventReceiver {
    inner: broadcast::Receiver<AccountControllerEvent>,
}

pub struct AccountControllerEventSender {
    inner: broadcast::Sender<AccountControllerEvent>,
}

impl Default for AccountControllerEventSender {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountControllerEventSender {
    pub fn new() -> Self {
        AccountControllerEventSender {
            inner: broadcast::Sender::new(10),
        }
    }

    pub fn subscribe(&self) -> AccountControllerEventReceiver {
        AccountControllerEventReceiver {
            inner: self.inner.subscribe(),
        }
    }

    pub fn broadcast(&self, event: AccountControllerEvent) {
        if let Ok(receivers) = self.inner.send(event) {
            debug!("managed to broadcast AccountControllerEvent to {receivers} receivers");
        }
    }
}

impl AccountControllerEventReceiver {
    /// Re-subscribes to the channel starting from the current tail element.
    pub fn resubscribe(&mut self) {
        self.inner = self.inner.resubscribe();
    }

    pub async fn recv(&mut self) -> Result<AccountControllerEvent, RecvError> {
        self.inner.recv().await
    }

    pub fn try_recv(&mut self) -> Result<AccountControllerEvent, TryRecvError> {
        self.inner.try_recv()
    }

    pub fn blocking_recv(&mut self) -> Result<AccountControllerEvent, RecvError> {
        self.inner.blocking_recv()
    }
}
