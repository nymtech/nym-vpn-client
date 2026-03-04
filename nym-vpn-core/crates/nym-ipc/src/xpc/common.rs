// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    pin::Pin,
    sync::{Arc, atomic::AtomicBool},
};

use objc2::{
    AnyThread, DefinedClass as _, define_class, extern_protocol, msg_send,
    rc::Retained,
    runtime::{AnyProtocol, ProtocolObject},
};
use objc2_foundation::{NSData, NSObject, NSObjectProtocol, NSXPCInterface};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc::UnboundedSender,
};
use tokio_stream::{Stream, wrappers::UnboundedReceiverStream};
use tokio_util::sync::DropGuard;
use tonic::transport::server::Connected;

pub(crate) const DAEMON_BUNDLE_IDENTIFIER: &str = "net.nymtech.vpn.daemon";

pub(crate) fn connection_interface() -> Retained<NSXPCInterface> {
    ensure_xpc_protocols_linked();
    unsafe {
        NSXPCInterface::interfaceWithProtocol(AnyProtocol::get(c"NSConnectionInterface").unwrap())
    }
}

extern_protocol!(
    /// # Safety
    ///
    /// The name is correct.
    #[name = "NSConnectionInterface"]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe trait NSConnectionInterface {
        /// # Safety
        ///
        /// The method is correctly specified.
        #[unsafe(method(write:))]
        fn write(&self, buf: &NSData);
    }
);

pub(crate) struct ConnectionInterfaceObjIvars {
    data_tx: UnboundedSender<Vec<u8>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = ConnectionInterfaceObjIvars]
    pub(crate) struct ConnectionInterfaceObj;

    unsafe impl NSObjectProtocol for ConnectionInterfaceObj {}

    unsafe impl NSConnectionInterface for ConnectionInterfaceObj {
        #[unsafe(method(write:))]
        fn write(&self, buf: &NSData) {
            if self.ivars().data_tx.send(buf.to_vec()).is_err() {
                tracing::error!("Daemon receiver shouldn't be dropped");
            }
        }
    }
);

unsafe extern "C" {
    fn nym_force_link_xpc_protocols();
}

#[inline]
pub fn ensure_xpc_protocols_linked() {
    unsafe { nym_force_link_xpc_protocols() }
}

impl ConnectionInterfaceObj {
    pub(crate) fn new(data_tx: UnboundedSender<Vec<u8>>) -> Retained<Self> {
        let this = Self::alloc().set_ivars(ConnectionInterfaceObjIvars { data_tx });
        unsafe { msg_send![super(this), init] }
    }
}

pub struct XpcConnection {
    own_interface: Option<Retained<ConnectionInterfaceObj>>,
    proxy: Option<Retained<ProtocolObject<dyn NSConnectionInterface + Send + Sync>>>,
    // if the connection is self contained (not depending on a listener service)
    // as is the case for client connections, a shutdown token is needed to keep
    // alive the XPC connection objects
    drop_guard: Option<DropGuard>,
    xpc_conn_invalidated: Arc<AtomicBool>,

    data_stream_rx: UnboundedReceiverStream<Vec<u8>>,
    to_be_copied: Option<Vec<u8>>,
}

impl XpcConnection {
    pub(crate) fn new(
        own_interface: Retained<ConnectionInterfaceObj>,
        proxy: Retained<ProtocolObject<dyn NSConnectionInterface + Send + Sync>>,
        data_stream_rx: UnboundedReceiverStream<Vec<u8>>,
        xpc_conn_invalidated: Arc<AtomicBool>,
    ) -> Self {
        XpcConnection {
            own_interface: Some(own_interface),
            proxy: Some(proxy),
            drop_guard: None,
            xpc_conn_invalidated,
            data_stream_rx,
            to_be_copied: None,
        }
    }

    pub(crate) fn with_drop_guard(mut self, drop_guard: DropGuard) -> Self {
        self.drop_guard = Some(drop_guard);
        self
    }

    // Tries to fill the destination buffer and returns true if it got filled
    // and there is not more data to be copied, and false if there's still left
    // data to be copied
    fn try_to_fill(&mut self, mut src: Vec<u8>, dst: &mut tokio::io::ReadBuf<'_>) -> bool {
        if dst.remaining() >= src.len() {
            // we can consume the entire data
            dst.put_slice(&src);
            false
        } else {
            // we have to store some of it for a later call
            self.to_be_copied = Some(src.split_off(dst.remaining()));
            dst.put_slice(&src);
            true
        }
    }

