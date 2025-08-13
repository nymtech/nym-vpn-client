// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Uniffi interface for RPC client from `nym-vpn-proto`.

uniffi::setup_scaffolding!();

use std::sync::Arc;

use futures::StreamExt;
use nym_vpn_proto::rpc_client::{Error as DaemonRpcError, RpcClient as DaemonRpcClient};
use tokio_util::sync::CancellationToken;

use nym_vpn_lib_types_uniffi::{TunnelEvent, TunnelState};

#[derive(Debug, uniffi::Object)]
pub struct RpcError {
    inner: DaemonRpcError,
}

#[uniffi::export]
impl RpcError {
    pub fn message(&self) -> String {
        self.inner.to_string()
    }
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<DaemonRpcError> for RpcError {
    fn from(err: DaemonRpcError) -> Self {
        RpcError { inner: err }
    }
}

pub type Result<T, E = RpcError> = std::result::Result<T, E>;

#[derive(Clone, uniffi::Object)]
pub struct StreamObserver {
    cancel_token: CancellationToken,
}

#[uniffi::export]
impl StreamObserver {
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }
}

impl StreamObserver {
    fn new(cancel_token: CancellationToken) -> Self {
        StreamObserver { cancel_token }
    }
}

impl Drop for StreamObserver {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}

#[uniffi::export(with_foreign)]
pub trait TunnelEventObserver: Send + Sync {
    fn on_tunnel_event(&self, event: TunnelEvent);
    fn on_close(&self);
}

#[derive(Clone, uniffi::Object)]
struct RpcClient {
    inner: DaemonRpcClient,
}

#[uniffi::export(async_runtime = "tokio")]
impl RpcClient {
    #[uniffi::constructor]
    pub async fn new() -> Result<Self> {
        Ok(Self {
            inner: DaemonRpcClient::new().await?,
        })
    }

    pub async fn get_tunnel_state(&self) -> Result<TunnelState> {
        Ok(self
            .inner
            .clone()
            .get_tunnel_state()
            .await
            .map(TunnelState::from)?)
    }

    pub async fn disconnect_tunnel(&self) -> Result<()> {
        self.inner.clone().disconnect_tunnel().await?;
        Ok(())
    }

    pub async fn listen_to_events(
        &self,
        observer: Arc<dyn TunnelEventObserver>,
    ) -> Result<StreamObserver> {
        let cancel_token = CancellationToken::new();
        let child_token = cancel_token.child_token();
        let mut event_stream = self.inner.clone().listen_to_events().await?;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = child_token.cancelled() => {
                        break;
                    },
                    event = event_stream.next() => {
                        match event {
                            Some(Ok(evt)) => {
                                let tunnel_event = TunnelEvent::from(evt);
                                observer.on_tunnel_event(tunnel_event);
                            }
                            Some(Err(err)) => {
                                tracing::error!("Error receiving event: {err}");
                                break;
                            }
                            None => break
                        }
                    }
                }
            }

            observer.on_close();
        });

        Ok(StreamObserver::new(cancel_token))
    }
}
