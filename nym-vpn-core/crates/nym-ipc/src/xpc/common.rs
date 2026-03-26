// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::pin::Pin;

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
use tokio_util::sync::{CancellationToken, DropGuard};
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
    proxy: Option<Retained<ProtocolObject<dyn NSConnectionInterface + Send + Sync>>>,
    // if the connection is self contained (not depending on a listener service)
    // as is the case for client connections, a shutdown token is needed to keep
    // alive the XPC connection objects
    drop_guard: Option<DropGuard>,
    shutdown_token: CancellationToken,

    data_stream_rx: UnboundedReceiverStream<Vec<u8>>,
    to_be_copied: Option<Vec<u8>>,
}

impl XpcConnection {
    pub(crate) fn new(
        proxy: Retained<ProtocolObject<dyn NSConnectionInterface + Send + Sync>>,
        data_stream_rx: UnboundedReceiverStream<Vec<u8>>,
        shutdown_token: CancellationToken,
    ) -> Self {
        XpcConnection {
            proxy: Some(proxy),
            drop_guard: None,
            shutdown_token,
            data_stream_rx,
            to_be_copied: None,
        }
    }

    pub(crate) fn with_drop_guard(mut self, drop_guard: DropGuard) -> Self {
        self.drop_guard = Some(drop_guard);
        self
    }

    // Tries to fill the destination buffer
    fn try_to_fill(&mut self, mut src: Vec<u8>, dst: &mut tokio::io::ReadBuf<'_>) {
        if dst.remaining() >= src.len() {
            // we can consume the entire data
            dst.put_slice(&src);
        } else {
            // we have to store some of it for a later call
            self.to_be_copied = Some(src.split_off(dst.remaining()));
            dst.put_slice(&src);
        }
    }

