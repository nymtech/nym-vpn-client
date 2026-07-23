// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{FutureExt, SinkExt, StreamExt, channel::mpsc};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fmt::Write,
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tarpc::{ClientMessage, Response};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::futures::Notified,
};
use tokio_util::codec::{Decoder, Encoder, Framed, LengthDelimitedCodec};

use crate::{Error, ServiceRequest, ServiceResponse};

/// How long to wait for the RPC server to start
const CONNECT_TIMEOUT: Duration = Duration::from_secs(300);
const FRAME_TYPE_SIZE: usize = std::mem::size_of::<FrameType>();
const DAEMON_CHANNEL_BUF_SIZE: usize = 16 * 1024;

/// Unique payload that comes with the "handshake" frame
const MULLVAD_SIGNATURE: &[u8] = b"MULLV4D;";

pub enum Frame {
    Handshake,
    TestRunner(Bytes),
    DaemonRpc(Bytes),
}

#[repr(u8)]
enum FrameType {
    Handshake,
    TestRunner,
    DaemonRpc,
}

impl TryFrom<u8> for FrameType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            i if i == FrameType::Handshake as u8 => Ok(FrameType::Handshake),
            i if i == FrameType::TestRunner as u8 => Ok(FrameType::TestRunner),
            i if i == FrameType::DaemonRpc as u8 => Ok(FrameType::DaemonRpc),
            _ => Err(()),
        }
    }
}

pub type GrpcForwarder = tokio::io::DuplexStream;
pub type CompletionHandle = tokio::task::JoinHandle<()>;

pub async fn synchronize_framed_session(
    framed: &mut Framed<GrpcForwarder, LengthDelimitedCodec>,
) -> io::Result<()> {
    framed.send(Bytes::new()).await?;

    loop {
        match framed.next().await {
            Some(Ok(bytes)) if bytes.is_empty() => return Ok(()),
            Some(Ok(_)) => {}
            Some(Err(error)) => return Err(error),
            None => return Err(io::ErrorKind::UnexpectedEof.into()),
        }
    }
}

pub async fn forward_framed_bidirectional<S>(
    stream: S,
    framed: &mut Framed<GrpcForwarder, LengthDelimitedCodec>,
) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (mut stream_reader, mut stream_writer) = tokio::io::split(stream);
    let (mut framed_sink, mut framed_stream) = framed.split();

    let stream_to_framed = async {
        let mut buffer = vec![0u8; DAEMON_CHANNEL_BUF_SIZE];
        loop {
            let read = stream_reader.read(&mut buffer).await;
            match read {
                Ok(num_bytes) => {
                    framed_sink
                        .send(Bytes::copy_from_slice(&buffer[..num_bytes]))
                        .await?;
                    if num_bytes == 0 {
                        return Ok(());
                    }
                }
                Err(error) => {
                    let _ = framed_sink.send(Bytes::new()).await;
                    return Err(error);
                }
            }
        }
    };

    let framed_to_stream = async {
        loop {
            match framed_stream.next().await {
                Some(Ok(bytes)) if bytes.is_empty() => {
                    stream_writer.shutdown().await?;
                    return Ok(());
                }
                Some(Ok(bytes)) => stream_writer.write_all(&bytes).await?,
                Some(Err(error)) => return Err(error),
                None => return Ok(()),
            }
        }
    };

    tokio::try_join!(stream_to_framed, framed_to_stream).map(|_| ())
}

#[derive(Debug, Clone)]
pub struct ConnectionHandle {
    handshake_fwd_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<()>>>,
    // True if the connection has received an initial "handshake" frame from the other end.
    is_connected: Arc<AtomicBool>,
    reset_notify: Arc<tokio::sync::Notify>,
}

impl ConnectionHandle {
    /// Returns a new "handshake forwarder" and connection handle.
    fn new() -> (mpsc::UnboundedSender<()>, Self) {
        let (handshake_fwd_tx, handshake_fwd_rx) = mpsc::unbounded();

        (
            handshake_fwd_tx,
            Self {
                handshake_fwd_rx: Arc::new(tokio::sync::Mutex::new(handshake_fwd_rx)),
                is_connected: Self::new_connected_state(false),
                reset_notify: Arc::new(tokio::sync::Notify::new()),
            },
        )
    }

