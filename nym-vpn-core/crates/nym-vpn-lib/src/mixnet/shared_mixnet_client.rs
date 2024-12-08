// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
use std::os::fd::RawFd;
use std::sync::Arc;

use nym_sdk::mixnet::{ClientStatsEvents, MixnetClient, MixnetClientSender, Recipient};

#[derive(Clone)]
pub struct SharedMixnetClient {
    inner: Arc<tokio::sync::Mutex<Option<MixnetClient>>>,
    #[cfg(target_os = "android")]
    bypass_fn: Arc<dyn Fn(RawFd) + Send + Sync>,
}

impl SharedMixnetClient {
    pub fn new(
        mixnet_client: MixnetClient,
        #[cfg(target_os = "android")] bypass_fn: Arc<dyn Fn(RawFd) + Send + Sync>,
    ) -> Self {
        Self {
            inner: Arc::new(tokio::sync::Mutex::new(Some(mixnet_client))),
            #[cfg(target_os = "android")]
            bypass_fn,
        }
    }

    pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, Option<MixnetClient>> {
        self.inner.lock().await
    }

    pub async fn nym_address(&self) -> Recipient {
        *self.lock().await.as_ref().unwrap().nym_address()
    }

    pub async fn split_sender(&self) -> MixnetClientSender {
        self.lock().await.as_ref().unwrap().split_sender()
    }

    pub async fn send_stats_event(&self, event: ClientStatsEvents) {
        self.lock().await.as_ref().unwrap().send_stats_event(event);
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
        self.inner.clone()
    }

    #[cfg(target_os = "android")]
    pub fn bypass_fn(&self) -> Arc<dyn Fn(RawFd) + Send + Sync> {
        self.bypass_fn.clone()
    }
}
