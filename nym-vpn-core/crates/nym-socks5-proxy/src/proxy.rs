// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::Path,
    sync::Arc,
};

use crate::{
    default_interface::DefaultInterface,
    routing::{RoutingDatabase, RoutingDecision, decide_route_for_addrs, is_excluded_domain},
};

#[cfg(target_os = "windows")]
use crate::default_interface::set_socket_interface_index;

#[cfg(target_os = "linux")]
use crate::default_interface::set_socket_tunnel_fwmark;

#[cfg(target_os = "android")]
use std::os::fd::AsRawFd;

use anyhow::{Context, Error, Result, anyhow, bail};
use fast_socks5::{
    ReplyError, Socks5Command,
    server::{Socks5ServerProtocol, transfer},
    util::target_addr::TargetAddr,
};
use nym_socks5_proxy_ipc::{InterfaceAddresses, ProxyConfig};
use tokio::{
    net::{TcpListener, TcpSocket, TcpStream},
    sync::watch,
};
use tokio_util::sync::CancellationToken;

pub async fn run(
    config: ProxyConfig,
    data_dir: &Path,
    default_interface_rx: watch::Receiver<DefaultInterface>,
    tunnel_addrs_rx: watch::Receiver<InterfaceAddresses>,
    shutdown_token: CancellationToken,
    #[cfg(target_os = "android")] socket_protector: crate::SocketProtector,
) -> Result<()> {
    let listen_addr = SocketAddr::from(([127, 0, 0, 1], config.listen_port));
    let listener = TcpListener::bind(listen_addr)
        .await
        .with_context(|| format!("Failed to bind SOCKS5 listener on {listen_addr}"))?;

    tracing::info!("SOCKS5 proxy listener bound: {listen_addr}");

    let db = RoutingDatabase::load(&config.excluded_countries, data_dir)
        .await
        .context("Failed to build routing database")?;
    let db = Arc::new(db);

    tokio::spawn(accept_loop(
        listener,
        default_interface_rx,
        tunnel_addrs_rx,
        db,
        shutdown_token,
        #[cfg(target_os = "android")]
        socket_protector,
    ));

    Ok(())
}

