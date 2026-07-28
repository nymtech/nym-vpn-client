// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{SinkExt, StreamExt, channel::mpsc};
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
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::{Error, ServiceRequest, ServiceResponse};

/// How long to wait for the RPC server to start
const CONNECT_TIMEOUT: Duration = Duration::from_secs(300);
const FRAME_TYPE_SIZE: usize = std::mem::size_of::<FrameType>();
const DAEMON_CHANNEL_BUF_SIZE: usize = 16 * 1024;

/// Unique payload that comes with the "handshake" frame
const MULLVAD_SIGNATURE: &[u8] = b"MULLV4D;";

/// Cap resync byte-skipping per decode call so pathological noise cannot spin forever.
const MAX_RESYNC_BYTES_PER_DECODE: usize = 64 * 1024;

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

/// Independent length-delimited halves. Prefer over `Framed::split` (shared BiLock).
pub fn length_delimited_framed_halves<T>(
    io: T,
) -> (
    FramedRead<tokio::io::ReadHalf<T>, LengthDelimitedCodec>,
    FramedWrite<tokio::io::WriteHalf<T>, LengthDelimitedCodec>,
)
where
    T: AsyncRead + AsyncWrite,
{
    let (reader, writer) = tokio::io::split(io);
    let codec = LengthDelimitedCodec::new();
    (
        FramedRead::new(reader, codec.clone()),
        FramedWrite::new(writer, codec),
    )
}

pub const SESSION_SYNC_PING: &[u8] = b"NYMSESSIONSYNC;PING";
pub const SESSION_SYNC_ACK: &[u8] = b"NYMSESSIONSYNC;ACK";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForwardOutcome {
    Eof,
    SessionRestart,
}

pub async fn synchronize_framed_session<R, W>(
    framed_read: &mut FramedRead<R, LengthDelimitedCodec>,
    framed_write: &mut FramedWrite<W, LengthDelimitedCodec>,
) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    framed_write
        .send(Bytes::from_static(SESSION_SYNC_PING))
        .await?;

    loop {
        match framed_read.next().await {
            Some(Ok(bytes)) if bytes == SESSION_SYNC_ACK => return Ok(()),
            Some(Ok(bytes)) => {
                log::warn!(
                    "session sync: discarding unexpected frame ({} bytes) before ack",
                    bytes.len()
                );
            }
            Some(Err(error)) => return Err(error),
            None => return Err(io::ErrorKind::UnexpectedEof.into()),
        }
    }
}

