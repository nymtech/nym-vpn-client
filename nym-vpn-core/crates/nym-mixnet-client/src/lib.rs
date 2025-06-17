// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
use std::os::fd::RawFd;
use std::sync::Arc;

use nym_sdk::mixnet::{
    ClientStatsEvents, ClientStatsSender, LaneQueueLengths, MixnetClient, MixnetClientSender,
    MixnetMessageSender, Recipient, ed25519,
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

    pub async fn nym_address(&self) -> Result<Recipient, &'static str> {
        Ok(*self.lock().await.as_ref().ok_or("MixnetClient has been disconnected")?.nym_address())
    }

    pub async fn sign(&self, data: &[u8]) -> ed25519::Signature {
        self.lock().await.as_ref().unwrap().sign(data)
    }

    pub async fn send(&self, msg: nym_sdk::mixnet::InputMessage) -> Result<(), nym_sdk::Error> {
        match self.lock().await.as_mut() {
            Some(client) => client.send(msg).await,
            None => Err(nym_sdk::Error::MessageSendingFailure),
        }
    }

    pub async fn split_sender(&self) -> Result<MixnetClientSender, &'static str> {
        Ok(self.lock().await.as_ref().ok_or("MixnetClient has been disconnected")?.split_sender())
    }

    pub async fn stats_sender(&self) -> Result<ClientStatsSender, &'static str> {
        Ok(self.lock().await.as_ref().ok_or("MixnetClient has been disconnected")?.stats_events_reporter())
    }

    pub async fn send_stats_event(&self, event: ClientStatsEvents) {
        self.lock().await.as_ref().unwrap().send_stats_event(event);
    }

    pub async fn shared_lane_queue_lengths(&self) -> Result<LaneQueueLengths, &'static str> {
        Ok(self.lock()
            .await
            .as_ref()
            .ok_or("MixnetClient has been disconnected")?
            .shared_lane_queue_lengths())
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
        if let Some(mixnet_client) = self.lock().await.take() {
            mixnet_client.disconnect().await;
        }
    }

    // If the mixnet does have an external task manager, call this method to dispose.
    pub async fn dispose(self) {
        // A mixnet client that has an external task manager is dropped to disconnect.
        self.lock().await.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Simple test to verify that methods return errors when client is None
    #[tokio::test]
    async fn test_disconnected_client_behavior() {
        // Create a SharedMixnetClient with None (simulating a disconnected state)
        let shared_client = SharedMixnetClient {
            inner: Arc::new(tokio::sync::Mutex::new(None)),
            #[cfg(unix)]
            connection_fd_callback: Arc::new(|_| {}),
        };

        // All methods should return errors gracefully
        assert!(shared_client.nym_address().await.is_err());
        assert!(shared_client.split_sender().await.is_err());
        assert!(shared_client.stats_sender().await.is_err());
        assert!(shared_client.shared_lane_queue_lengths().await.is_err());

        // Check error messages for those that we can check
        match shared_client.nym_address().await {
            Err(msg) => assert_eq!(msg, "MixnetClient has been disconnected"),
            Ok(_) => panic!("Expected error"),
        }
        
        match shared_client.split_sender().await {
            Err(msg) => assert_eq!(msg, "MixnetClient has been disconnected"),
            Ok(_) => panic!("Expected error"),
        }
        
        match shared_client.stats_sender().await {
            Err(msg) => assert_eq!(msg, "MixnetClient has been disconnected"),
            Ok(_) => panic!("Expected error"),
        }
        
        match shared_client.shared_lane_queue_lengths().await {
            Err(msg) => assert_eq!(msg, "MixnetClient has been disconnected"),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_multiple_disconnects_safe() {
        // Create a SharedMixnetClient with None
        let shared_client = SharedMixnetClient {
            inner: Arc::new(tokio::sync::Mutex::new(None)),
            #[cfg(unix)]
            connection_fd_callback: Arc::new(|_| {}),
        };

        // Should be safe to call disconnect multiple times on an already disconnected client
        shared_client.disconnect().await;
        shared_client.disconnect().await;
        shared_client.disconnect().await;

        // Should still return appropriate errors
        assert!(shared_client.nym_address().await.is_err());
    }
}