async fn accept_loop(
    listener: TcpListener,
    default_interface_rx: watch::Receiver<DefaultInterface>,
    tunnel_addrs_rx: watch::Receiver<InterfaceAddresses>,
    db: Arc<RoutingDatabase>,
    shutdown_token: CancellationToken,
    #[cfg(target_os = "android")] socket_protector: crate::SocketProtector,
) {
    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, peer_addr)) => {
                        tracing::debug!("Accepted SOCKS5 connection from peer address {peer_addr}");
                        let shutdown = shutdown_token.clone();
                        let tunnel_addrs_rx_clone = tunnel_addrs_rx.clone();
                        let default_interface_rx_clone = default_interface_rx.clone();
                        let db = db.clone();
                        #[cfg(target_os = "android")]
                        let socket_protector = socket_protector.clone();
                        tokio::spawn(async move {
                            if let Err(err) =
                                handle_connection(stream, peer_addr, default_interface_rx_clone, tunnel_addrs_rx_clone, db, shutdown, #[cfg(target_os = "android")] socket_protector).await
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
    stream: TcpStream,
    peer_addr: SocketAddr,
    default_interface_rx: watch::Receiver<DefaultInterface>,
    tunnel_addrs_rx: watch::Receiver<InterfaceAddresses>,
    db: Arc<RoutingDatabase>,
    shutdown_token: CancellationToken,
    #[cfg(target_os = "android")] socket_protector: crate::SocketProtector,
) -> Result<()> {
    // Snapshot both address sets at connection time.
    let tunnel_addrs = tunnel_addrs_rx.borrow().clone();
    let default_interface = default_interface_rx.borrow().clone();

    tokio::select! {
        result = serve_socks5(stream, peer_addr, &default_interface, &tunnel_addrs, db, #[cfg(target_os = "android")] socket_protector) => result,
        _ = shutdown_token.cancelled() => {
            tracing::debug!(%peer_addr, "Connection aborted due to shutdown");
            Ok(())
        }
    }
}

async fn serve_socks5(
    stream: TcpStream,
    peer_addr: SocketAddr,
    default_interface: &DefaultInterface,
    tunnel_addrs: &InterfaceAddresses,
    db: Arc<RoutingDatabase>,
    #[cfg(target_os = "android")] socket_protector: crate::SocketProtector,
) -> Result<()> {
    let (proto, cmd, target_addr) = Socks5ServerProtocol::accept_no_auth(stream)
        .await
        .map_err(|err| anyhow!("SOCKS5 handshake failed from {peer_addr}: {err}"))?
        .read_command()
        .await
        .map_err(|err| anyhow!("SOCKS5 command read failed from {peer_addr}: {err}"))?;

    tracing::debug!("SOCKS5 accept command: {cmd:#x?}, target_addr: {target_addr:#?}");

    // Only TCP CONNECT is supported; BIND and UDP ASSOCIATE are not
    if cmd != Socks5Command::TCPConnect {
        let _ = proto.reply_error(&ReplyError::CommandNotSupported).await;
        bail!("Unsupported SOCKS5 command {cmd:?} from {peer_addr}");
    }

    // Domain-based exclusion check (before DNS to avoid CDN masking of CN origin IPs).
    let domain_excluded = match &target_addr {
        TargetAddr::Domain(host, _) => is_excluded_domain(host, &db.domain),
        TargetAddr::Ip(_) => false,
    };

    // Look-up target addresses
    let target_addrs: Vec<SocketAddr> = match &target_addr {
        TargetAddr::Ip(addr) => vec![*addr],
        TargetAddr::Domain(host, port) => tokio::net::lookup_host(format!("{host}:{port}"))
            .await
            .map_err(|err| anyhow!("DNS lookup failed for {host}:{port}: {err}"))?
            .collect(),
    };

    if target_addrs.is_empty() {
        let _ = proto.reply_error(&ReplyError::HostUnreachable).await;
        bail!("No addresses resolved for {target_addr} (from {peer_addr})");
    }

    let routing_decision = if domain_excluded {
        RoutingDecision::DefaultInterface
    } else {
        decide_route_for_addrs(&target_addrs, tunnel_addrs, &db.geo_ip)
    };

    tracing::info!("SOCKS5 CONNECT {peer_addr} -> {target_addr}");
    tracing::debug!(
        "domain_excluded={domain_excluded} target_addrs={target_addrs:?} \
         routing_decision={routing_decision:?} \
         default_interface={default_interface:?} tunnel_addrs={tunnel_addrs:?}"
    );

    let outbound = match connect_to_target(
        if routing_decision == RoutingDecision::DefaultInterface {
            Some(default_interface)
        } else {
            None
        },
        &target_addrs,
        #[cfg(target_os = "android")]
        &socket_protector,
    )
    .await
    {
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
        .map_err(|err| anyhow!("SOCKS5 reply_success to {peer_addr} failed: {err}"))?;

    transfer(inner, outbound).await;

    tracing::info!("SOCKS5 connection {peer_addr} -> {target_addr} closed");

    Ok(())
}

// If default_interface is Some, then we attempt to bind to the default interface,
// else we bind to the INADDR_ANY and allow routing to direct traffic through the VPN tunnel.
async fn connect_to_target(
    default_interface: Option<&DefaultInterface>,
    addrs: &[SocketAddr],
    #[cfg(target_os = "android")] socket_protector: &crate::SocketProtector,
) -> Result<TcpStream> {
    let mut last_err: Option<Error> = None;

    for &addr in addrs {
        let socket = match addr {
            SocketAddr::V4(_) => TcpSocket::new_v4().context("Failed to create IPv4 socket")?,
            SocketAddr::V6(_) => TcpSocket::new_v6().context("Failed to create IPv6 socket")?,
        };

        // On Linux, in order to force traffic via the default interface, set the SPLIT_TUNNEL_MARK on the socket.
        #[cfg(target_os = "linux")]
        if default_interface.is_some()
            && let Err(err) = set_socket_tunnel_fwmark(&socket)
        {
            tracing::warn!("Failed to set split tunnel mark on socket: {err}");
            last_err = Some(err);
            continue;
        }

        // On Windows, in order to force the socket to bind to the default interface, set the interface index on the socket.
        #[cfg(target_os = "windows")]
        if let Some(default_interface) = default_interface
            && let Err(err) = set_socket_interface_index(&socket, default_interface, addr)
        {
            tracing::warn!("Failed to set interface index on socket: {err:#}");
            last_err = Some(err);
            continue;
        }

        // On Android, protect the socket from VPN routing so excluded traffic reaches the default interface.
        #[cfg(target_os = "android")]
        if default_interface.is_some() {
            socket_protector(socket.as_raw_fd());
        }

        // On macOS, we only need to bind using the correct bind address.

        // bind_addr will always be None on Linux as we don't monitor the default interface at all.
        let bind_addr: Option<IpAddr> = if let Some(default_interface) = default_interface {
            match addr {
                SocketAddr::V4(_) => default_interface.v4_addr.map(IpAddr::V4),
                SocketAddr::V6(_) => default_interface.v6_addr.map(IpAddr::V6),
            }
        } else {
            None
        };

        let bind_ip = match (addr, bind_addr) {
            (SocketAddr::V4(_), Some(IpAddr::V4(v4))) => IpAddr::V4(v4),
            (SocketAddr::V6(_), Some(IpAddr::V6(v6))) => IpAddr::V6(v6),
            (SocketAddr::V4(_), _) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            (SocketAddr::V6(_), _) => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
        };

        if let Err(err) = socket.bind(SocketAddr::new(bind_ip, 0)) {
            tracing::warn!("Socket bind to {bind_ip} failed: {err}");
            last_err = Some(Error::from(err));
            continue;
        }

        match socket.connect(addr).await {
            Ok(stream) => return Ok(stream),
            Err(err) => {
                tracing::warn!("Connect to {addr} failed: {err}");
                last_err = Some(Error::from(err));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow!("Failed to connect to target")))
}
