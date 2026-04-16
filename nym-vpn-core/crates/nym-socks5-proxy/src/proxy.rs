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

use nym_socks5_proxy_ipc::{InterfaceAddresses, ProxyConfig};

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
    default_addrs_rx: watch::Receiver<InterfaceAddresses>,
    tunnel_addrs_rx: watch::Receiver<InterfaceAddresses>,
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
        default_addrs_rx,
        tunnel_addrs_rx,
        db,
        shutdown_token,
    ));

    Ok(())
}

async fn accept_loop(
    listener: TcpListener,
    default_addrs_rx: watch::Receiver<InterfaceAddresses>,
    tunnel_addrs_rx: watch::Receiver<InterfaceAddresses>,
    db: Arc<GeoIpDatabase>,
    shutdown_token: CancellationToken,
) {
    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, peer_addr)) => {
                        tracing::debug!("Accepted SOCKS5 connection from peer address {peer_addr}");
                        let shutdown = shutdown_token.clone();
                        let tunnel_addrs_rx = tunnel_addrs_rx.clone();
                        let default_addrs_rx = default_addrs_rx.clone();
                        let db = db.clone();
                        tokio::spawn(async move {
                            if let Err(err) =
                                handle_connection(stream, peer_addr, default_addrs_rx, tunnel_addrs_rx, db, shutdown).await
                            {
                                tracing::warn!("SOCKS5 connection error for peer address {peer_addr}: {err:#}");
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
    default_addrs_rx: watch::Receiver<InterfaceAddresses>,
    tunnel_addrs_rx: watch::Receiver<InterfaceAddresses>,
    db: Arc<GeoIpDatabase>,
    shutdown_token: CancellationToken,
) -> Result<()> {
    // Snapshot both address sets at connection time.
    let tunnel_addrs = tunnel_addrs_rx.borrow().clone();
    let default_addrs = default_addrs_rx.borrow().clone();

    tokio::select! {
        result = serve_socks5(stream, peer_addr, &default_addrs, &tunnel_addrs, db) => result,
        _ = shutdown_token.cancelled() => {
            tracing::debug!(%peer_addr, "Connection aborted due to shutdown");
            Ok(())
        }
    }
}

async fn serve_socks5(
    stream: tokio::net::TcpStream,
    peer_addr: SocketAddr,
    default_addrs: &InterfaceAddresses,
    tunnel_addrs: &InterfaceAddresses,
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

    // Routing decision is based on the first resolved address.
    let first_ip = target_addrs[0].ip();
    let routing = decide_route(first_ip, tunnel_addrs, &db);

    // For CN (excluded-country) traffic we bind to the matching default-interface
    // address so the OS routes it directly over the physical interface.
    // For VPN traffic we bind to INADDR_ANY and let the kernel route it through
    // the tunnel interface automatically.
    let bind_addr: Option<IpAddr> = match routing {
        RoutingDecision::DefaultInterface => match first_ip {
            IpAddr::V4(_) => default_addrs.v4_addr.map(IpAddr::V4),
            IpAddr::V6(_) => default_addrs.v6_addr.map(IpAddr::V6),
        },
        RoutingDecision::VpnTunnelInterface => None,
    };

    tracing::debug!(
        "SOCKS5 CONNECT peer={peer_addr} target={target_addr} routing={routing:?} \
         bind={bind_addr:?} default={default_addrs:?} tunnel={tunnel_addrs:?}"
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
    #[cfg(target_os = "windows")]
    if let Some(ip) = bind_addr {
        match bind_by_interface_index(socket, ip, target) {
            Ok(()) => return,
            Err(err) => {
                tracing::warn!("IP_UNICAST_IF binding failed: {err:#}; falling back to bind-by-IP")
            }
        }
    }

    // Bind to the chosen source address, or INADDR_ANY / IN6ADDR_ANY when
    // there is no explicit bind (VPN tunnel path — kernel does the routing).
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
