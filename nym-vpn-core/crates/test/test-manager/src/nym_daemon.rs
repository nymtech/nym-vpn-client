// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#![allow(clippy::disallowed_types)]
use std::{future::Future, io, time::Duration};

use anyhow::Context;
use futures::{StreamExt, channel::mpsc, future::BoxFuture};
use hyper_util::rt::TokioIo;
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use test_rpc::transport::{
    ConnectionHandle, GrpcForwarder, forward_framed_bidirectional, length_delimited_framed_halves,
    synchronize_framed_session,
};
use tokio::io::DuplexStream;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tower::Service;

/// Unary gRPC timeout over the serial mux.
const GRPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const RPC_CLIENT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const RPC_SESSION_SYNC_TIMEOUT: Duration = Duration::from_secs(30);
const CONVERTER_BUF_SIZE: usize = 16 * 1024;

#[derive(Clone)]
pub(super) struct DummyService {
    pub(super) management_channel_provider_tx: mpsc::UnboundedSender<TokioIo<DuplexStream>>,
}

impl<Request> Service<Request> for DummyService {
    type Response = TokioIo<DuplexStream>;
    type Error = std::io::Error;
    type Future = BoxFuture<'static, Result<TokioIo<DuplexStream>, Self::Error>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: Request) -> Self::Future {
        log::trace!("DummyService::call");

        let (channel_in, channel_out) = tokio::io::duplex(CONVERTER_BUF_SIZE);
        let notifier_tx = self.management_channel_provider_tx.clone();

        Box::pin(async move {
            notifier_tx
                .unbounded_send(TokioIo::new(channel_in))
                .map_err(|_| io::Error::other("stream receiver is down"))?;
            Ok(TokioIo::new(channel_out))
        })
    }
}

#[derive(Clone)]
pub struct RpcClientProvider {
    pub(super) service: DummyService,
}

impl RpcClientProvider {
    /// Test seam: provider whose `new_client_nym` will hang until connect timeout unless a
    /// forward loop consumes the duplex (asserts recreate path requests a new channel).
    #[cfg(test)]
    pub(crate) fn dangling_for_tests() -> (Self, mpsc::UnboundedReceiver<TokioIo<DuplexStream>>) {
        let (management_channel_provider_tx, management_channel_provider_rx) = mpsc::unbounded();
        (
            Self {
                service: DummyService {
                    management_channel_provider_tx,
                },
            },
            management_channel_provider_rx,
        )
    }

    pub async fn new_client_nym(&self) -> anyhow::Result<NymProxyClient> {
        log::trace!("Nym daemon: connecting");
        await_rpc_client_connection(
            async {
                NymProxyClient::new_over_serial(self.service.clone(), Some(GRPC_REQUEST_TIMEOUT))
                    .await
                    .context("Failed to create RpcClient over serial")
            },
            RPC_CLIENT_CONNECT_TIMEOUT,
        )
        .await
    }
}

async fn await_rpc_client_connection<T>(
    connection: impl Future<Output = anyhow::Result<T>>,
    timeout: Duration,
) -> anyhow::Result<T> {
    tokio::time::timeout(timeout, connection)
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "Timed out after {}s creating RpcClient over serial",
                timeout.as_secs()
            )
        })?
}

async fn await_session_synchronization<R, W>(
    framed_read: &mut FramedRead<R, LengthDelimitedCodec>,
    framed_write: &mut FramedWrite<W, LengthDelimitedCodec>,
    timeout: Duration,
) -> io::Result<()>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    tokio::time::timeout(
        timeout,
        synchronize_framed_session(framed_read, framed_write),
    )
    .await
    .map_err(|_| {
        io::Error::new(
            io::ErrorKind::TimedOut,
            format!(
                "Timed out after {}s synchronizing daemon RPC session",
                timeout.as_secs()
            ),
        )
    })?
}

pub fn new_rpc_client(
    connection_handle: ConnectionHandle,
    nym_daemon_transport: GrpcForwarder,
) -> RpcClientProvider {
    let (mut framed_read, mut framed_write) = length_delimited_framed_halves(nym_daemon_transport);
    let (management_channel_provider_tx, mut management_channel_provider_rx) = mpsc::unbounded();

    tokio::spawn(async move {
        loop {
            log::trace!("waiting for management interface client");

            let management_channel_in: DuplexStream =
                match management_channel_provider_rx.next().await {
                    Some(channel) => TokioIo::into_inner(channel),
                    None => {
                        log::debug!("👻 exiting management interface forward loop");
                        break;
                    }
                };

            if let Err(error) = await_session_synchronization(
                &mut framed_read,
                &mut framed_write,
                RPC_SESSION_SYNC_TIMEOUT,
            )
            .await
            {
                log::debug!("Failed to synchronize daemon RPC session: {error}");
                break;
            }

            log::debug!("👻 Entering loop...");
            tokio::select! {
                _ = connection_handle.notified_reset() => {
                    log::debug!("Restarting daemon RPC client");
                }
                result = forward_framed_bidirectional(
                    management_channel_in,
                    &mut framed_read,
                    &mut framed_write,
                ) => {
                    match result {
                        Ok(()) => log::debug!("Nym daemon connection EOF (not an error)"),
                        Err(error) => log::debug!("Management channel stream errored: {error}"),
                    }
                }
            }
        }
    });

    let service = DummyService {
        management_channel_provider_tx,
    };

    RpcClientProvider { service }
}

#[cfg(test)]
mod tests {
    use super::{await_rpc_client_connection, await_session_synchronization};
    use std::{future::pending, time::Duration};
    use test_rpc::transport::length_delimited_framed_halves;

    #[tokio::test]
    async fn serial_rpc_client_connection_is_bounded() {
        let error =
            await_rpc_client_connection(pending::<anyhow::Result<()>>(), Duration::from_millis(1))
                .await
                .expect_err("pending connection must time out");

        assert!(error.to_string().contains("creating RpcClient over serial"));
    }

    #[tokio::test]
    async fn serial_rpc_client_connection_returns_success() {
        let result = await_rpc_client_connection(async { Ok(42) }, Duration::from_secs(1))
            .await
            .expect("ready connection");

        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn serial_rpc_session_synchronization_is_bounded() {
        let (transport, _unresponsive_peer) = tokio::io::duplex(64);
        let (mut framed_read, mut framed_write) = length_delimited_framed_halves(transport);

        let error = await_session_synchronization(
            &mut framed_read,
            &mut framed_write,
            Duration::from_millis(1),
        )
        .await
        .expect_err("missing synchronization acknowledgement must time out");

        assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    }
}