    fn underlaying_conn_invalidated(&mut self) -> bool {
        if self.shutdown_token.is_cancelled() {
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
        if let Some(to_be_copied) = self.to_be_copied.take() {
            self.try_to_fill(to_be_copied, buf);
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
    use std::time::Duration;

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt, ReadBuf},
        sync::mpsc,
    };

    use super::*;

    #[tokio::test]
    async fn write_to_conn() {
        let (own_tx, own_rx) = mpsc::unbounded_channel();
        let _own_interface = ConnectionInterfaceObj::new(own_tx.clone());
        let (remote_tx, mut remote_rx) = mpsc::unbounded_channel();
        let remote_proxy = unsafe {
            Retained::cast_unchecked::<ProtocolObject<dyn NSConnectionInterface + Send + Sync>>(
                ConnectionInterfaceObj::new(remote_tx),
            )
        };
        let mut own_conn =
            XpcConnection::new(remote_proxy, own_rx.into(), CancellationToken::new());

        let data = vec![42];
        own_conn.write_all(&data).await.unwrap();
        assert_eq!(remote_rx.recv().await.unwrap(), data);
    }

    // simulate 100 gRPC calls of 100 bytes per call, and the same 8KB buffer reused until full,
    // at which point it gets re-created
    // this behavior follows what was observed when running 100 calls with vpnc
    // reproducing bug NYM-967 where the second read would hang even though the last piece of data
    // was being written into the buffer, but Pending was being returned
    #[tokio::test]
    async fn fixed_buf_multiple_big_sends() {
        let (own_tx, own_rx) = mpsc::unbounded_channel();
        let _own_interface = ConnectionInterfaceObj::new(own_tx.clone());
        let (remote_tx, _remote_rx) = mpsc::unbounded_channel();
        let remote_proxy = unsafe {
            Retained::cast_unchecked::<ProtocolObject<dyn NSConnectionInterface + Send + Sync>>(
                ConnectionInterfaceObj::new(remote_tx),
            )
        };
        let mut own_conn =
            XpcConnection::new(remote_proxy, own_rx.into(), CancellationToken::new());

        let fut = async move {
            let data: Vec<u8> = (1u8..=100).collect();
            let mut read_buf = [0; 8192];
            let mut buffer = ReadBuf::new(&mut read_buf);
            for _ in 0..100 {
                own_tx.send(data.clone()).unwrap();

                let remaining = buffer.remaining();
                if remaining < 100 {
                    own_conn.read_buf(&mut buffer).await.unwrap();
                    buffer = ReadBuf::new(&mut read_buf);
                    own_conn.read_buf(&mut buffer).await.unwrap();
                } else {
                    own_conn.read_buf(&mut buffer).await.unwrap();
                }
            }
        };
        // if a read happens correctly but the data is written without returning Ready,
        // the read call will stall and the timeout will trigger
        tokio::time::timeout(Duration::from_secs(1), fut)
            .await
            .unwrap();
    }

    // simulate 100 gRPC calls of 50 + 50 bytes per call, and the same 8KB buffer reused until full,
    // at which point it gets re-created
    // this behavior follows what was observed when running 100 calls with vpnc
    // reproducing bug NYM-967 where the third read when having buffer capacity >50 and <100 would not only take
    // the data from the previous send (which was addressed to the current read iteration) but also data from the
    // next iteration's write. The overlap caused corruption of data
    #[tokio::test]
    async fn fixed_buf_multiple_couple_sends() {
        const READ_BUF_SIZE: usize = 8192;
        let (own_tx, own_rx) = mpsc::unbounded_channel();
        let _own_interface = ConnectionInterfaceObj::new(own_tx.clone());
        let (remote_tx, _remote_rx) = mpsc::unbounded_channel();
        let remote_proxy = unsafe {
            Retained::cast_unchecked::<ProtocolObject<dyn NSConnectionInterface + Send + Sync>>(
                ConnectionInterfaceObj::new(remote_tx),
            )
        };
        let mut own_conn =
            XpcConnection::new(remote_proxy, own_rx.into(), CancellationToken::new());

        tokio::spawn(async move {
            let data1: Vec<u8> = (1u8..=50).collect();
            let data2: Vec<u8> = (51u8..=100).collect();
            for _ in 0..100 {
                own_tx.send(data1.clone()).unwrap();
                own_tx.send(data2.clone()).unwrap();
            }
        })
        .await
        .unwrap();
        let fut = async move {
            let mut read_buf = [0; READ_BUF_SIZE];
            // use a double sized buffer to more easily verify data that otherwise
            // gets wrapped and written to the beginning of a new buffer
            let mut expected_buf = [0u8; 2 * READ_BUF_SIZE].to_vec();
            let mut reinited = false;
            let mut buffer = ReadBuf::new(&mut read_buf);
            for i in 0..100 {
                // compute and place the 1, 2, ..., 100 bytes we expect to get
                // in this iteration
                for idx in 0..100 {
                    expected_buf[100 * i + idx] = (idx % 100 + 1) as u8;
                }

                let remaining = buffer.remaining();
                if remaining < 50 {
                    // this branch doesn't happen in this case because 8192 % 100 = 92 > 50
                    // but we'll just keep it around for completion
                    own_conn.read_buf(&mut buffer).await.unwrap();

                    assert_eq!(buffer.initialized(), &expected_buf[..READ_BUF_SIZE]);
                    read_buf = [0; READ_BUF_SIZE];
                    buffer = ReadBuf::new(&mut read_buf);
                    reinited = true;
                    own_conn.read_buf(&mut buffer).await.unwrap();
                } else if remaining < 100 {
                    // we read the first write correctly, the second write gets partially read...
                    own_conn.read_buf(&mut buffer).await.unwrap();
                    own_conn.read_buf(&mut buffer).await.unwrap();

                    // we verify the first half before moving to the next wrapped partial read..
                    assert_eq!(buffer.initialized(), &expected_buf[..READ_BUF_SIZE]);
                    read_buf = [0; READ_BUF_SIZE];
                    buffer = ReadBuf::new(&mut read_buf);
                    reinited = true;
                    // we read the remaining part of the second write
                    own_conn.read_buf(&mut buffer).await.unwrap();
                } else {
                    // reads and writes have enough buffer space to be matched
                    own_conn.read_buf(&mut buffer).await.unwrap();
                    own_conn.read_buf(&mut buffer).await.unwrap();
                }
                if reinited {
                    // the first half of the buffer has been completely verified, it's time we verify the second half
                    assert_eq!(buffer.initialized(), &expected_buf[READ_BUF_SIZE..]);
                } else {
                    // we haven't reached the end of the initial 8KB buffer, so we only verify the first half
                    assert_eq!(buffer.initialized(), &expected_buf[..READ_BUF_SIZE]);
                }
            }
        };
        // not needed in this test but good to have a timeout just in case
        tokio::time::timeout(Duration::from_secs(1), fut)
            .await
            .unwrap();
    }
}
