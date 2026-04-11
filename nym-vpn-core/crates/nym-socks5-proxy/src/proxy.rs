// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::Path,
    sync::Arc,
};

use anyhow::{Context, Result};
use fast_socks5::{
    server::{transfer, Socks5ServerProtocol}, util::target_addr::TargetAddr,
    ReplyError,
    Socks5Command,
};
use tokio::{
    net::{TcpListener, TcpSocket},
    sync::watch,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use nym_socks5_proxy_ipc::ProxyConfig;

use crate::routing::{decide_route, GeoIpDatabase, RoutingDecision};

pub async fn run(
    config: ProxyConfig,
    data_dir: &Path,
    tunnel_rx: watch::Receiver<Option<IpAddr>>,
    shutdown_token: CancellationToken,
) -> Result<()> {
    let listen_addr = SocketAddr::from(([127, 0, 0, 1], config.listen_port));
    let listener = TcpListener::bind(listen_addr)
        .await
        .with_context(|| format!("Failed to bind SOCKS5 listener on {listen_addr}"))?;

    info!(%listen_addr, "SOCKS5 proxy listener bound");

    let db = GeoIpDatabase::load(&config.excluded_countries, data_dir)
        .await
        .context("Failed to build GeoIP database")?;
    let db = Arc::new(db);

    tokio::spawn(accept_loop(listener, tunnel_rx, db, shutdown_token));

    Ok(())
}

async fn accept_loop(
    listener: TcpListener,
    tunnel_rx: watch::Receiver<Option<IpAddr>>,
    db: Arc<GeoIpDatabase>,
    shutdown_token: CancellationToken,
) {
    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, peer_addr)) => {
                        debug!(%peer_addr, "Accepted SOCKS5 connection");
                        let shutdown = shutdown_token.clone();
                        let tunnel = tunnel_rx.clone();
                        let db = db.clone();
                        tokio::spawn(async move {
                            if let Err(err) =
                                handle_connection(stream, peer_addr, tunnel, db, shutdown).await
                            {
                                warn!(%peer_addr, "SOCKS5 connection error: {err:#}");
                            }
                        });
                    }
                    Err(err) => {
                        warn!("SOCKS5 accept error: {err}");
                    }
                }
            }
            _ = shutdown_token.cancelled() => {
                info!("SOCKS5 accept loop shutting down");
                break;
            }
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    peer_addr: SocketAddr,
    tunnel_rx: watch::Receiver<Option<IpAddr>>,
    db: Arc<GeoIpDatabase>,
    shutdown_token: CancellationToken,
) -> Result<()> {
    // Snapshot the current tunnel state at connection time.
    let tunnel_addr = *tunnel_rx.borrow();

    tokio::select! {
        result = serve_socks5(stream, peer_addr, tunnel_addr, db) => result,
        _ = shutdown_token.cancelled() => {
            debug!(%peer_addr, "Connection aborted due to shutdown");
            Ok(())
        }
    }
}

async fn serve_socks5(
    stream: tokio::net::TcpStream,
    peer_addr: SocketAddr,
    tunnel_addr: Option<IpAddr>,
    db: Arc<GeoIpDatabase>,
) -> Result<()> {
    // SOCKS5 handshake: method negotiation (no-auth) followed by command read.
    let (proto, cmd, target_addr) = Socks5ServerProtocol::accept_no_auth(stream)
        .await
        .map_err(|e| anyhow::anyhow!("SOCKS5 handshake failed from {peer_addr}: {e}"))?
        .read_command()
        .await
        .map_err(|e| anyhow::anyhow!("SOCKS5 command read failed from {peer_addr}: {e}"))?;

    // Only TCP CONNECT is supported; BIND and UDP ASSOCIATE are not.
    if cmd != Socks5Command::TCPConnect {
        let _ = proto.reply_error(&ReplyError::CommandNotSupported).await;
        anyhow::bail!("Unsupported SOCKS5 command {cmd:?} from {peer_addr}");
    }

    // Resolve the target to one or more socket addresses.
    let target_addrs: Vec<SocketAddr> = match &target_addr {
        TargetAddr::Ip(addr) => vec![*addr],
        TargetAddr::Domain(host, port) => tokio::net::lookup_host(format!("{host}:{port}"))
            .await
            .map_err(|e| anyhow::anyhow!("DNS lookup failed for {host}:{port}: {e}"))?
            .collect(),
    };

    if target_addrs.is_empty() {
        let _ = proto.reply_error(&ReplyError::HostUnreachable).await;
        anyhow::bail!("No addresses resolved for {target_addr} (from {peer_addr})");
    }

    // Make the routing decision based on the first resolved address.
    // All IPs from the same domain should be in the same country, so checking
    // the first is representative and avoids unnecessary work.
    let first_ip = target_addrs[0].ip();
    let routing = decide_route(first_ip, tunnel_addr, &db);

    // When routing via default interface, pass `None` so sockets bind to INADDR_ANY.
    let effective_tunnel = match routing {
        RoutingDecision::VpnTunnelInterface => tunnel_addr,
        RoutingDecision::DefaultInterface => None,
    };

    debug!(
        %peer_addr,
        %target_addr,
        vpn_tunnel = ?tunnel_addr,
        routing = ?routing,
        "SOCKS5 CONNECT",
    );

    // Connect to the target, binding to the chosen interface.
    let outbound = match connect_to_target(&target_addrs, effective_tunnel).await {
        Ok(s) => s,
        Err(e) => {
            let _ = proto.reply_error(&ReplyError::HostUnreachable).await;
            return Err(e.context(format!(
                "Connect to {target_addr} failed (from {peer_addr})"
            )));
        }
    };

    let local_addr = outbound
        .local_addr()
        .context("Failed to get local address of outbound socket")?;

    // Inform the SOCKS5 client that we've connected successfully.
    let inner = proto
        .reply_success(local_addr)
        .await
        .map_err(|e| anyhow::anyhow!("SOCKS5 reply_success to {peer_addr} failed: {e}"))?;

    // Relay data between client and target until either side closes.
    transfer(inner, outbound).await;

    debug!(%peer_addr, %target_addr, "SOCKS5 connection closed");
    Ok(())
}

async fn connect_to_target(
    addrs: &[SocketAddr],
    tunnel_addr: Option<IpAddr>,
) -> Result<tokio::net::TcpStream> {
    let mut last_err: Option<anyhow::Error> = None;

    for &addr in addrs {
        let socket = match addr {
            SocketAddr::V4(_) => TcpSocket::new_v4().context("Failed to create IPv4 socket")?,
            SocketAddr::V6(_) => TcpSocket::new_v6().context("Failed to create IPv6 socket")?,
        };

        let bind_ip = effective_bind_ip(addr, tunnel_addr);
        if let Err(e) = socket.bind(SocketAddr::new(bind_ip, 0)) {
            debug!("Socket bind to {bind_ip} failed ({e}), using default route");
        }

        match socket.connect(addr).await {
            Ok(stream) => return Ok(stream),
            Err(e) => {
                debug!("Connect to {addr} failed: {e}");
                last_err = Some(anyhow::Error::from(e));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("No addresses to try")))
}

fn effective_bind_ip(target: SocketAddr, tunnel_addr: Option<IpAddr>) -> IpAddr {
    match (target, tunnel_addr) {
        (SocketAddr::V4(_), Some(IpAddr::V4(v4))) => IpAddr::V4(v4),
        (SocketAddr::V6(_), Some(IpAddr::V6(v6))) => IpAddr::V6(v6),
        (SocketAddr::V4(_), _) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        (SocketAddr::V6(_), _) => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
    }
}