    pub async fn wait_for_server(&mut self) -> Result<(), Error> {
        let mut handshake_fwd = self.handshake_fwd_rx.lock().await;

        log::info!("Waiting for server");

        match tokio::time::timeout(CONNECT_TIMEOUT, handshake_fwd.next()).await {
            Ok(_) => {
                log::info!("Server responded");
                Ok(())
            }
            _ => {
                log::error!("Connection timed out");
                Err(Error::TestRunnerTimeout)
            }
        }
    }

    /// Resets `Self::is_connected`.
    pub async fn reset_connected_state(&self) {
        let mut handshake_fwd = self.handshake_fwd_rx.lock().await;
        // empty stream
        while handshake_fwd.try_recv().is_ok() {}

        self.is_connected.store(false, Ordering::SeqCst);
        self.reset_notify.notify_waiters();
    }

    /// Returns a future that is notified when `reset_connected_state` is called.
    pub fn notified_reset(&self) -> Notified<'_> {
        self.reset_notify.notified()
    }

    fn connected_state(&self) -> Arc<AtomicBool> {
        self.is_connected.clone()
    }

    fn new_connected_state(initial: bool) -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(initial))
    }
}

type ServerTransports = (
    tarpc::transport::channel::UnboundedChannel<
        ClientMessage<ServiceRequest>,
        Response<ServiceResponse>,
    >,
    GrpcForwarder,
    CompletionHandle,
);

pub fn create_server_transports(
    serial_stream: impl AsyncRead + AsyncWrite + Unpin + Send + 'static,
) -> ServerTransports {
    let (runner_forwarder_1, runner_forwarder_2) = tarpc::transport::channel::unbounded();

    let (daemon_rx, mullvad_daemon_forwarder) = tokio::io::duplex(DAEMON_CHANNEL_BUF_SIZE);

    let (handshake_tx, handshake_rx) = mpsc::unbounded();

    let _ = handshake_tx.unbounded_send(());

    let completion_handle = tokio::spawn(async move {
        if let Err(error) = forward_messages(
            serial_stream,
            runner_forwarder_2,
            mullvad_daemon_forwarder,
            (handshake_tx, handshake_rx),
            None,
            // The server needs to be init to connected, or it will skip things it shouldn't
            ConnectionHandle::new_connected_state(true),
        )
        .await
        {
            log::error!(
                "forward_messages stopped due an error: {}",
                display_chain(error)
            );
        } else {
            log::debug!("forward_messages stopped");
        }
    });

    (runner_forwarder_1, daemon_rx, completion_handle)
}

/// Should work for either Mullvad or Nym
pub fn create_client_transports(
    serial_stream: impl AsyncRead + AsyncWrite + Unpin + Send + 'static,
) -> Result<ClientTransports, Error> {
    let (runner_forwarder_1, runner_forwarder_2) = tarpc::transport::channel::unbounded();

    let (daemon_rx, daemon_forwarder) = tokio::io::duplex(DAEMON_CHANNEL_BUF_SIZE);

    let (handshake_tx, handshake_rx) = mpsc::unbounded();

    let (handshake_fwd_tx, conn_handle) = ConnectionHandle::new();

    let _ = handshake_tx.unbounded_send(());

    let connected_state = conn_handle.connected_state();

    let completion_handle = tokio::spawn(async move {
        if let Err(error) = forward_messages(
            serial_stream,
            runner_forwarder_1,
            daemon_forwarder,
            (handshake_tx, handshake_rx),
            Some(handshake_fwd_tx),
            connected_state,
        )
        .await
        {
            log::error!(
                "forward_messages stopped due an error: {}",
                display_chain(error)
            );
        } else {
            log::debug!("forward_messages stopped");
        }
    });

    Ok((
        runner_forwarder_2,
        daemon_rx,
        conn_handle,
        completion_handle,
    ))
}

type ClientTransports = (
    tarpc::transport::channel::UnboundedChannel<
        Response<ServiceResponse>,
        ClientMessage<ServiceRequest>,
    >,
    GrpcForwarder,
    ConnectionHandle,
    CompletionHandle,
);

