// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#![allow(clippy::disallowed_types)]
use std::{io, time::Duration};

use anyhow::Context;
use futures::{FutureExt, SinkExt, StreamExt, channel::mpsc, future::BoxFuture, pin_mut};
use hyper_util::rt::TokioIo;
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use test_rpc::transport::{ConnectionHandle, GrpcForwarder};
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
use tokio_util::codec::{Decoder, LengthDelimitedCodec};
use tower::Service;

/// Unary gRPC timeout over the serial mux.
const GRPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
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
    pub async fn new_client_nym(&self) -> anyhow::Result<NymProxyClient> {
        // FIXME: Ugly workaround to ensure that we don't receive stuff from a
        // previous RPC session.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        log::trace!("Nym daemon: connecting");
        NymProxyClient::new_over_serial(self.service.clone(), Some(GRPC_REQUEST_TIMEOUT))
            .await
            .context("Failed to create RpcClient over serial")
    }
}

pub fn new_rpc_client(
    connection_handle: ConnectionHandle,
    nym_daemon_transport: GrpcForwarder,
) -> RpcClientProvider {
    let mut framed_transport = LengthDelimitedCodec::new().framed(nym_daemon_transport);
    let (management_channel_provider_tx, mut management_channel_provider_rx) = mpsc::unbounded();

    tokio::spawn(async move {
        let mut read_buf = [0u8; CONVERTER_BUF_SIZE];
        loop {
            log::trace!("waiting for management interface client");

            let mut management_channel_in: DuplexStream =
                match management_channel_provider_rx.next().await {
                    Some(channel) => TokioIo::into_inner(channel),
                    None => {
                        log::debug!("👻 exiting management interface forward loop");
                        break;
                    }
                };

            // clear data from last session
            while let Some(_next) = framed_transport.next().now_or_never() {}

            log::debug!("👻 Entering loop...");
            loop {
                let proxy_read = management_channel_in.read(&mut read_buf);
                pin_mut!(proxy_read);

                let reset_notified = connection_handle.notified_reset();
                pin_mut!(reset_notified);

                match futures::future::select(
                    reset_notified,
                    futures::future::select(framed_transport.next(), proxy_read),
                )
                .await
                {
                    futures::future::Either::Left(_) => {
                        log::debug!("Restarting daemon RPC client");
                        break;
                    }
                    futures::future::Either::Right((
                        futures::future::Either::Left((Some(Ok(bytes)), _)),
                        _,
                    )) => {
                        if bytes.is_empty() {
                            log::debug!("👻 Management channel EOF");

                            if let Err(error) = management_channel_in.shutdown().await {
                                log::error!("Failed to shut down forwarder stream: {}", error);
                            }
                            break;
                        }
                        if management_channel_in.write_all(&bytes).await.is_err() {
                            break;
                        }
                    }
                    futures::future::Either::Right((
                        futures::future::Either::Left((Some(Err(error)), _)),
                        _,
                    )) => {
                        log::debug!("Management channel stream errored: {}", error);
                        break;
                    }
                    futures::future::Either::Right((
                        futures::future::Either::Left((None, _)),
                        _,
                    )) => break,
                    futures::future::Either::Right((
                        futures::future::Either::Right((Ok(num_bytes), _)),
                        _,
                    )) => {
                        if let Err(e) = framed_transport
                            .send(read_buf[..num_bytes].to_vec().into())
                            .await
                        {
                            log::error!("👻 error: {e}");
                            break;
                        }
                        if num_bytes == 0 {
                            log::debug!("Nym daemon connection EOF (not an error)");
                            break;
                        }
                    }
                    futures::future::Either::Right((
                        futures::future::Either::Right((Err(_), _)),
                        _,
                    )) => {
                        let _ = framed_transport.send(bytes::Bytes::new()).await;
                        break;
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
