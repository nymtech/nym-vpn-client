// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::pin::Pin;

use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::UnixStream,
};
use tokio_stream::Stream;
use tonic::transport::server::Connected;

use crate::authentication::{AuthenticationLayer, error::AuthenticationError};
#[cfg(any(not(debug_assertions), feature = "xpc"))]
use crate::xpc::{common::XpcConnection, daemon::XpcService};

pub enum StreamItem {
    #[cfg(any(not(debug_assertions), feature = "xpc"))]
    Xpc(XpcConnection),
    Uds(UnixStream),
}

#[cfg(any(not(debug_assertions), feature = "xpc"))]
impl From<XpcConnection> for StreamItem {
    fn from(value: XpcConnection) -> Self {
        Self::Xpc(value)
    }
}

impl From<UnixStream> for StreamItem {
    fn from(value: UnixStream) -> Self {
        Self::Uds(value)
    }
}

impl AsyncWrite for StreamItem {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.get_mut() {
            #[cfg(any(not(debug_assertions), feature = "xpc"))]
            StreamItem::Xpc(xpc_connection) => Pin::new(xpc_connection).poll_write(cx, buf),
            StreamItem::Uds(unix_stream) => Pin::new(unix_stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(any(not(debug_assertions), feature = "xpc"))]
            StreamItem::Xpc(xpc_connection) => Pin::new(xpc_connection).poll_flush(cx),
            StreamItem::Uds(unix_stream) => Pin::new(unix_stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(any(not(debug_assertions), feature = "xpc"))]
            StreamItem::Xpc(xpc_connection) => Pin::new(xpc_connection).poll_shutdown(cx),
            StreamItem::Uds(unix_stream) => Pin::new(unix_stream).poll_shutdown(cx),
        }
    }
}

impl AsyncRead for StreamItem {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(any(not(debug_assertions), feature = "xpc"))]
            StreamItem::Xpc(xpc_connection) => Pin::new(xpc_connection).poll_read(cx, buf),
            StreamItem::Uds(unix_stream) => Pin::new(unix_stream).poll_read(cx, buf),
        }
    }
}

impl Connected for StreamItem {
    type ConnectInfo = ();

    fn connect_info(&self) -> Self::ConnectInfo {}
}

// Not implemented yet, implicit is to consider it authenticated
pub(crate) async fn is_authenticated(_stream: &mut StreamItem) -> Result<(), AuthenticationError> {
    Ok(())
}

#[cfg(any(not(debug_assertions), feature = "xpc"))]
pub(crate) fn incoming_xpc(
    xpc_service: XpcService,
) -> impl Stream<Item = std::io::Result<StreamItem>> {
    let auth_layer = AuthenticationLayer::new(xpc_service);
    auth_layer.stream()
}

#[allow(unused)]
pub(crate) fn incoming_uds(
    uds: crate::uds::Uds,
) -> impl Stream<Item = std::io::Result<StreamItem>> {
    let auth_layer = AuthenticationLayer::new(uds);
    auth_layer.stream()
}
