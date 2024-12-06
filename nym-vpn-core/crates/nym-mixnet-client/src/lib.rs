// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_sdk::mixnet::{
    ed25519, ClientStatsEvents, MixnetClient, MixnetClientSender, MixnetMessageSender, Recipient,
};
#[cfg(target_os = "android")]
use nym_tunnel_provider::android::AndroidTunProvider;

use crate::error::*;

pub mod error;

#[derive(Clone)]
pub struct SharedMixnetClient {
    inner: Arc<tokio::sync::Mutex<Option<MixnetClient>>>,
    #[cfg(target_os = "android")]
    tun_provider: Arc<dyn AndroidTunProvider>,
}

impl SharedMixnetClient {
    pub fn new(
        mixnet_client: MixnetClient,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
    ) -> Self {
        Self {
            inner: Arc::new(tokio::sync::Mutex::new(Some(mixnet_client))),
            #[cfg(target_os = "android")]
            tun_provider,
        }
    }

    pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, Option<MixnetClient>> {
        self.inner.lock().await
    }

    pub async fn nym_address(&self) -> Recipient {
        *self.lock().await.as_ref().unwrap().nym_address()
    }

    pub async fn sign(&self, data: &[u8]) -> ed25519::Signature {
        self.lock().await.as_ref().unwrap().sign(data)
    }

    pub async fn send(&self, msg: nym_sdk::mixnet::InputMessage) -> Result<()> {
        self.lock().await.as_mut().unwrap().send(msg).await?;
        Ok(())
    }

    pub async fn split_sender(&self) -> MixnetClientSender {
        self.lock().await.as_ref().unwrap().split_sender()
    }

    pub async fn send_stats_event(&self, event: ClientStatsEvents) {
        self.lock().await.as_ref().unwrap().send_stats_event(event);
    }

    #[cfg(target_os = "android")]
    pub async fn bypass(&self) -> Result<()> {
        let fd = self
            .lock()
            .await
            .as_ref()
            .unwrap()
            .gateway_connection()
            .gateway_ws_fd
            .ok_or(Error::NoWebSocket)?;
        self.tun_provider.bypass(fd);
        Ok(())
    }

    pub fn inner(&self) -> Arc<tokio::sync::Mutex<Option<MixnetClient>>> {
        self.inner.clone()
    }

    #[cfg(target_os = "android")]
    pub fn tun_provider(&self) -> Arc<dyn AndroidTunProvider> {
        self.tun_provider.clone()
    }
}