    fn underlaying_conn_invalidated(&mut self) -> bool {
        if self
            .xpc_conn_invalidated
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            // consume the object interface, which drops the UnboundedSender
            // so that the async reads of XpcConnection don't get left hanging
            self.own_interface.take();
            // consume the proxy object interface, since there's no point in
            // making RPC calls on a non existing connection
            self.proxy.take();
            true
        } else {
            false
        }
    }
}

impl AsyncRead for XpcConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // we check for invalidation, but we could still consume the buffered
        // data, so don't take any action on possible invalidation
        self.underlaying_conn_invalidated();
        if let Some(to_be_copied) = self.to_be_copied.take()
            && self.try_to_fill(to_be_copied, buf)
        {
            return std::task::Poll::Ready(Ok(()));
        }
        match Pin::new(&mut self.data_stream_rx).poll_next(cx) {
            std::task::Poll::Ready(data) => {
                if let Some(data) = data {
                    self.try_to_fill(data, buf);
                }
                std::task::Poll::Ready(Ok(()))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl AsyncWrite for XpcConnection {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        if self.underlaying_conn_invalidated() {
            return std::task::Poll::Ready(Err(std::io::ErrorKind::ConnectionReset.into()));
        }
        let Some(proxy) = self.proxy.as_ref() else {
            return std::task::Poll::Ready(Ok(0));
        };
        proxy.write(&NSData::with_bytes(buf));
        std::task::Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if self.underlaying_conn_invalidated() {
            return std::task::Poll::Ready(Err(std::io::ErrorKind::ConnectionReset.into()));
        }
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if self.underlaying_conn_invalidated() {
            return std::task::Poll::Ready(Err(std::io::ErrorKind::ConnectionReset.into()));
        }
        self.proxy.take();
        std::task::Poll::Ready(Ok(()))
    }
}

impl Connected for XpcConnection {
    type ConnectInfo = ();

    fn connect_info(&self) -> Self::ConnectInfo {}
}

#[cfg(test)]
mod tests {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        sync::mpsc,
    };
    use tokio_util::bytes::BytesMut;

    use super::*;

    #[tokio::test]
    async fn write_to_conn() {
        let (own_tx, own_rx) = mpsc::unbounded_channel();
        let own_interface = ConnectionInterfaceObj::new(own_tx.clone());
        let (remote_tx, mut remote_rx) = mpsc::unbounded_channel();
        let remote_proxy = unsafe {
            Retained::cast_unchecked::<ProtocolObject<dyn NSConnectionInterface + Send + Sync>>(
                ConnectionInterfaceObj::new(remote_tx),
            )
        };
        let mut own_conn = XpcConnection::new(
            own_interface,
            remote_proxy,
            own_rx.into(),
            Arc::new(AtomicBool::new(false)),
        );

        let data = vec![42];
        own_conn.write_all(&data).await.unwrap();
        assert_eq!(remote_rx.recv().await.unwrap(), data);
    }

    #[tokio::test]
    async fn read_from_conn() {
        let (own_tx, own_rx) = mpsc::unbounded_channel();
        let own_interface = ConnectionInterfaceObj::new(own_tx.clone());
        let (remote_tx, _remote_rx) = mpsc::unbounded_channel();
        let remote_proxy = unsafe {
            Retained::cast_unchecked::<ProtocolObject<dyn NSConnectionInterface + Send + Sync>>(
                ConnectionInterfaceObj::new(remote_tx),
            )
        };
        let mut own_conn = XpcConnection::new(
            own_interface,
            remote_proxy,
            own_rx.into(),
            Arc::new(AtomicBool::new(false)),
        );

        let data = vec![1, 2, 3, 4, 5];
        own_tx.send(data.clone()).unwrap();

        let mut buffer = BytesMut::with_capacity(2);
        own_conn.read_buf(&mut buffer).await.unwrap();
        assert_eq!(buffer, vec![1, 2]);

        // reactivate the poll
        own_tx.send(vec![]).unwrap();
        let mut buffer = BytesMut::with_capacity(2);
        own_conn.read_buf(&mut buffer).await.unwrap();
        assert_eq!(buffer, vec![3, 4]);

        // reactivate the poll
        own_tx.send(vec![]).unwrap();
        let mut buffer = BytesMut::with_capacity(2);
        own_conn.read_buf(&mut buffer).await.unwrap();
        assert_eq!(buffer, vec![5]);
    }
}
