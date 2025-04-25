// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use futures::StreamExt;
use nym_mixnet_client::SharedMixnetClient;
use nym_sdk::mixnet::ReconstructedMessage;
use tokio::{sync::broadcast, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::AuthClient;

pub type MixnetMessageBroadcastSender = broadcast::Sender<Arc<ReconstructedMessage>>;
pub type MixnetMessageBroadcastReceiver = broadcast::Receiver<Arc<ReconstructedMessage>>;

// The AuthClientsMixnetListener listens to mixnet messages and rebroadcasts them to the
// AuthClients, or whoever else is interested.
// While it is running, it has a lock on the shared mixnet client. This is the reason it's
// designed to be able to start and stop, so that the lock can be released when it's not needed.
//
// NOTE: this is potentially bit wasteful. Ideally we should have proper channels where the
// recipient only gets messages they're interested in.
pub struct AuthClientMixnetListener {
    // The shared mixnet client that we're listening to
    mixnet_client: SharedMixnetClient,

    // Broadcast channel for the messages that we re-broadcast to the AuthClients
    message_broadcast: MixnetMessageBroadcastSender,

    // Listen to cancel from the outside world
    external_cancel_token: CancellationToken,
}

impl AuthClientMixnetListener {
    pub fn new(mixnet_client: SharedMixnetClient) -> Self {
        let (message_broadcast, _) = broadcast::channel(100);
        Self {
            mixnet_client,
            message_broadcast,
            external_cancel_token: CancellationToken::new(),
        }
    }

    pub fn with_external_cancel_token(mut self, external_cancel_token: CancellationToken) -> Self {
        self.external_cancel_token = external_cancel_token;
        self
    }

    pub fn subscribe(&self) -> MixnetMessageBroadcastReceiver {
        self.message_broadcast.subscribe()
    }

    async fn run(self, cancel_token: CancellationToken) {
        let mut mixnet_client = self.mixnet_client.lock().await.take().unwrap();
        cancel_token
            .run_until_cancelled(async {
                while let Some(event) = mixnet_client.next().await {
                    if let Err(err) = self.message_broadcast.send(Arc::new(event)) {
                        tracing::error!("Failed to broadcast mixnet message: {err}");
                    }
                }
                tracing::error!("Mixnet client stream ended unexpectedly");
            })
            .await;
        self.mixnet_client.lock().await.replace(mixnet_client);
    }

    pub fn start(self) -> AuthClientMixnetListenerHandle {
        let mixnet_client = self.mixnet_client.clone();
        let message_broadcast = self.message_broadcast.clone();
        let external_cancel_token = self.external_cancel_token.clone();
        let internal_cancel_token = self.external_cancel_token.child_token();

        let handle = tokio::spawn(self.run(internal_cancel_token.clone()));

        AuthClientMixnetListenerHandle {
            mixnet_client,
            message_broadcast,
            external_cancel_token,
            internal_cancel_token,
            handle,
        }
    }
}

pub struct AuthClientMixnetListenerHandle {
    mixnet_client: SharedMixnetClient,
    message_broadcast: MixnetMessageBroadcastSender,
    external_cancel_token: CancellationToken,
    internal_cancel_token: CancellationToken,
    handle: JoinHandle<()>,
}

impl AuthClientMixnetListenerHandle {
    pub async fn new_auth_client(&self) -> AuthClient {
        AuthClient::new(
            self.mixnet_client.split_sender().await,
            self.message_broadcast.subscribe(),
            self.mixnet_client.stats_sender().await,
            self.mixnet_client.nym_address().await,
        )
        .await
    }

    pub fn subscribe(&self) -> MixnetMessageBroadcastReceiver {
        self.message_broadcast.subscribe()
    }

    pub async fn disconnect(mut self) -> AuthClientMixnetListener {
        self.internal_cancel_token.cancel();
        self = self.wait().await;
        AuthClientMixnetListener {
            mixnet_client: self.mixnet_client,
            message_broadcast: self.message_broadcast,
            external_cancel_token: self.external_cancel_token,
        }
    }

    pub async fn wait(mut self) -> Self {
        tokio::select! {
            join_result = &mut self.handle => {
                if let Err(err) = join_result {
                    tracing::error!("Error waiting for auth clients mixnet listener to stop: {err}");
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                tracing::error!("Timeout waiting for auth clients mixnet listener to stop. Forcing stop");
                self.handle.abort();
            }
        }
        self
    }
}
