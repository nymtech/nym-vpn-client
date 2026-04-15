// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::Path,
    sync::Arc,
};

use crate::routing::{GeoIpDatabase, RoutingDecision, decide_route};

#[cfg(target_os = "windows")]
use super::windows_bind::bind_by_interface_index;

use nym_socks5_proxy_ipc::ProxyConfig;

use anyhow::{Context, Result};
use fast_socks5::{
    ReplyError, Socks5Command,
    server::{Socks5ServerProtocol, transfer},
    util::target_addr::TargetAddr,
};
use tokio::{
    net::{TcpListener, TcpSocket, TcpStream},
    sync::watch,
};
use tokio_util::sync::CancellationToken;

pub async fn run(
    config: ProxyConfig,
    data_dir: &Path,
    default_addr_rx: watch::Receiver<Option<IpAddr>>,
    tunnel_addr_rx: watch::Receiver<Option<IpAddr>>,
    shutdown_token: CancellationToken,
) -> Result<()> {
    let listen_addr = SocketAddr::from(([127, 0, 0, 1], config.listen_port));
    let listener = TcpListener::bind(listen_addr)
        .await
        .with_context(|| format!("Failed to bind SOCKS5 listener on {listen_addr}"))?;

    tracing::info!(%listen_addr, "SOCKS5 proxy listener bound");

    let db = GeoIpDatabase::load(&config.excluded_countries, data_dir)
        .await
        .context("Failed to build GeoIP database")?;
    let db = Arc::new(db);

    tokio::spawn(accept_loop(
        listener,
        default_addr_rx,
        tunnel_addr_rx,
        db,
        shutdown_token,
    ));

    Ok(())
}

async fn accept_loop(
    listener: TcpListener,
    default_addr_rx: watch::Receiver<Option<IpAddr>>,
    tunnel_addr_rx: watch::Receiver<Option<IpAddr>>,
    db: Arc<GeoIpDatabase>,
    shutdown_token: CancellationToken,
) {
    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, peer_addr)) => {
                        tracing::debug!(%peer_addr, "Accepted SOCKS5 connection");
                        let shutdown = shutdown_token.clone();
                        let tunnel_addr_rx_clone = tunnel_addr_rx.clone();
                        let default_addr_rx_clone = default_addr_rx.clone();
                        let db = db.clone();
                        tokio::spawn(async move {
                            if let Err(err) =
                                handle_connection(stream, peer_addr, default_addr_rx_clone, tunnel_addr_rx_clone, db, shutdown).await
                            {
                                tracing::warn!(%peer_addr, "SOCKS5 connection error: {err:#}");
                            }
                        });
                    }
                    Err(err) => {
                        tracing::warn!("SOCKS5 accept error: {err}");
                    }
                }
            }
            _ = shutdown_token.cancelled() => {
                tracing::info!("SOCKS5 accept loop shutting down");
                break;
            }
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    peer_addr: SocketAddr,
    default_addr_rx: watch::Receiver<Option<IpAddr>>,
    tunnel_addr_rx: watch::Receiver<Option<IpAddr>>,
    db: Arc<GeoIpDatabase>,
    shutdown_token: CancellationToken,
) -> Result<()> {
    // Snapshot the current addresses at connection time.
    let tunnel_addr = *tunnel_addr_rx.borrow();
    let default_addr = *default_addr_rx.borrow();

    tokio::select! {
        result = serve_socks5(stream, peer_addr, default_addr, tunnel_addr, db) => result,
        _ = shutdown_token.cancelled() => {
            tracing::debug!(%peer_addr, "Connection aborted due to shutdown");
            Ok(())
        }
    }
}

async fn serve_socks5(
    stream: tokio::net::TcpStream,
    peer_addr: SocketAddr,
    default_addr: Option<IpAddr>,
    tunnel_addr: Option<IpAddr>,
    db: Arc<GeoIpDatabase>,
) -> Result<()> {
    // SOCKS5 handshake: method negotiation (no-auth) followed by command read.
    let (proto, cmd, target_addr) = Socks5ServerProtocol::accept_no_auth(stream)
        .await
        .map_err(|err| anyhow::anyhow!("SOCKS5 handshake failed from {peer_addr}: {err}"))?
        .read_command()
        .await
        .map_err(|err| anyhow::anyhow!("SOCKS5 command read failed from {peer_addr}: {err}"))?;

    tracing::debug!("SOCKS5 accept command: {cmd:#x?}, target_addr: {target_addr:#?}");

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

    tracing::debug!("Routing target_addr {target_addr} via {routing:?}");

    let bind_addr: Option<IpAddr> = match routing {
        RoutingDecision::VpnTunnelInterface => None,
        RoutingDecision::DefaultInterface => default_addr,
    };

    tracing::debug!(
        "SOCKS5 CONNECT. peer_addr: {peer_addr}, target_addr: {target_addr}, default_addr: {default_addr:?}, tunnel_addr: {tunnel_addr:?}, routing: {routing:?}"
    );

    let outbound = match connect_to_target(&target_addrs, bind_addr).await {
        Ok(s) => s,
        Err(err) => {
            let _ = proto.reply_error(&ReplyError::HostUnreachable).await;
            return Err(err.context(format!(
                "Connect to {target_addr} failed (from {peer_addr})"
            )));
        }
    };

    let local_addr = outbound
        .local_addr()
        .context("Failed to get local address of outbound socket")?;

    let inner = proto
        .reply_success(local_addr)
        .await
        .map_err(|e| anyhow::anyhow!("SOCKS5 reply_success to {peer_addr} failed: {e}"))?;

    transfer(inner, outbound).await;

    tracing::debug!(%peer_addr, %target_addr, "SOCKS5 connection closed");
    Ok(())
}

async fn connect_to_target(addrs: &[SocketAddr], bind_addr: Option<IpAddr>) -> Result<TcpStream> {
    let mut last_err: Option<anyhow::Error> = None;

    for &addr in addrs {
        let socket = match addr {
            SocketAddr::V4(_) => TcpSocket::new_v4().context("Failed to create IPv4 socket")?,
            SocketAddr::V6(_) => TcpSocket::new_v6().context("Failed to create IPv6 socket")?,
        };

        bind_socket_for_routing(&socket, addr, bind_addr);

        match socket.connect(addr).await {
            Ok(stream) => return Ok(stream),
            Err(err) => {
                tracing::error!("Connect to {addr} failed: {err}");
                last_err = Some(anyhow::Error::from(err));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("No addresses to try")))
}

fn bind_socket_for_routing(socket: &TcpSocket, target: SocketAddr, bind_addr: Option<IpAddr>) {
    #[cfg(windows)]
    if let Some(ip) = bind_addr {
        match bind_by_interface_index(socket, ip, target) {
            Ok(()) => return,
            Err(err) => {
                tracing::warn!("IP_UNICAST_IF binding failed: {err:#}; falling back to bind-by-IP")
            }
        }
    }

    // Non-Windows, or Windows fallback: bind by source IP.
    let bind_ip = match (target, bind_addr) {
        (SocketAddr::V4(_), Some(IpAddr::V4(v4))) => IpAddr::V4(v4),
        (SocketAddr::V6(_), Some(IpAddr::V6(v6))) => IpAddr::V6(v6),
        (SocketAddr::V4(_), _) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        (SocketAddr::V6(_), _) => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
    };

    if let Err(err) = socket.bind(SocketAddr::new(bind_ip, 0)) {
        tracing::error!("Socket bind to {bind_ip} failed: {err}; using default route");
    }
}
