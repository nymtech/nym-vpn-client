// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_sdk::mixnet::{MixnetClient, MixnetClientSender, Recipient};

#[derive(Clone)]
pub struct SharedMixnetClient(Arc<tokio::sync::Mutex<Option<MixnetClient>>>);

impl SharedMixnetClient {
    pub fn new(mixnet_client: MixnetClient) -> Self {
        Self(Arc::new(tokio::sync::Mutex::new(Some(mixnet_client))))
    }

    pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, Option<MixnetClient>> {
        self.0.lock().await
    }

    pub async fn nym_address(&self) -> Recipient {
        *self.lock().await.as_ref().unwrap().nym_address()
    }

    pub async fn split_sender(&self) -> MixnetClientSender {
        self.lock().await.as_ref().unwrap().split_sender()
    }

    #[cfg(target_os = "android")]
    pub async fn gateway_ws_fd(&self) -> Option<std::os::fd::RawFd> {
        self.lock()
            .await
            .as_ref()
            .unwrap()
            .gateway_connection()
            .gateway_ws_fd
    }

    pub fn inner(&self) -> Arc<tokio::sync::Mutex<Option<MixnetClient>>> {
        self.0.clone()
    }
}
