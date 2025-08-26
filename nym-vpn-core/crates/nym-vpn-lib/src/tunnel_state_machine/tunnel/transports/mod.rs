use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::prelude::*;
use nym_wg_gateway_client::GatewayData;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;
use tracing::*;

mod certs;
use certs::*;

#[derive(thiserror::Error, Debug)]
pub enum TransportError {
    #[error("quic conn error: {0}")]
    Quic(#[from] quinn::ConnectError),

    #[error("quic proto error: {0}")]
    QuicProto(#[from] quinn::ConnectionError),

    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("insufficient or broken transport params: {0}")]
    Config(String),
}

impl TransportError {
    fn config_err(s: impl AsRef<str>) -> Self {
        Self::Config(s.as_ref().to_string())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BridgeParams {
    Quic(ClientOptions),
}

impl From<&GatewayData> for BridgeParams {
    fn from(value: &GatewayData) -> Self {
        let address = SocketAddr::new(value.endpoint.ip(), 4443);
        // TODO: NET-512 jmwample - THIS CANNOT STAY AS A STATIC KEY
        // this is meant to work for dev with node 3wqfp9
        let id_pubkey_bs64 = "K8PEmaK/z6Xj6owLmU4c9m08OXrrXLLm16d3ZTfzd64=";
        let mut pubkey_bytes = [0u8; 32];
        BASE64_STANDARD
            .decode_slice(id_pubkey_bs64, &mut pubkey_bytes)
            .unwrap();
        let id_pubkey = VerifyingKey::from_bytes(&pubkey_bytes).unwrap();

        BridgeParams::Quic(ClientOptions {
            address,
            host: Some("quic-test.example.com".into()),
            bind: Some("0.0.0.0:0".parse().unwrap()),
            id_pubkey,
        })
    }
}

pub struct BridgeConn {
    params: BridgeParams,
    reader: Box<dyn AsyncRead + Send + Unpin>,
    writer: Box<dyn AsyncWrite + Send + Unpin>,
}

impl BridgeConn {
    pub async fn try_connect(params: BridgeParams) -> Result<Self, TransportError> {
        let start = Instant::now();

        match params {
            BridgeParams::Quic(ref opts) => {
                let conn = transport_conn(opts).await?;
                // .context("failed to connect to transport conn")?;
                let (wr, rd) = conn.open_bi().await?;
                // .context("failed to connect to transport stream")?;
                info!("quic transport connected in {:?}", start.elapsed());
                Ok(Self {
                    reader: Box::new(rd),
                    writer: Box::new(wr),
                    params,
                })
            }
        }
    }
}

pub struct UdpForwarder {
    socket: Arc<UdpSocket>,
}

impl UdpForwarder {
    pub async fn new(
        egress_conn: BridgeConn,
        bind_addr: Option<SocketAddr>,
        token: CancellationToken,
    ) -> Result<Self, TransportError> {
        let bind_addr = bind_addr.unwrap_or(DEFAULT_CLIENT_BIND_ADDR.parse().unwrap());
        let socket = UdpSocket::bind(&bind_addr).await?;
        let socket = Arc::new(socket);

        info!(
            "udp forwarder started listening on: {}",
            socket.local_addr()?
        );

        tokio::spawn(process_udp(
            egress_conn.reader,
            egress_conn.writer,
            socket.clone(),
            token,
        ));
        // conn.close(0u32.into(), b"done");
        // debug!("stats: {:?}", conn.stats());
        info!("end session");

        Ok(Self { socket })
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }
}

pub async fn process_udp<R, W>(
    mut rd: R,
    mut wr: W,
    sock: Arc<UdpSocket>,
    // close_hook: Option<fn(SocketAddr)>,
    token: CancellationToken,
) where
    R: AsyncRead + Unpin + Send,
    W: AsyncWrite + Unpin + Send,
{
    info!("starting udp forward");
    let mut up_buf = [0u8; 1500];
    let mut dn_buf = [0u8; 1500];

    // receive (and forward) a first message to establish a consistent peer address
    let fw_addr =
        match tokio::time::timeout(Duration::from_secs(10), sock.recv_from(&mut dn_buf)).await {
            Ok(Ok((len, src))) => {
                trace!(" <- [fw] read {len}B");
                if let Err(e) = wr.write_all(&dn_buf[..len]).await {
                    debug!("error sending to transport connection: {e}");
                    token.cancel();
                    return;
                };
                trace!("[tr] <- wrote {len}B");
                src
            }
            Ok(Err(e)) => {
                debug!("error receiving from egress socket: {e}");
                token.cancel();
                return;
            }
            Err(_) => {
                debug!("forwarder timed out");
                token.cancel();
                return;
            }
        };

    loop {
        tokio::select! {
            res = rd.read(&mut up_buf) => {
                let len = match res {
                    Ok(0) => {
                        info!("connection closed");
                        break;
                    }
                    Ok(l) => l,
                    Err(e) => {
                        debug!("error reading from transport conn: {e}");
                        token.cancel();
                        break;
                    }
                };
                trace!("[tr] -> read {len}B");
                if let Err(e) = sock.send_to(&up_buf[..len], fw_addr).await {
                        debug!("error sending to egress socket: {e}");
                        token.cancel();
                        break;
                };
                trace!(" -> [fw] wrote {len}B");
            }
            res = sock.recv_from(&mut dn_buf) => {
                let (len, src) = match res {
                    Ok(l) => l,
                    Err(e) => {
                        debug!("error receiving from egress socket: {e}");
                        token.cancel();
                        break;
                    }
                };
                if src != fw_addr {
                    debug!("received {len}B from alt addr {src} -- ignoring");
                    continue
                }
                trace!(" <- [fw] read {len}B");
                if let Err(e) = wr.write_all(&dn_buf[..len]).await {
                        debug!("error sending to transport connection: {e}");
                        token.cancel();
                        break;
                };
                trace!("[tr] <- wrote {len}B");
            }
            _ = token.cancelled() => {
                // stop copying
                debug!("end io copy from [tr] <-> [fw]");
                break;
            }
        }
    }

    let _ = wr
        .shutdown()
        .await
        .inspect_err(|e| warn!("failed to close conn: {e}"));
    drop(sock);
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClientOptions {
    /// Address describing the remote transport server
    ///
    /// Must parse as a valid [`std::net::SocketAddr`] - e.g. `123.45.67.89:443`
    pub address: SocketAddr,

    /// Override hostname used for certificate verification
    pub host: Option<String>,

    /// Address to bind on
    pub bind: Option<SocketAddr>,

    /// Use identity public key to verify server self signed certificate
    pub id_pubkey: VerifyingKey,
}

const DEFAULT_CLIENT_BIND_ADDR: &str = "[::]:0";
pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-29"];

use ed25519_dalek::VerifyingKey;
use quinn_proto::crypto::rustls::QuicClientConfig;

pub async fn transport_conn(options: &ClientOptions) -> Result<quinn::Connection, TransportError> {
    info!("initializing from transport identity pubkey");

    let alt_names = options.host.clone().map(|h| vec![h]);
    let verifier =
        IdentityBasedVerifier::new_with_alt_names(&options.id_pubkey, alt_names).unwrap();

    let mut client_crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth();

    client_crypto.alpn_protocols = ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();
    let quic_client_config = QuicClientConfig::try_from(client_crypto)
        .map_err(|e| TransportError::config_err(format!("invalid tls crypto config: {e}")))?;

    let client_config = quinn::ClientConfig::new(Arc::new(quic_client_config));
    let mut endpoint = quinn::Endpoint::client(
        options
            .bind
            .unwrap_or(DEFAULT_CLIENT_BIND_ADDR.parse().unwrap()),
    )?;
    endpoint.set_default_client_config(client_config);

    // If no hostname is provided use the IP address of the remote server as the hostname.
    let addr_host = options.address.ip().to_string();
    let host = options.host.as_deref().unwrap_or(&addr_host);

    endpoint
        .connect(options.address, host)?
        .await
        .map_err(|e| TransportError::QuicProto(e))
}
