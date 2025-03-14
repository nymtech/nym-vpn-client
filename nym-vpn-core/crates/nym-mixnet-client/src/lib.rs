// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
use std::os::fd::RawFd;
use std::sync::Arc;

use nym_sdk::mixnet::{
    ed25519, ClientStatsEvents, ClientStatsSender, LaneQueueLengths, MixnetClient,
    MixnetClientSender, MixnetMessageSender, Recipient,
};

#[derive(Clone)]
pub struct SharedMixnetClient {
    inner: Arc<tokio::sync::Mutex<Option<MixnetClient>>>,
    #[cfg(unix)]
    connection_fd_callback: Arc<dyn Fn(RawFd) + Send + Sync>,
}

impl SharedMixnetClient {
    pub fn new(
        mixnet_client: MixnetClient,
        #[cfg(unix)] connection_fd_callback: Arc<dyn Fn(RawFd) + Send + Sync>,
    ) -> Self {
        Self {
            inner: Arc::new(tokio::sync::Mutex::new(Some(mixnet_client))),
            #[cfg(unix)]
            connection_fd_callback,
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

    pub async fn send(&self, msg: nym_sdk::mixnet::InputMessage) -> Result<(), nym_sdk::Error> {
        self.lock().await.as_mut().unwrap().send(msg).await?;
        Ok(())
    }

    pub async fn split_sender(&self) -> MixnetClientSender {
        self.lock().await.as_ref().unwrap().split_sender()
    }

    pub async fn stats_sender(&self) -> ClientStatsSender {
        self.lock().await.as_ref().unwrap().stats_events_reporter()
    }

    pub async fn send_stats_event(&self, event: ClientStatsEvents) {
        self.lock().await.as_ref().unwrap().send_stats_event(event);
    }

    pub async fn shared_lane_queue_lengths(&self) -> LaneQueueLengths {
        self.lock()
            .await
            .as_ref()
            .unwrap()
            .shared_lane_queue_lengths()
    }

    #[cfg(unix)]
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

    #[cfg(unix)]
    pub fn connection_fd_callback(&self) -> Arc<dyn Fn(RawFd) + Send + Sync> {
        self.connection_fd_callback.clone()
    }

    // If the mixnet client does NOT have an external task manager, call this method to disconnect.
    pub async fn disconnect(&self) {
        let mixnet_client = self.lock().await.take().unwrap();
        mixnet_client.disconnect().await;
    }

    // If the mixnet does have an external task manager, call this method to dispose.
    pub async fn dispose(self) {
        // A mixnet client that has an external task manager is dropped to disconnect.
        self.lock().await.take();
    }
}