#[derive(thiserror::Error, Debug)]
enum ForwardError {
    #[error("Failed to deserialize JSON data")]
    DeserializeFailed(#[source] serde_json::Error),

    #[error("Failed to serialize JSON data")]
    SerializeFailed(#[source] serde_json::Error),

    #[error("Serial connection error")]
    SerialConnection(#[source] io::Error),

    #[error("Test runner channel error")]
    TestRunnerChannel(#[source] tarpc::transport::channel::ChannelError),

    #[error("Daemon channel error")]
    DaemonChannel(#[source] io::Error),

    #[error("Handshake error")]
    HandshakeError(#[source] io::Error),

    #[error("{0} forwarding queue closed")]
    ForwardQueueClosed(&'static str),
}

async fn forward_messages<
    T: Serialize + Unpin + Send + 'static,
    S: DeserializeOwned + Unpin + Send + 'static,
>(
    serial_stream: impl AsyncRead + AsyncWrite + Unpin + Send + 'static,
    runner_forwarder: tarpc::transport::channel::UnboundedChannel<T, S>,
    daemon_forwarder: GrpcForwarder,
    mut handshaker: (mpsc::UnboundedSender<()>, mpsc::UnboundedReceiver<()>),
    handshake_fwd: Option<mpsc::UnboundedSender<()>>,
    connected_state: Arc<AtomicBool>,
) -> Result<(), ForwardError> {
    let codec = MultiplexCodec::new(connected_state);
    let serial_stream = codec.framed(serial_stream);
    let (mut serial_sink, mut serial_source) = serial_stream.split();

    // Needs to be framed to allow empty messages.
    let daemon_forwarder = LengthDelimitedCodec::new().framed(daemon_forwarder);
    let (mut daemon_sink, mut daemon_source) = daemon_forwarder.split();
    let (mut runner_sink, mut runner_source) = runner_forwarder.split();
    let (runner_tx, mut runner_rx) = mpsc::unbounded();
    let (daemon_tx, mut daemon_rx) = mpsc::unbounded();

    let serial_reader = async move {
        while let Some(frame) = serial_source.next().await {
            match frame.map_err(ForwardError::SerialConnection)? {
                Frame::TestRunner(data) => {
                    let message =
                        serde_json::from_slice(&data).map_err(ForwardError::DeserializeFailed)?;
                    runner_tx
                        .unbounded_send(message)
                        .map_err(|_| ForwardError::ForwardQueueClosed("test runner"))?;
                }
                Frame::DaemonRpc(data) => {
                    daemon_tx
                        .unbounded_send(data)
                        .map_err(|_| ForwardError::ForwardQueueClosed("daemon"))?;
                }
                Frame::Handshake => {
                    log::trace!("shake: recv");
                    if let Some(shake_fwd) = handshake_fwd.as_ref() {
                        let _ = shake_fwd.unbounded_send(());
                    } else {
                        let _ = handshaker.0.unbounded_send(());
                    }
                }
            }
        }
        Ok(())
    };

    let runner_writer = async move {
        while let Some(message) = runner_rx.next().await {
            runner_sink
                .send(message)
                .await
                .map_err(ForwardError::TestRunnerChannel)?;
        }
        Ok(())
    };

    let daemon_writer = async move {
        while let Some(data) = daemon_rx.next().await {
            daemon_sink
                .send(data)
                .await
                .map_err(ForwardError::DaemonChannel)?;
        }
        Ok(())
    };

    let serial_writer = async move {
        loop {
            futures::select! {
                handshake = handshaker.1.next().fuse() => {
                    if handshake.is_none() {
                        break Ok(());
                    }

                    log::trace!("shake: send");
                    serial_sink
                        .send(Frame::Handshake)
                        .await
                        .map_err(ForwardError::HandshakeError)?;
                }

                message = runner_source.next().fuse() => {
                    let Some(message) = message else {
                        break Ok(());
                    };
                    let message = message.map_err(ForwardError::TestRunnerChannel)?;
                    let serialized =
                        serde_json::to_vec(&message).map_err(ForwardError::SerializeFailed)?;
                    serial_sink
                        .send(Frame::TestRunner(serialized.into()))
                        .await
                        .map_err(ForwardError::SerialConnection)?;
                }

                data = daemon_source.next().fuse() => {
                    let Some(data) = data else {
                        let _ = serial_sink.send(Frame::DaemonRpc(Bytes::new())).await;
                        break Ok(());
                    };
                    let data = data.map_err(ForwardError::DaemonChannel)?;
                    serial_sink
                        .send(Frame::DaemonRpc(data.into()))
                        .await
                        .map_err(ForwardError::SerialConnection)?;
                }
            }
        }
    };

    tokio::select! {
        result = serial_reader => result,
        result = runner_writer => result,
        result = daemon_writer => result,
        result = serial_writer => result,
    }
}

const MULTIPLEX_LEN_DELIMITED_HEADER_SIZE: usize = 4;

#[derive(Default, Debug, Clone)]
pub struct MultiplexCodec {
    len_delim_codec: LengthDelimitedCodec,
    has_connected: Arc<AtomicBool>,
}

impl MultiplexCodec {
    fn new(has_connected: Arc<AtomicBool>) -> Self {
        let mut codec_builder = LengthDelimitedCodec::builder();

        codec_builder
            .length_field_length(MULTIPLEX_LEN_DELIMITED_HEADER_SIZE)
            .max_frame_length(usize::MAX);

        Self {
            has_connected,
            len_delim_codec: codec_builder.new_codec(),
        }
    }

    fn decode_frame(mut frame: BytesMut) -> Result<Frame, io::Error> {
        if frame.len() < FRAME_TYPE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "frame does not contain frame type",
            ));
        }

        let mut type_bytes = frame.split_to(FRAME_TYPE_SIZE);
        let frame_type = FrameType::try_from(type_bytes.get_u8())
            .map_err(|_err| io::Error::new(io::ErrorKind::InvalidInput, "invalid frame type"))?;

        match frame_type {
            FrameType::Handshake => Ok(Frame::Handshake),
            FrameType::TestRunner => Ok(Frame::TestRunner(frame.into())),
            FrameType::DaemonRpc => Ok(Frame::DaemonRpc(frame.into())),
        }
    }

    fn encode_frame(
        &mut self,
        frame_type: FrameType,
        bytes: Option<Bytes>,
        dst: &mut BytesMut,
    ) -> Result<(), io::Error> {
        let mut buffer = BytesMut::new();
        if let Some(bytes) = bytes {
            buffer.reserve(bytes.len() + FRAME_TYPE_SIZE);
            buffer.put_u8(frame_type as u8);
            // TODO: implement without copying
            buffer.put(&bytes[..]);
        } else {
            buffer.reserve(FRAME_TYPE_SIZE);
            buffer.put_u8(frame_type as u8);
        }
        self.len_delim_codec.encode(buffer.into(), dst)
    }

    fn decode_inner(&mut self, src: &mut BytesMut) -> Result<Option<Frame>, io::Error> {
        self.skip_noise(src);
        if !self.has_connected.load(Ordering::SeqCst) {
            return Ok(None);
        }
        let frame = self.len_delim_codec.decode(src)?;
        frame.map(Self::decode_frame).transpose()
    }

    fn skip_noise(&mut self, src: &mut BytesMut) {
        // The test runner likes to send ^@ once in while. Unclear why,
        // but it probably occurs (sometimes) when it reconnects to the
        // serial device. Ignoring these control characters is safe.
        while src.len() >= 2 {
            if src[0] == b'^' {
                log::debug!("ignoring control character");
                src.advance(2);
                continue;
            }

            // We use a magic constant to ignore any garbage sent before
            // our service starts. The reason is that OVMF sends stuff to
            // our serial device that we don't care about.
            if !self.has_connected.load(Ordering::SeqCst) {
                for (window_i, window) in src.windows(MULLVAD_SIGNATURE.len()).enumerate() {
                    if window == MULLVAD_SIGNATURE {
                        log::debug!("Found conn signature");

                        // Skip to where the first frame begins
                        src.advance(
                            window_i
                                .saturating_sub(FRAME_TYPE_SIZE)
                                .saturating_sub(MULTIPLEX_LEN_DELIMITED_HEADER_SIZE),
                        );

                        self.has_connected.store(true, Ordering::SeqCst);

                        break;
                    }
                }
            }

            break;
        }
    }
}

impl Decoder for MultiplexCodec {
    type Item = Frame;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decode_inner(src)
    }
}

impl Encoder<Frame> for MultiplexCodec {
    type Error = io::Error;

    fn encode(&mut self, frame: Frame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match frame {
            Frame::Handshake => self.encode_frame(
                FrameType::Handshake,
                Some(Bytes::from_static(MULLVAD_SIGNATURE)),
                dst,
            ),
            Frame::TestRunner(bytes) => self.encode_frame(FrameType::TestRunner, Some(bytes), dst),
            Frame::DaemonRpc(bytes) => self.encode_frame(FrameType::DaemonRpc, Some(bytes), dst),
        }
    }
}

fn display_chain(error: impl std::error::Error) -> String {
    let mut s = error.to_string();
    let mut error = &error as &dyn std::error::Error;
    while let Some(source) = error.source() {
        write!(&mut s, "\nCaused by: {source}").unwrap();
        error = source;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::{
        DAEMON_CHANNEL_BUF_SIZE, create_client_transports, create_server_transports,
        forward_framed_bidirectional, synchronize_framed_session,
    };
    use bytes::Bytes;
    use futures::{SinkExt, StreamExt};
    use std::{io, time::Duration};
    use tokio::io::{AsyncReadExt, AsyncWriteExt, duplex};
    use tokio_util::codec::{Decoder, LengthDelimitedCodec};

    #[tokio::test]
    async fn stream_eof_waits_for_framed_eof_acknowledgement() {
        let (bridge_stream, stream_peer) = duplex(64);
        let (mut stream_peer_reader, mut stream_peer_writer) = tokio::io::split(stream_peer);
        let (bridge_framed_io, framed_peer_io) = duplex(64);
        let mut bridge_framed = LengthDelimitedCodec::new().framed(bridge_framed_io);
        let mut framed_peer = LengthDelimitedCodec::new().framed(framed_peer_io);

        let bridge = tokio::spawn(async move {
            forward_framed_bidirectional(bridge_stream, &mut bridge_framed).await
        });

        stream_peer_writer
            .shutdown()
            .await
            .expect("close stream input");
        let eof = framed_peer
            .next()
            .await
            .expect("framed bridge remains open")
            .expect("EOF frame is valid");
        assert!(eof.is_empty());
        assert!(!bridge.is_finished());

        framed_peer
            .send(Bytes::new())
            .await
            .expect("acknowledge stream EOF");
        assert_eq!(
            stream_peer_reader.read_u8().await.unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
        bridge.await.expect("bridge task").expect("clean shutdown");
    }

    #[tokio::test]
    async fn framed_eof_waits_for_stream_eof_acknowledgement() {
        let (bridge_stream, stream_peer) = duplex(64);
        let (mut stream_peer_reader, mut stream_peer_writer) = tokio::io::split(stream_peer);
        let (bridge_framed_io, framed_peer_io) = duplex(64);
        let mut bridge_framed = LengthDelimitedCodec::new().framed(bridge_framed_io);
        let mut framed_peer = LengthDelimitedCodec::new().framed(framed_peer_io);

        let bridge = tokio::spawn(async move {
            forward_framed_bidirectional(bridge_stream, &mut bridge_framed).await
        });

        framed_peer
            .send(Bytes::new())
            .await
            .expect("send framed EOF");
        assert_eq!(
            stream_peer_reader.read_u8().await.unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
        assert!(!bridge.is_finished());

        stream_peer_writer
            .shutdown()
            .await
            .expect("acknowledge framed EOF");
        let eof = framed_peer
            .next()
            .await
            .expect("framed bridge remains open")
            .expect("EOF acknowledgement is valid");
        assert!(eof.is_empty());
        bridge.await.expect("bridge task").expect("clean shutdown");
    }

    #[tokio::test]
    async fn session_sync_discards_delayed_frames_before_acknowledgement() {
        let (bridge_io, peer_io) = duplex(64);
        let mut bridge = LengthDelimitedCodec::new().framed(bridge_io);
        let mut peer = LengthDelimitedCodec::new().framed(peer_io);

        bridge
            .send(Bytes::from_static(b"stale request"))
            .await
            .expect("queue stale request");
        let sync = tokio::spawn(async move {
            synchronize_framed_session(&mut bridge)
                .await
                .map(|()| bridge)
        });

        assert_eq!(
            peer.next()
                .await
                .expect("stale request")
                .expect("valid frame"),
            Bytes::from_static(b"stale request")
        );
        assert!(
            peer.next()
                .await
                .expect("sync marker")
                .expect("valid frame")
                .is_empty()
        );
        peer.send(Bytes::from_static(b"stale response"))
            .await
            .expect("send delayed stale response");
        peer.send(Bytes::new())
            .await
            .expect("acknowledge synchronization");

        let mut bridge = sync
            .await
            .expect("sync task")
            .expect("session synchronized");
        assert!(
            tokio::time::timeout(Duration::from_millis(10), bridge.next())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn session_sync_flushes_a_cancelled_partial_frame_before_acknowledgement() {
        let (bridge_io, peer_io) = duplex(64);
        let mut bridge = LengthDelimitedCodec::new().framed(bridge_io);
        let mut peer = LengthDelimitedCodec::new().framed(peer_io);
        let stale_request = Bytes::from(vec![9; 4096]);

        let mut cancelled_send = Box::pin(bridge.send(stale_request.clone()));
        assert!(futures::poll!(&mut cancelled_send).is_pending());
        drop(cancelled_send);

        let sync = tokio::spawn(async move {
            synchronize_framed_session(&mut bridge)
                .await
                .map(|()| bridge)
        });

        assert_eq!(
            peer.next()
                .await
                .expect("cancelled frame")
                .expect("valid frame"),
            stale_request
        );
        assert!(
            peer.next()
                .await
                .expect("sync marker")
                .expect("valid frame")
                .is_empty()
        );
        peer.send(Bytes::new())
            .await
            .expect("acknowledge synchronization");

        sync.await
            .expect("sync task")
            .expect("session synchronized");
    }

    #[tokio::test]
    async fn blocked_response_does_not_block_request_forwarding() {
        let (bridge_stream, mut stream_peer) = duplex(64);
        let (bridge_framed_io, framed_peer_io) = duplex(64);
        let mut bridge_framed = LengthDelimitedCodec::new().framed(bridge_framed_io);
        let mut framed_peer = LengthDelimitedCodec::new().framed(framed_peer_io);

        let bridge = tokio::spawn(async move {
            forward_framed_bidirectional(bridge_stream, &mut bridge_framed).await
        });

        framed_peer
            .send(Bytes::from(vec![7; 4096]))
            .await
            .expect("queue response that blocks on the unread peer");
        tokio::task::yield_now().await;

        stream_peer
            .write_all(b"request")
            .await
            .expect("write request while response direction is blocked");
        let forwarded = tokio::time::timeout(Duration::from_secs(1), framed_peer.next())
            .await
            .expect("request forwarding must not share response backpressure")
            .expect("framed bridge remains open")
            .expect("request frame is valid");

        assert_eq!(&forwarded[..], b"request");
        bridge.abort();
    }

    #[tokio::test]
    async fn blocked_daemon_destination_does_not_block_opposite_direction() {
        let (client_serial, server_serial) = duplex(64 * 1024);
        let (_client_runner, client_daemon, mut connection, client_task) =
            create_client_transports(client_serial).expect("create client transports");
        let (_server_runner, server_daemon, server_task) = create_server_transports(server_serial);
        let mut client_daemon = LengthDelimitedCodec::new().framed(client_daemon);
        let mut server_daemon = LengthDelimitedCodec::new().framed(server_daemon);

        connection
            .wait_for_server()
            .await
            .expect("complete transport handshake");

        client_daemon
            .send(Bytes::from(vec![7; DAEMON_CHANNEL_BUF_SIZE]))
            .await
            .expect("fill the unread server daemon destination");
        tokio::task::yield_now().await;

        server_daemon
            .send(Bytes::from_static(b"opposite direction"))
            .await
            .expect("queue opposite-direction daemon traffic");
        let forwarded = tokio::time::timeout(Duration::from_secs(1), client_daemon.next())
            .await
            .expect("opposite direction must not share destination backpressure")
            .expect("client daemon transport remains open")
            .expect("opposite-direction frame is valid");

        assert_eq!(&forwarded[..], b"opposite direction");
        client_task.abort();
        server_task.abort();
    }

    #[tokio::test]
    async fn blocked_daemon_destination_does_not_block_runner_demux() {
        let (client_serial, server_serial) = duplex(64 * 1024);
        let (client_runner, client_daemon, mut connection, client_task) =
            create_client_transports(client_serial).expect("create client transports");
        let (mut server_runner, server_daemon, server_task) =
            create_server_transports(server_serial);
        let mut client_daemon = LengthDelimitedCodec::new().framed(client_daemon);
        // Leave the server daemon unread so destination writes back up into the mux.
        let _blocked_server_daemon = LengthDelimitedCodec::new().framed(server_daemon);

        connection
            .wait_for_server()
            .await
            .expect("complete transport handshake");

        let flood = tokio::spawn(async move {
            for _ in 0..4 {
                if client_daemon
                    .send(Bytes::from(vec![7; DAEMON_CHANNEL_BUF_SIZE]))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            // Keep the daemon half alive until the assertion finishes; dropping it
            // closes the multiplexed session.
            client_daemon
        });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client =
            crate::service::ServiceClient::new(tarpc::client::Config::default(), client_runner)
                .spawn();
        let _request = tokio::spawn(async move {
            let _ = client
                .get_default_interface(tarpc::context::current())
                .await;
        });

        let received = tokio::time::timeout(Duration::from_secs(1), server_runner.next())
            .await
            .expect("runner demux must not share daemon destination backpressure")
            .expect("server runner transport remains open")
            .expect("runner frame is valid");

        assert!(matches!(received, tarpc::ClientMessage::Request(_)));
        flood.abort();
        client_task.abort();
        server_task.abort();
    }
}
