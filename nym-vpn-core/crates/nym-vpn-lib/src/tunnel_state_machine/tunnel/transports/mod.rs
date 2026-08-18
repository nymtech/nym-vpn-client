// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(any(target_os = "linux", target_os = "android"))]
use std::os::fd::{AsRawFd, RawFd};
use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};

use base64::prelude::*;
use bytes::{Buf, BytesMut};
use futures::{Sink, SinkExt, Stream, StreamExt};
use quinn::{IdleTimeout, TransportConfig, congestion};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::UdpSocket,
    sync::mpsc::UnboundedSender,
    task::JoinHandle,
};
use tokio_util::{codec::LengthDelimitedCodec, sync::CancellationToken};
use tracing::*;

mod certs;
use certs::*;
pub use nym_vpn_api_client::response::{BridgeInformation, BridgeParameters, QuicClientOptions};

use crate::tunnel_state_machine::tunnel::wireguard::two_hop_config::ETHERNET_V2_MTU;

const LENGTH_DELIMITER_BYTELEN: usize = 2;
/// First WG datagram wait: five handshake attempts at RekeyTimeout (5s).
const INITIAL_CONNECTION_TIMEOUT: Duration = Duration::from_secs(21);
const QUIC_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(thiserror::Error, Debug)]
pub enum TransportError {
    #[error("quic conn error: {0}")]
    Quic(#[from] quinn::ConnectError),

    #[error("quic proto error: {0}")]
    QuicProto(#[from] quinn::ConnectionError),

    #[error("transport socket io error")]
    SocketIo(#[source] std::io::Error),

    #[error("insufficient or broken transport params: {0}")]
    Config(String),

    #[error("transport connection was cancelled")]
    Cancelled,

    #[error("quic connection timed out after {0:?}")]
    TimedOut(Duration),

    #[error("transport error: {0}")]
    Other(String),
}

impl TransportError {
    pub fn config_err(s: impl AsRef<str>) -> Self {
        Self::Config(s.as_ref().to_string())
    }

    pub fn other(s: impl AsRef<str>) -> Self {
        Self::Other(s.as_ref().to_string())
    }
}

pub struct BridgeConn {
    /// Configured parameters from which this bridge connections was built
    #[allow(unused)] // we will want these later for metrics tracking
    pub(crate) params: BridgeParameters,
    /// Remote address of the bridge transport connection
    pub(crate) endpoint: SocketAddr,
    pub(crate) reader: Box<dyn AsyncRead + Send + Unpin>,
    pub(crate) writer: Box<dyn AsyncWrite + Send + Unpin>,
}

impl BridgeConn {
    pub async fn try_connect(
        params: BridgeParameters,
        token: CancellationToken,
        #[cfg(any(target_os = "linux", target_os = "android"))] on_socket_open: impl FnOnce(RawFd),
    ) -> Result<Self, TransportError> {
        let start = Instant::now();

        match params {
            BridgeParameters::QuicPlain(ref opts) => {
                let opts = ClientOptions::try_from(opts)?;

                let conn = token
                    .run_until_cancelled(transport_conn(
                        &opts,
                        #[cfg(any(target_os = "linux", target_os = "android"))]
                        on_socket_open,
                    ))
                    .await
                    .ok_or(TransportError::Cancelled)??;
                let endpoint = conn.remote_address();
                // .context("failed to connect to transport conn")?;
                let (writer, reader) = token
                    .run_until_cancelled(conn.open_bi())
                    .await
                    .ok_or(TransportError::Cancelled)??;
                // .context("failed to connect to transport stream")?;
                info!("quic transport connected in {:?}", start.elapsed());
                Ok(Self {
                    reader: Box::new(reader),
                    writer: Box::new(writer),
                    params,
                    endpoint,
                })
            }
        }
    }
}

pub struct UdpForwarder {}

impl UdpForwarder {
    pub async fn launch(
        egress_conn: BridgeConn,
        bind_addr: Option<SocketAddr>,
        close_tx: UnboundedSender<()>,
        token: CancellationToken,
    ) -> Result<(SocketAddr, JoinHandle<()>), TransportError> {
        let bind_addr = bind_addr.unwrap_or(match egress_conn.endpoint.is_ipv4() {
            true => (Ipv4Addr::LOCALHOST, 0).into(),
            false => (Ipv6Addr::LOCALHOST, 0).into(),
        });
        let socket = make_socket(Some(bind_addr)).map_err(TransportError::SocketIo)?;
        let socket = Arc::new(UdpSocket::from_std(socket).map_err(TransportError::SocketIo)?);
        let local_addr = socket.local_addr().map_err(TransportError::SocketIo)?;

        info!("udp forwarder started listening on: {local_addr}",);

        Ok((
            local_addr,
            tokio::spawn(process_udp(
                egress_conn.reader,
                egress_conn.writer,
                socket.clone(),
                ETHERNET_V2_MTU,
                close_tx,
                token,
            )),
        ))
    }
}

pub async fn process_udp<R, W>(
    reader: R,
    writer: W,
    sock: Arc<UdpSocket>,
    mtu: u16,
    // close_hook: Option<fn(SocketAddr)>,
    close_tx: UnboundedSender<()>,
    token: CancellationToken,
) where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    info!("starting udp forward");

    let mut dn_buf = BytesMut::with_capacity(mtu as usize);

    let mut framed_writer = LengthDelimitedCodec::builder()
        .length_field_length(LENGTH_DELIMITER_BYTELEN)
        .new_write(writer);

    let framed_reader = LengthDelimitedCodec::builder()
        .length_field_length(LENGTH_DELIMITER_BYTELEN)
        .new_read(reader);

    // receive (and forward) a first message to establish a consistent peer address
    let fwd_initial_recv_fut =
        tokio::time::timeout(INITIAL_CONNECTION_TIMEOUT, sock.recv_buf_from(&mut dn_buf));

    let fwd_addr = match token.run_until_cancelled(fwd_initial_recv_fut).await {
        Some(res) => {
            match res {
                Ok(Ok((len, src))) => {
                    trace!(" <- [fw] read {len}B");
                    if let Err(e) = framed_writer.send(dn_buf.copy_to_bytes(len)).await {
                        debug!("error sending to transport connection: {e}");
                        None
                    } else {
                        trace!("[tr] <- wrote {len}B");
                        // keep track of the address of the sender for the initial write
                        Some(src)
                    }
                }
                Ok(Err(e)) => {
                    debug!("error receiving from egress socket: {e}");
                    None
                }
                Err(_) => {
                    debug!("forwarder timed out");
                    None
                }
            }
        }
        None => {
            debug!("forwarder cancelled before initial receive");
            None
        }
    };

    let Some(fwd_addr) = fwd_addr else {
        close_tx.send(()).ok();
        return;
    };

    if let Err(e) = sock.connect(fwd_addr).await {
        error!("udp sock config failure: {e}");
        close_tx.send(()).ok();
        return;
    }

    let mut tasks = tokio::task::JoinSet::new();
    tasks.spawn(udp_to_transport_task(
        sock.clone(),
        framed_writer,
        fwd_addr,
        mtu,
        token.child_token(),
    ));
    tasks.spawn(transport_to_udp_task(
        framed_reader,
        sock.clone(),
        fwd_addr,
        token.child_token(),
    ));

    let mut token = Some(token);

    // Wait for both tasks to complete, if either one exits, make sure to cancel the other as well.
    while let Some(res) = tasks.join_next().await {
        if let Err(err) = res {
            tracing::error!("bridge udp forwarder join error: {err}");
        } else if let Ok(Err(err)) = res {
            tracing::error!("bridge udp forwarder error: {err}");
        }

        // Cancel all tasks if any of sub-tasks exit for any reason
        if let Some(token) = token.take() {
            token.cancel();
        }
    }

    close_tx.send(()).ok();

    info!("transport udp forwarder shutdown");
}

// Assumes that the socket has already had `connect` called.
async fn udp_to_transport_task<W>(
    sock: Arc<UdpSocket>,
    mut framed_writer: W,
    fwd_addr: SocketAddr,
    mtu: u16,
    token: CancellationToken,
) -> Result<(), io::Error>
where
    W: Sink<bytes::Bytes, Error = io::Error> + Unpin + Send,
{
    // allocate buffers of mtu size, and take ownership to ensure they can't be resized anymore
    let mut dn_buf = BytesMut::with_capacity(mtu as usize);

    loop {
        tokio::select! {
            res = sock.recv_buf(&mut dn_buf) => {
                let len = res.map_err(|e| {
                    error!("error receiving from forward socket: {e}");
                    e
                })?;

                trace!(" <-{fwd_addr} read {len}B");
                framed_writer.send(dn_buf.copy_to_bytes(len)).await.map_err(|e| {
                    error!("error sending to transport connection: {e}");
                    e
                })?;
                trace!(" [tr]<- wrote {len}B");

                //reset the buffer without any new allocations.
                dn_buf.clear();
                if !dn_buf.try_reclaim(mtu as usize) {
                    warn!("unable to reclaim bytes in buffer: {} ", dn_buf.capacity());
                }
            }
            _ = token.cancelled() => {
                debug!("end io copy from {fwd_addr}<->[tr]");
                break;
            }
        }
    }
    Ok(())
}

// Assumes that the socket has already had `connect` called.
async fn transport_to_udp_task<R>(
    mut framed_reader: R,
    sock: Arc<UdpSocket>,
    fwd_addr: SocketAddr,
    token: CancellationToken,
) -> Result<(), io::Error>
where
    R: Stream<Item = Result<bytes::BytesMut, io::Error>> + Unpin + Send,
{
    loop {
        tokio::select! {
            res = framed_reader.next() => {
                match res {
                    None => {
                        info!("connection closed");
                        break;
                    }
                    Some(Ok(buf)) => {
                        let len = buf.len();
                        trace!("[tr]-> read {len}B");
                        let mut sent = 0;
                        let mut sends = 1;
                        while sent < len {
                            let len_sent = sock.send(&buf[sent..len]).await.map_err(|e| {
                                error!("error sending to egress socket: {e}");
                                e
                            })?;
                            sent += len_sent;
                            trace!(" ->{fwd_addr} wrote {len_sent}B {sends} send");
                            sends +=1;
                        }
                    }
                    Some(Err(e)) => {
                        error!("error reading from transport conn: {e}");
                        return Err(e);
                    }
                }
            }
            _ = token.cancelled() => {
                debug!("end io copy");
                break;
            }
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClientOptions {
    /// Address describing the remote transport server
    ///
    /// Must parse as a valid [`std::net::SocketAddr`] - e.g. `123.45.67.89:443`
    pub addresses: Vec<SocketAddr>,

    /// Override hostname used for certificate verification
    pub host: Option<String>,

    /// Use identity public key to verify server self signed certificate
    pub id_pubkey: VerifyingKey,
}

impl TryFrom<&QuicClientOptions> for ClientOptions {
    type Error = TransportError;
    fn try_from(value: &QuicClientOptions) -> Result<Self, Self::Error> {
        let id_pubkey = Self::parse_base64_pubkey(&value.id_pubkey)?;

        Ok(Self {
            addresses: value.addresses.clone(),
            host: value.host.clone(),
            id_pubkey,
        })
    }
}

impl ClientOptions {
    fn parse_base64_pubkey(key: impl AsRef<str>) -> Result<VerifyingKey, TransportError> {
        let mut pubkey_bytes = [0u8; 32];
        BASE64_STANDARD
            .decode_slice(key.as_ref(), &mut pubkey_bytes)
            .map_err(|e| {
                TransportError::config_err(format!(
                    "failed to decode Quic bridge public key as base64: {e}"
                ))
            })?;
        VerifyingKey::from_bytes(&pubkey_bytes)
            .map_err(|e| TransportError::config_err(format!("bad Quic bridge public key: {e}")))
    }

    fn get_ipv4(&self) -> Option<SocketAddr> {
        self.addresses.iter().find(|s| s.is_ipv4()).cloned()
    }
}

pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-29"];

use ed25519_dalek::VerifyingKey;
use quinn_proto::crypto::rustls::QuicClientConfig;

pub async fn transport_conn(
    options: &ClientOptions,
    #[cfg(any(target_os = "linux", target_os = "android"))] on_socket_open: impl FnOnce(RawFd),
) -> Result<quinn::Connection, TransportError> {
    info!("initializing from transport identity pubkey");

    let transport_endpoint = options
        .get_ipv4()
        .ok_or(TransportError::config_err("No IPv4 endpoint provided"))?;

    let client_config = create_quic_config(options)?;

    let bind_addr = match transport_endpoint.is_ipv4() {
        true => (Ipv4Addr::UNSPECIFIED, 0).into(),
        false => (Ipv6Addr::UNSPECIFIED, 0).into(),
    };
    let socket = make_socket(Some(bind_addr)).map_err(TransportError::SocketIo)?;
    #[cfg(any(target_os = "linux", target_os = "android"))]
    on_socket_open(socket.as_raw_fd());

    let runtime =
        quinn::default_runtime().ok_or_else(|| TransportError::other("no async runtime found"))?;
    let mut endpoint = quinn::Endpoint::new_with_abstract_socket(
        Default::default(),
        None,
        runtime
            .wrap_udp_socket(socket)
            .map_err(TransportError::SocketIo)?,
        runtime,
    )
    .map_err(TransportError::SocketIo)?;
    endpoint.set_default_client_config(client_config);

    // If no hostname is provided use the IP address of the remote server as the hostname.
    let addr_host = transport_endpoint.ip().to_string();
    let host = options.host.as_deref().unwrap_or(&addr_host);

    let connecting = endpoint
        .connect(transport_endpoint, host)
        .map_err(TransportError::Quic)?;

    match tokio::time::timeout(QUIC_CONNECT_TIMEOUT, connecting).await {
        Ok(connection) => connection.map_err(TransportError::QuicProto),
        Err(_) => {
            tracing::warn!(
                "QUIC bridge connection to {transport_endpoint} timed out after {QUIC_CONNECT_TIMEOUT:?}"
            );
            endpoint.close(0u32.into(), b"timeout");
            // TimedOut has no error_state_reason, so tunnel_monitor propagates it as
            // TunnelMonitorEvent::Down { error_state_reason: None }, which triggers
            // ConnectingState::reconnect() with a different gateway selection.
            Err(TransportError::TimedOut(QUIC_CONNECT_TIMEOUT))
        }
    }
}

/// Session Keepalive interval to prevent sessions from closing due to lull in user traffic.
///
/// ```txt
/// Keep-alive packets prevent an inactive but otherwise healthy connection from timing out.
///
/// ... Only one side of any given connection needs keep-alive enabled for the connection to
/// be preserved. Must be set lower than the idle_timeout of both peers to be effective.
/// ```
/// Default idle timeout is 30s. Our clients set to 60s  using [`QUIC_SESSION_IDLE_TIMEOUT`] to be
/// safe.
const QUIC_SESSION_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(20);

lazy_static::lazy_static! {
    /// Session Idle Timeout Interval -- if nothing is sent within this interval then the session
    /// will proceed with a healthy close. This is intentionally set higher than the
    /// [`QUIC_SESSION_KEEPALIVE_INTERVAL`] as we do not want a low period in traffic to result
    /// in a tunnel closing.
    static ref QUIC_SESSION_IDLE_TIMEOUT: IdleTimeout = IdleTimeout::from(quinn::VarInt::from_u32(60_000));
}

/// Create a client configuration for the quinn Quic client.
///
/// This sets the following properties to prepare the connection:
/// - adds hostname(s) from options to TLS alt names (if any)
/// - sets the TLS server cert verifier to our custom handler based on the pre-shared bridge pubkey
/// - sets TLS ALPN protocol header to HTTP
/// - sets keepalive_interval and max_idle_timeout to prevent sessions from closing during idle
/// - sets congestion controller to more fault tolerant BBR algorithm
/// - prevent server opening streams to client by setting uni and bidi streams to 0
///
/// All other properties are default.
fn create_quic_config(options: &ClientOptions) -> Result<quinn::ClientConfig, TransportError> {
    let crypto_provider = rustls::crypto::CryptoProvider::get_default()
        .unwrap_or(&Arc::new(rustls::crypto::ring::default_provider()))
        .clone();

    let alt_names = options.host.clone().map(|h| vec![h]);
    let verifier = IdentityBasedVerifier::builder(&options.id_pubkey)
        .with_alt_names(alt_names)
        .with_crypto_provider(crypto_provider.clone())
        .build()
        .map_err(|e| {
            TransportError::Config(format!(
                "failed to initialize quic cert verifier from options: {e}"
            ))
        })?;

    let mut client_crypto = rustls::ClientConfig::builder_with_provider(crypto_provider)
        .with_protocol_versions(rustls::DEFAULT_VERSIONS)
        .map_err(|e| TransportError::other(format!("rustls client config init failed: {e}")))?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth();

    client_crypto.alpn_protocols = ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();
    let quic_client_config = QuicClientConfig::try_from(client_crypto)
        .map_err(|e| TransportError::config_err(format!("invalid tls crypto config: {e}")))?;

    let mut transport_cfg = TransportConfig::default();
    // Set keepalive_interval and max_idle_timeout to prevent sessions from closing during idle
    transport_cfg.keep_alive_interval(Some(QUIC_SESSION_KEEPALIVE_INTERVAL));
    transport_cfg.max_idle_timeout(Some(*QUIC_SESSION_IDLE_TIMEOUT));

    // set congestion control to more fault tolerant BBR
    transport_cfg.congestion_controller_factory(Arc::new(congestion::BbrConfig::default()));

    // Prevent server opening streams to client by setting uni and bidi streams to 0 (we just have
    // no reason to allow this for now).
    transport_cfg.max_concurrent_bidi_streams(0_u32.into());
    transport_cfg.max_concurrent_uni_streams(0_u32.into());

    let mut client_config = quinn::ClientConfig::new(Arc::new(quic_client_config));
    client_config.transport_config(Arc::new(transport_cfg));

    Ok(client_config)
}

fn make_socket(addr: Option<SocketAddr>) -> io::Result<std::net::UdpSocket> {
    let addr = addr.unwrap_or((Ipv4Addr::UNSPECIFIED, 0).into());
    let socket = std::net::UdpSocket::bind(addr)?;
    socket.set_nonblocking(true)?;
    Ok(socket)
}