pub async fn forward_framed_bidirectional<S, R, W>(
    stream: S,
    framed_read: &mut FramedRead<R, LengthDelimitedCodec>,
    framed_write: &mut FramedWrite<W, LengthDelimitedCodec>,
) -> io::Result<ForwardOutcome>
where
    S: AsyncRead + AsyncWrite + Unpin,
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let (mut stream_reader, mut stream_writer) = tokio::io::split(stream);

    let mut stream_to_framed = std::pin::pin!(async {
        let mut buffer = vec![0u8; DAEMON_CHANNEL_BUF_SIZE];
        loop {
            let read = stream_reader.read(&mut buffer).await;
            match read {
                Ok(num_bytes) => {
                    framed_write
                        .send(Bytes::copy_from_slice(&buffer[..num_bytes]))
                        .await?;
                    log::trace!("fwd[stream→framed] bytes={num_bytes}");
                    if num_bytes == 0 {
                        return Ok(());
                    }
                }
                Err(error) => {
                    let _ = framed_write.send(Bytes::new()).await;
                    return Err(error);
                }
            }
        }
    });

    let mut framed_to_stream = std::pin::pin!(async {
        loop {
            match framed_read.next().await {
                Some(Ok(bytes)) if bytes == SESSION_SYNC_PING => {
                    log::debug!("fwd[framed→stream] peer requested a new session");
                    let _ = stream_writer.shutdown().await;
                    return Ok(ForwardOutcome::SessionRestart);
                }
                Some(Ok(bytes)) if bytes == SESSION_SYNC_ACK => {
                    log::warn!("fwd[framed→stream] discarding stale session sync ack");
                }
                Some(Ok(bytes)) if bytes.is_empty() => {
                    log::trace!("fwd[framed→stream] bytes=0");
                    stream_writer.shutdown().await?;
                    return Ok(ForwardOutcome::Eof);
                }
                Some(Ok(bytes)) => {
                    stream_writer.write_all(&bytes).await?;
                    log::trace!("fwd[framed→stream] bytes={}", bytes.len());
                }
                Some(Err(error)) => return Err(error),
                None => return Ok(ForwardOutcome::Eof),
            }
        }
    });

    // A restart abandons the stream half rather than waiting for its EOF: the peer
    // owns the session now, and a hung stream must not stall the next session.
    tokio::select! {
        outcome = &mut framed_to_stream => match outcome? {
            ForwardOutcome::SessionRestart => Ok(ForwardOutcome::SessionRestart),
            ForwardOutcome::Eof => (&mut stream_to_framed).await.map(|()| ForwardOutcome::Eof),
        },
        result = &mut stream_to_framed => {
            result?;
            (&mut framed_to_stream).await
        }
    }
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
    ///
    /// Only safe around reboot: while `is_connected` is false the serial mux
    /// stops decoding frames (including tarpc). Prefer [`Self::abort_active_forward`]
    /// for mid-suite DaemonRpc recovery.
    pub async fn reset_connected_state(&self) {
        let mut handshake_fwd = self.handshake_fwd_rx.lock().await;
        // empty stream
        while handshake_fwd.try_recv().is_ok() {}

        self.is_connected.store(false, Ordering::SeqCst);
        self.reset_notify.notify_waiters();
    }

    /// Abort the host DaemonRpc forward loop's current session without clearing
    /// mux handshake state (so tarpc keeps decoding).
    pub fn abort_active_forward(&self) {
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
    let (serial_reader, serial_writer) = tokio::io::split(serial_stream);
    let mut serial_source = FramedRead::new(serial_reader, codec.clone());
    let mut serial_sink = FramedWrite::new(serial_writer, codec);

    // Needs to be framed to allow empty messages.
    let (daemon_reader, daemon_writer) = tokio::io::split(daemon_forwarder);
    let daemon_codec = LengthDelimitedCodec::new();
    let mut daemon_source = FramedRead::new(daemon_reader, daemon_codec.clone());
    let mut daemon_sink = FramedWrite::new(daemon_writer, daemon_codec);
    let (mut runner_sink, mut runner_source) = runner_forwarder.split();
    let (runner_tx, mut runner_rx) = mpsc::unbounded();
    let (daemon_tx, mut daemon_rx) = mpsc::unbounded();

    let serial_reader = async move {
        while let Some(frame) = serial_source.next().await {
            match frame.map_err(ForwardError::SerialConnection)? {
                Frame::TestRunner(data) => {
                    log::trace!("serial[wire→runner] bytes={}", data.len());
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

    // Prefer handshake, then tarpc, then daemon gRPC. Daemon EOF must not tear down tarpc.
    let serial_writer = async move {
        let mut daemon_open = true;
        loop {
            tokio::select! {
                biased;

                handshake = handshaker.1.next() => {
                    if handshake.is_none() {
                        break Ok(());
                    }

                    log::trace!("shake: send");
                    serial_sink
                        .send(Frame::Handshake)
                        .await
                        .map_err(ForwardError::HandshakeError)?;
                }

                message = runner_source.next() => {
                    let Some(message) = message else {
                        break Ok(());
                    };
                    let message = message.map_err(ForwardError::TestRunnerChannel)?;
                    let serialized =
                        serde_json::to_vec(&message).map_err(ForwardError::SerializeFailed)?;
                    log::trace!("serial[runner→wire] bytes={}", serialized.len());
                    serial_sink
                        .send(Frame::TestRunner(serialized.into()))
                        .await
                        .map_err(ForwardError::SerialConnection)?;
                }

                data = daemon_source.next(), if daemon_open => {
                    let Some(data) = data else {
                        log::warn!(
                            "serial daemon source EOF; keeping mux alive for TestRunner"
                        );
                        let _ = serial_sink.send(Frame::DaemonRpc(Bytes::new())).await;
                        daemon_open = false;
                        continue;
                    };
                    let data = data.map_err(ForwardError::DaemonChannel)?;
                    log::trace!("serial[daemon→wire] bytes={}", data.len());
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
/// Matches `LengthDelimitedCodec` default. Must fit in a 4-byte length field
/// (`MULTIPLEX_LEN_DELIMITED_HEADER_SIZE`); do not use `usize::MAX`.
const MULTIPLEX_MAX_FRAME_LENGTH: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct MultiplexCodec {
    len_delim_codec: LengthDelimitedCodec,
    has_connected: Arc<AtomicBool>,
    /// True after a bad length/type; cleared on the next successful frame.
    desynced: bool,
}

impl MultiplexCodec {
    fn new(has_connected: Arc<AtomicBool>) -> Self {
        let mut codec_builder = LengthDelimitedCodec::builder();

        codec_builder
            .length_field_length(MULTIPLEX_LEN_DELIMITED_HEADER_SIZE)
            .max_frame_length(MULTIPLEX_MAX_FRAME_LENGTH);

        Self {
            has_connected,
            len_delim_codec: codec_builder.new_codec(),
            desynced: false,
        }
    }

    fn mark_desynced(&mut self, detail: &str) {
        if self.desynced {
            log::debug!("serial mux: {detail}");
        } else {
            self.desynced = true;
            log::warn!("serial mux: desync; {detail}");
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
        loop {
            if !self.resync_to_plausible_frame_header(src) {
                return Ok(None);
            }
            match self.len_delim_codec.decode(src) {
                Ok(Some(frame)) => {
                    let decoded = Self::decode_frame(frame)?;
                    if self.desynced {
                        log::info!("serial mux: re-locked after desync");
                        self.desynced = false;
                    }
                    return Ok(Some(decoded));
                }
                Ok(None) => return Ok(None),
                Err(error) if is_oversized_length_field_error(&error) => {
                    if src.is_empty() {
                        return Err(error);
                    }
                    if !self.desynced {
                        self.mark_desynced(&format!(
                            "oversized length after peek; skip 1 (remaining={})",
                            src.len()
                        ));
                    }
                    src.advance(1);
                }
                Err(error) => return Err(error),
            }
        }
    }

    /// Skip one byte at a time until length + FrameType (+ Handshake signature) look valid.
    fn resync_to_plausible_frame_header(&mut self, src: &mut BytesMut) -> bool {
        let mut skipped = 0usize;
        while src.len() >= MULTIPLEX_LEN_DELIMITED_HEADER_SIZE {
            if skipped >= MAX_RESYNC_BYTES_PER_DECODE {
                if !self.desynced {
                    self.mark_desynced("resync budget exhausted; pause until more bytes");
                }
                return false;
            }
            let claimed_len = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;
            if !(1..=MULTIPLEX_MAX_FRAME_LENGTH).contains(&claimed_len) {
                if !self.desynced {
                    self.mark_desynced(&format!(
                        "implausible length {claimed_len}; skip 1 (remaining={})",
                        src.len()
                    ));
                }
                src.advance(1);
                skipped += 1;
                continue;
            }
            if src.len() < MULTIPLEX_LEN_DELIMITED_HEADER_SIZE + FRAME_TYPE_SIZE {
                return false;
            }
            let frame_type = match FrameType::try_from(src[MULTIPLEX_LEN_DELIMITED_HEADER_SIZE]) {
                Ok(frame_type) => frame_type,
                Err(()) => {
                    if !self.desynced {
                        self.mark_desynced(&format!(
                            "length {claimed_len} bad type {:#04x}; skip 1 (remaining={})",
                            src[MULTIPLEX_LEN_DELIMITED_HEADER_SIZE],
                            src.len()
                        ));
                    }
                    src.advance(1);
                    skipped += 1;
                    continue;
                }
            };
            if src.len() < MULTIPLEX_LEN_DELIMITED_HEADER_SIZE + claimed_len {
                // While desynced, keep hunting through incomplete claims in the same decode
                // call. Waiting would stall 1-byte resync on phantom lengths assembled from
                // misaligned noise (CI recovery). Trade-off: a real in-flight frame that
                // arrives mid-desync can lose its first byte; the next complete frame recovers.
                if self.desynced {
                    src.advance(1);
                    skipped += 1;
                    continue;
                }
                return false;
            }
            if matches!(frame_type, FrameType::Handshake) {
                let payload_start = MULTIPLEX_LEN_DELIMITED_HEADER_SIZE + FRAME_TYPE_SIZE;
                let payload_end = MULTIPLEX_LEN_DELIMITED_HEADER_SIZE + claimed_len;
                if &src[payload_start..payload_end] != MULLVAD_SIGNATURE {
                    if !self.desynced {
                        self.mark_desynced(&format!(
                            "Handshake without signature; skip 1 (remaining={})",
                            src.len()
                        ));
                    }
                    src.advance(1);
                    skipped += 1;
                    continue;
                }
            }
            return true;
        }
        false
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

fn is_oversized_length_field_error(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::InvalidData && error.to_string().contains("frame size too big")
}

#[cfg(test)]
mod tests {
    use super::{
        DAEMON_CHANNEL_BUF_SIZE, ForwardOutcome, SESSION_SYNC_ACK, SESSION_SYNC_PING,
        create_client_transports, create_server_transports, forward_framed_bidirectional,
        length_delimited_framed_halves, synchronize_framed_session,
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
        let (mut bridge_framed_read, mut bridge_framed_write) =
            length_delimited_framed_halves(bridge_framed_io);
        let mut framed_peer = LengthDelimitedCodec::new().framed(framed_peer_io);

        let bridge = tokio::spawn(async move {
            forward_framed_bidirectional(
                bridge_stream,
                &mut bridge_framed_read,
                &mut bridge_framed_write,
            )
            .await
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
        let (mut bridge_framed_read, mut bridge_framed_write) =
            length_delimited_framed_halves(bridge_framed_io);
        let mut framed_peer = LengthDelimitedCodec::new().framed(framed_peer_io);

        let bridge = tokio::spawn(async move {
            forward_framed_bidirectional(
                bridge_stream,
                &mut bridge_framed_read,
                &mut bridge_framed_write,
            )
            .await
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
    async fn session_restart_returns_without_waiting_for_stream_eof() {
        let (bridge_stream, stream_peer) = duplex(64);
        let (mut stream_peer_reader, _stream_peer_writer) = tokio::io::split(stream_peer);
        let (bridge_framed_io, framed_peer_io) = duplex(64);
        let (mut bridge_framed_read, mut bridge_framed_write) =
            length_delimited_framed_halves(bridge_framed_io);
        let mut framed_peer = LengthDelimitedCodec::new().framed(framed_peer_io);

        let bridge = tokio::spawn(async move {
            forward_framed_bidirectional(
                bridge_stream,
                &mut bridge_framed_read,
                &mut bridge_framed_write,
            )
            .await
        });

        framed_peer
            .send(Bytes::from_static(SESSION_SYNC_PING))
            .await
            .expect("request a new session mid-forward");

        assert_eq!(
            stream_peer_reader.read_u8().await.unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(1), bridge)
                .await
                .expect("restart must not wait for the stream peer to close")
                .expect("bridge task")
                .expect("clean restart"),
            ForwardOutcome::SessionRestart
        );
        let trailing = tokio::time::timeout(Duration::from_millis(50), framed_peer.next()).await;
        assert!(
            matches!(trailing, Err(_) | Ok(None)),
            "a stray EOF frame would close the session that follows the restart: {trailing:?}"
        );
    }

    #[tokio::test]
    async fn session_sync_completes_while_the_previous_session_is_still_streaming() {
        let (guest_io, host_io) = duplex(64);
        let (mut guest_read, mut guest_write) = length_delimited_framed_halves(guest_io);
        let (mut host_read, mut host_write) = length_delimited_framed_halves(host_io);
        let (guest_stream, stream_peer) = duplex(64);
        let (_stream_peer_reader, mut stream_peer_writer) = tokio::io::split(stream_peer);

        let flood = tokio::spawn(async move {
            let _ = stream_peer_writer.write_all(&vec![3u8; 32 * 1024]).await;
            stream_peer_writer
        });

        let guest = tokio::spawn(async move {
            let outcome =
                forward_framed_bidirectional(guest_stream, &mut guest_read, &mut guest_write)
                    .await
                    .expect("guest forward");
            assert_eq!(outcome, ForwardOutcome::SessionRestart);
            guest_write
                .send(Bytes::from_static(SESSION_SYNC_ACK))
                .await
                .expect("acknowledge session restart");
        });

        tokio::time::sleep(Duration::from_millis(50)).await;
        tokio::time::timeout(
            Duration::from_secs(5),
            synchronize_framed_session(&mut host_read, &mut host_write),
        )
        .await
        .expect("in-flight responses must not stall the next session")
        .expect("session synchronized");

        flood.abort();
        guest.await.expect("guest task");
    }

    #[tokio::test]
    async fn session_sync_discards_delayed_frames_before_acknowledgement() {
        let (bridge_io, peer_io) = duplex(64);
        let (mut bridge_read, mut bridge_write) = length_delimited_framed_halves(bridge_io);
        let mut peer = LengthDelimitedCodec::new().framed(peer_io);

        bridge_write
            .send(Bytes::from_static(b"stale request"))
            .await
            .expect("queue stale request");
        let sync = tokio::spawn(async move {
            synchronize_framed_session(&mut bridge_read, &mut bridge_write)
                .await
                .map(|()| (bridge_read, bridge_write))
        });

        assert_eq!(
            peer.next()
                .await
                .expect("stale request")
                .expect("valid frame"),
            Bytes::from_static(b"stale request")
        );
        assert_eq!(
            peer.next()
                .await
                .expect("sync marker")
                .expect("valid frame"),
            Bytes::from_static(SESSION_SYNC_PING)
        );
        peer.send(Bytes::from_static(b"stale response"))
            .await
            .expect("send delayed stale response");
        peer.send(Bytes::new())
            .await
            .expect("send stale EOF from the torn-down session");
        peer.send(Bytes::from_static(SESSION_SYNC_ACK))
            .await
            .expect("acknowledge synchronization");

        let (mut bridge_read, _bridge_write) = sync
            .await
            .expect("sync task")
            .expect("session synchronized");
        assert!(
            tokio::time::timeout(Duration::from_millis(10), bridge_read.next())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn session_sync_flushes_a_cancelled_partial_frame_before_acknowledgement() {
        let (bridge_io, peer_io) = duplex(64);
        let (mut bridge_read, mut bridge_write) = length_delimited_framed_halves(bridge_io);
        let mut peer = LengthDelimitedCodec::new().framed(peer_io);
        let stale_request = Bytes::from(vec![9; 4096]);

        let mut cancelled_send = Box::pin(bridge_write.send(stale_request.clone()));
        assert!(futures::poll!(&mut cancelled_send).is_pending());
        drop(cancelled_send);

        let sync = tokio::spawn(async move {
            synchronize_framed_session(&mut bridge_read, &mut bridge_write).await
        });

        assert_eq!(
            peer.next()
                .await
                .expect("cancelled frame")
                .expect("valid frame"),
            stale_request
        );
        assert_eq!(
            peer.next()
                .await
                .expect("sync marker")
                .expect("valid frame"),
            Bytes::from_static(SESSION_SYNC_PING)
        );
        peer.send(Bytes::from_static(SESSION_SYNC_ACK))
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
        let (mut bridge_framed_read, mut bridge_framed_write) =
            length_delimited_framed_halves(bridge_framed_io);
        let mut framed_peer = LengthDelimitedCodec::new().framed(framed_peer_io);

        let bridge = tokio::spawn(async move {
            forward_framed_bidirectional(
                bridge_stream,
                &mut bridge_framed_read,
                &mut bridge_framed_write,
            )
            .await
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
    async fn blocked_framed_write_flush_does_not_block_framed_read() {
        let (bridge_stream, stream_peer) = duplex(64);
        let (mut stream_reader, mut stream_writer) = tokio::io::split(stream_peer);
        let (bridge_framed_io, framed_peer_io) = duplex(64);
        let (mut bridge_framed_read, mut bridge_framed_write) =
            length_delimited_framed_halves(bridge_framed_io);
        let mut framed_peer = LengthDelimitedCodec::new().framed(framed_peer_io);

        let bridge = tokio::spawn(async move {
            forward_framed_bidirectional(
                bridge_stream,
                &mut bridge_framed_read,
                &mut bridge_framed_write,
            )
            .await
        });

        let flood = tokio::spawn(async move {
            let _ = stream_writer.write_all(&vec![1u8; 32 * 1024]).await;
            stream_writer
        });
        tokio::time::sleep(Duration::from_millis(50)).await;

        framed_peer
            .send(Bytes::from_static(b"pong"))
            .await
            .expect("send opposite-direction frame while framed write flush is blocked");

        let mut buf = [0u8; 4];
        tokio::time::timeout(Duration::from_secs(1), stream_reader.read_exact(&mut buf))
            .await
            .expect("framed read must progress while framed write flush is blocked")
            .expect("read opposite-direction bytes");
        assert_eq!(&buf, b"pong");

        flood.abort();
        bridge.abort();
    }

    #[tokio::test]
    async fn bidirectional_large_framed_exchange_over_tiny_duplex() {
        let (a_io, b_io) = duplex(128);
        let (mut a_read, mut a_write) = length_delimited_framed_halves(a_io);
        let (mut b_read, mut b_write) = length_delimited_framed_halves(b_io);
        let payload_a = Bytes::from(vec![0xA; 8192]);
        let payload_b = Bytes::from(vec![0xB; 8192]);

        tokio::time::timeout(Duration::from_secs(1), async {
            tokio::try_join!(
                async {
                    let send = a_write.send(payload_a.clone());
                    let recv = async {
                        a_read
                            .next()
                            .await
                            .ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))?
                    };
                    let (_, received) = tokio::try_join!(send, recv)?;
                    assert_eq!(received, payload_b);
                    Ok::<_, io::Error>(())
                },
                async {
                    let send = b_write.send(payload_b.clone());
                    let recv = async {
                        b_read
                            .next()
                            .await
                            .ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))?
                    };
                    let (_, received) = tokio::try_join!(send, recv)?;
                    assert_eq!(received, payload_a);
                    Ok::<_, io::Error>(())
                },
            )
        })
        .await
        .expect("concurrent large framed exchange must complete within 1s")
        .expect("exchange succeeds");
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

    /// Control-plane (tarpc) replies must not be starved when the daemon half keeps the
    /// shared serial writer Ready. Regression for post-Connected WaitOutcome delivery.
    #[tokio::test]
    async fn runner_reply_is_not_starved_by_ready_daemon_source() {
        use crate::ServiceResponse;
        use tarpc::{ClientMessage, Response};

        let (client_serial, server_serial) = duplex(64 * 1024);
        let (client_runner, _client_daemon, mut connection, client_task) =
            create_client_transports(client_serial).expect("create client transports");
        let (mut server_runner, server_daemon, server_task) =
            create_server_transports(server_serial);
        let mut server_daemon = LengthDelimitedCodec::new().framed(server_daemon);

        connection
            .wait_for_server()
            .await
            .expect("complete transport handshake");

        let flood = tokio::spawn(async move {
            loop {
                if server_daemon
                    .send(Bytes::from(vec![7; 1024]))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });
        // Let daemon_source become continuously Ready on the server serial writer.
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client =
            crate::service::ServiceClient::new(tarpc::client::Config::default(), client_runner)
                .spawn();
        let reply = tokio::spawn(async move {
            client
                .get_default_interface(tarpc::context::current())
                .await
        });

        let request = tokio::time::timeout(Duration::from_secs(1), server_runner.next())
            .await
            .expect("request must reach the server despite daemon flood")
            .expect("server runner transport remains open")
            .expect("runner request is valid");
        let ClientMessage::Request(request) = request else {
            panic!("expected a tarpc request");
        };

        // `Response` is `#[non_exhaustive]`; build via serde (same wire shape as production).
        let response: Response<ServiceResponse> = serde_json::from_value(serde_json::json!({
            "request_id": request.id,
            "message": {
                "Ok": { "GetDefaultInterface": { "Ok": "eth0" } }
            }
        }))
        .expect("construct tarpc Response via serde");

        server_runner
            .send(response)
            .await
            .expect("queue runner reply while daemon source is Ready");

        let interface = tokio::time::timeout(Duration::from_secs(1), reply)
            .await
            .expect("runner reply must not be starved by a Ready daemon source")
            .expect("reply task joins")
            .expect("tarpc round-trip succeeds")
            .expect("get_default_interface Ok");

        assert_eq!(interface, "eth0");
        flood.abort();
        client_task.abort();
        server_task.abort();
    }

    #[test]
    fn multiplex_max_frame_fits_in_four_byte_length_header() {
        use super::{MULTIPLEX_LEN_DELIMITED_HEADER_SIZE, MULTIPLEX_MAX_FRAME_LENGTH};

        assert_eq!(MULTIPLEX_LEN_DELIMITED_HEADER_SIZE, 4);
        assert_eq!(MULTIPLEX_MAX_FRAME_LENGTH, 8 * 1024 * 1024);
        assert!(
            MULTIPLEX_MAX_FRAME_LENGTH <= u32::MAX as usize,
            "max frame must fit in a 4-byte length field; usize::MAX does not"
        );
    }

    #[test]
    fn multiplex_codec_ignores_frames_until_handshake_connected() {
        use super::{Frame, MultiplexCodec};
        use bytes::{BufMut, BytesMut};
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        use tokio_util::codec::{Decoder, Encoder};

        let connected = Arc::new(AtomicBool::new(false));
        let mut codec = MultiplexCodec::new(connected.clone());
        let mut encoded = BytesMut::new();
        codec
            .encode(Frame::TestRunner(Bytes::from_static(b"ping")), &mut encoded)
            .expect("encode");
        let mut src = encoded;
        assert!(
            codec.decode(&mut src).expect("decode").is_none(),
            "mid-suite reset_connected_state(false) would freeze tarpc; use abort_active_forward"
        );
        connected.store(true, Ordering::SeqCst);
        let mut src2 = BytesMut::new();
        src2.put_slice(&{
            let mut again = BytesMut::new();
            codec
                .encode(Frame::TestRunner(Bytes::from_static(b"ping")), &mut again)
                .expect("encode");
            again
        });
        assert!(matches!(
            codec.decode(&mut src2).expect("decode"),
            Some(Frame::TestRunner(_))
        ));
    }

    #[test]
    fn multiplex_codec_rejects_oversized_frame() {
        use super::{Frame, MULTIPLEX_MAX_FRAME_LENGTH, MultiplexCodec};
        use bytes::BytesMut;
        use std::sync::{Arc, atomic::AtomicBool};
        use tokio_util::codec::Encoder;

        let mut codec = MultiplexCodec::new(Arc::new(AtomicBool::new(true)));
        // Payload alone equals the max; + frame-type byte exceeds it.
        let oversized = Bytes::from(vec![0u8; MULTIPLEX_MAX_FRAME_LENGTH]);
        let mut dst = BytesMut::new();
        let err = codec
            .encode(Frame::TestRunner(oversized), &mut dst)
            .expect_err("frame at max without room for type byte must fail");
        assert!(
            matches!(
                err.kind(),
                io::ErrorKind::InvalidData | io::ErrorKind::InvalidInput
            ),
            "unexpected error kind: {:?}",
            err.kind()
        );
    }

    fn encode_test_runner_frame(payload: &'static [u8]) -> bytes::BytesMut {
        use super::{Frame, MultiplexCodec};
        use bytes::BytesMut;
        use std::sync::{Arc, atomic::AtomicBool};
        use tokio_util::codec::Encoder;

        let mut codec = MultiplexCodec::new(Arc::new(AtomicBool::new(true)));
        let mut valid = BytesMut::new();
        codec
            .encode(Frame::TestRunner(Bytes::from_static(payload)), &mut valid)
            .expect("encode valid frame");
        valid
    }

    fn assert_decoded_test_runner(decoded: super::Frame, expected: &[u8]) {
        match decoded {
            super::Frame::TestRunner(payload) => assert_eq!(&payload[..], expected),
            super::Frame::Handshake => panic!("expected TestRunner, got Handshake"),
            super::Frame::DaemonRpc(_) => panic!("expected TestRunner, got DaemonRpc"),
        }
    }

    #[test]
    fn multiplex_codec_resyncs_after_oversized_length_field() {
        use super::{MULTIPLEX_MAX_FRAME_LENGTH, MultiplexCodec};
        use bytes::{BufMut, BytesMut};
        use std::sync::{Arc, atomic::AtomicBool};
        use tokio_util::codec::Decoder;

        let mut codec = MultiplexCodec::new(Arc::new(AtomicBool::new(true)));
        let mut src = BytesMut::new();
        src.put_u32((MULTIPLEX_MAX_FRAME_LENGTH as u32).saturating_add(1));
        src.extend_from_slice(&encode_test_runner_frame(b"ok"));

        let decoded = codec
            .decode(&mut src)
            .expect("resync must not abort decode")
            .expect("valid frame after oversized length resync");
        assert_decoded_test_runner(decoded, b"ok");
    }

    /// 4-byte header skip destroys a frame that starts one byte after noise.
    #[test]
    fn multiplex_codec_resyncs_when_misaligned_by_one_byte() {
        use super::MultiplexCodec;
        use bytes::BytesMut;
        use std::sync::{Arc, atomic::AtomicBool};
        use tokio_util::codec::Decoder;

        let mut codec = MultiplexCodec::new(Arc::new(AtomicBool::new(true)));
        let mut src = BytesMut::new();
        // Leading 0xFF makes the first u32 length look huge (almost always > 8 MiB).
        src.extend_from_slice(&[0xff]);
        src.extend_from_slice(&encode_test_runner_frame(b"aligned"));

        let decoded = codec
            .decode(&mut src)
            .expect("1-byte resync must not abort")
            .expect("valid frame after 1-byte misalignment");
        assert_decoded_test_runner(decoded, b"aligned");
        assert!(src.is_empty(), "must consume the full valid frame");
    }

    #[test]
    fn multiplex_codec_resync_budget_pauses_without_aborting() {
        use super::{MAX_RESYNC_BYTES_PER_DECODE, MultiplexCodec};
        use bytes::{BufMut, BytesMut};
        use std::sync::{Arc, atomic::AtomicBool};
        use tokio_util::codec::Decoder;

        let mut codec = MultiplexCodec::new(Arc::new(AtomicBool::new(true)));
        let mut src = BytesMut::new();
        // Implausible lengths forever: resync must stop after the per-decode budget.
        for _ in 0..(MAX_RESYNC_BYTES_PER_DECODE + 32) {
            src.put_u8(0xff);
        }

        let decoded = codec
            .decode(&mut src)
            .expect("budget exhaustion must not abort decode");
        assert!(decoded.is_none());
        assert_eq!(
            src.len(),
            32,
            "must pause after the resync budget and leave unread noise"
        );
    }

    #[test]
    fn multiplex_codec_resyncs_past_plausible_length_with_bad_frame_type() {
        use super::{FRAME_TYPE_SIZE, MULTIPLEX_LEN_DELIMITED_HEADER_SIZE, MultiplexCodec};
        use bytes::{BufMut, BytesMut};
        use std::sync::{Arc, atomic::AtomicBool};
        use tokio_util::codec::Decoder;

        let mut codec = MultiplexCodec::new(Arc::new(AtomicBool::new(true)));
        let mut src = BytesMut::new();
        // Length=1 looks fine to LengthDelimitedCodec, but 0xFE is not a FrameType.
        src.put_u32(FRAME_TYPE_SIZE as u32);
        src.put_u8(0xfe);
        // Padding so a 4-byte-only skip would still leave us unaligned into the real frame.
        src.extend_from_slice(&[0x00; MULTIPLEX_LEN_DELIMITED_HEADER_SIZE]);
        src.extend_from_slice(&encode_test_runner_frame(b"typed"));

        let decoded = codec
            .decode(&mut src)
            .expect("bad frame-type resync must not abort")
            .expect("valid frame after skipping fake header");
        assert_decoded_test_runner(decoded, b"typed");
    }
}
