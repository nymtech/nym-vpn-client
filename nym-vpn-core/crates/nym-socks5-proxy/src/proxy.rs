// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use crate::{
    default_interface::DefaultInterface,
    file_manager,
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
use nym_file_updater::{FileUpdaterError, FileUpdaterHandle, UpdateOutcome};
use nym_socks5_proxy_ipc::{InterfaceAddresses, ProxyConfig};
use tokio::{
    net::{TcpListener, TcpSocket, TcpStream},
    sync::{mpsc, watch},
};
use tokio_util::sync::CancellationToken;
use url::Url;

const SOCKS5_INITIAL_UPDATE_DELAY: Duration = Duration::from_mins(5);
const SOCKS5_UPDATE_INTERVAL: Duration = Duration::from_hours(8);

pub async fn run(
    config: ProxyConfig,
    default_interface_rx: watch::Receiver<DefaultInterface>,
    tunnel_addrs_rx: watch::Receiver<InterfaceAddresses>,
    excluded_countries_rx: watch::Receiver<Vec<String>>,
    shutdown_token: CancellationToken,
    file_updater_handle: FileUpdaterHandle,
    #[cfg(target_os = "android")] socket_protector: crate::SocketProtector,
) -> Result<()> {
    let listen_addr = SocketAddr::from(([127, 0, 0, 1], config.listen_port));
    let listener = TcpListener::bind(listen_addr)
        .await
        .with_context(|| format!("Failed to bind SOCKS5 listener on {listen_addr}"))?;

    tracing::info!("SOCKS5 proxy listener bound: {listen_addr}");

    // Seed builtin files to disk on first run, for selected countries only.
    file_manager::init_files(&config.data_dir, &config.excluded_countries)
        .await
        .context("Failed to initialise SOCKS5 routing data files")?;

    // Register each selected country's source file with the updater for periodic refresh.
    let excluded_countries = config.excluded_countries.clone();
    let receivers =
        register_country_sources(&config.data_dir, &excluded_countries, &file_updater_handle).await;

    // Load the initial routing database.
    let db = RoutingDatabase::load(&excluded_countries, &config.data_dir)
        .await
        .context("Failed to build routing database")?;
    let (db_tx, db_rx) = watch::channel(Arc::new(db));

    // Background task: reload the routing database whenever a source file is updated, or
    // whenever the daemon reports a change to the excluded countries list.
    // The file_updater_handle is moved here to keep the updater loop alive for the lifetime
    // of this task — dropping it earlier would cause the updater to exit.
    tokio::spawn(handle_db_updates(
        excluded_countries,
        config.data_dir.clone(),
        db_tx,
        receivers,
        excluded_countries_rx,
        file_updater_handle,
        shutdown_token.child_token(),
    ));

    tokio::spawn(accept_loop(
        listener,
        default_interface_rx,
        tunnel_addrs_rx,
        db_rx,
        shutdown_token,
        #[cfg(target_os = "android")]
        socket_protector,
    ));

    Ok(())
}

/// Register each of the given countries' source files with the updater for periodic refresh.
/// Failures to register an individual source are logged and otherwise ignored.
async fn register_country_sources(
    data_dir: &std::path::Path,
    countries: &[String],
    file_updater_handle: &FileUpdaterHandle,
) -> Vec<(
    String,
    mpsc::UnboundedReceiver<Result<UpdateOutcome, FileUpdaterError>>,
)> {
    let mut receivers = Vec::new();

    for source in file_manager::selected_sources(countries) {
        let url = match source.url.parse::<Url>() {
            Ok(url) => url,
            Err(err) => {
                tracing::error!("Invalid SOCKS5 source URL {}: {err}", source.url);
                continue;
            }
        };
        let dest = data_dir.join(source.file_name);
        match file_updater_handle
            .register(
                url,
                dest,
                SOCKS5_INITIAL_UPDATE_DELAY,
                SOCKS5_UPDATE_INTERVAL,
            )
            .await
        {
            Ok(rx) => receivers.push((source.country.to_string(), rx)),
            Err(err) => {
                tracing::error!(
                    "Failed to register SOCKS5 source {} with updater: {err}",
                    source.file_name
                );
            }
        }
    }

    receivers
}

/// Listen for update notifications and reload the routing database when files change, and
/// react to the daemon changing the excluded countries list at runtime.
async fn handle_db_updates(
    mut excluded_countries: Vec<String>,
    data_dir: PathBuf,
    db_tx: watch::Sender<Arc<RoutingDatabase>>,
    mut receivers: Vec<(
        String,
        mpsc::UnboundedReceiver<Result<UpdateOutcome, FileUpdaterError>>,
    )>,
    mut excluded_countries_rx: watch::Receiver<Vec<String>>,
    file_updater_handle: FileUpdaterHandle,
    cancel_token: CancellationToken,
) {
    loop {
        tokio::select! {
            update = recv_any_update(&mut receivers, &cancel_token) => {
                match update {
                    Some(Ok(UpdateOutcome::Updated)) => {
                        // Drain any other notifications from the same update cycle.
                        for (_, rx) in &mut receivers {
                            while rx.try_recv().is_ok() {}
                        }
                        reload_routing_database(&excluded_countries, &data_dir, &db_tx, "after file update").await;
                    }
                    Some(Ok(UpdateOutcome::NotModified)) => {}
                    Some(Err(err)) => {
                        tracing::error!("SOCKS5 updater error: {err}");
                    }
                    None => {
                        tracing::debug!("SOCKS5 updater shutting down");
                        return;
                    }
                }
            }
            changed = excluded_countries_rx.changed() => {
                if changed.is_err() {
                    tracing::debug!("Excluded countries channel closed");
                    return;
                }
                let new_countries = excluded_countries_rx.borrow_and_update().clone();
                if new_countries == excluded_countries {
                    continue;
                }
                tracing::info!(countries = ?new_countries, "Geo-exclusion excluded countries changed");

                let newly_selected: Vec<String> = new_countries
                    .iter()
                    .filter(|c| !excluded_countries.iter().any(|e| e.eq_ignore_ascii_case(c)))
                    .cloned()
                    .collect();

                if !newly_selected.is_empty() {
                    if let Err(err) = file_manager::init_files(&data_dir, &newly_selected).await {
                        tracing::error!(
                            "Failed to seed data files for newly selected countries {newly_selected:?}: {err:#}"
                        );
                    }
                    receivers.extend(
                        register_country_sources(&data_dir, &newly_selected, &file_updater_handle).await,
                    );
                }

                // Countries no longer selected: dropping their receivers automatically
                // unregisters them from periodic refresh.
                receivers.retain(|(country, _)| {
                    new_countries.iter().any(|c| c.eq_ignore_ascii_case(country))
                });

                excluded_countries = new_countries;
                reload_routing_database(&excluded_countries, &data_dir, &db_tx, "after country list change").await;
            }
        }
    }
}

async fn reload_routing_database(
    excluded_countries: &[String],
    data_dir: &std::path::Path,
    db_tx: &watch::Sender<Arc<RoutingDatabase>>,
    reason: &str,
) {
    match RoutingDatabase::load(excluded_countries, data_dir).await {
        Ok(db) => {
            tracing::info!("SOCKS5 routing database reloaded {reason}");
            let _ = db_tx.send(Arc::new(db));
        }
        Err(err) => {
            tracing::error!("Failed to reload SOCKS5 routing database {reason}: {err:#}");
        }
    }
}

/// Poll all receivers concurrently; return the first message that arrives, or None on
/// cancellation. Blocks indefinitely (without returning `None`) while `receivers` is empty,
/// so the caller's other event sources keep being serviced.
async fn recv_any_update(
    receivers: &mut Vec<(
        String,
        mpsc::UnboundedReceiver<Result<UpdateOutcome, FileUpdaterError>>,
    )>,
    cancel_token: &CancellationToken,
) -> Option<Result<UpdateOutcome, FileUpdaterError>> {
    loop {
        if receivers.is_empty() {
            // No sources registered (yet). Wait for cancellation rather than returning
            // None, so the caller's other event sources (e.g. excluded-countries changes)
            // keep being serviced instead of the whole task exiting.
            cancel_token.cancelled().await;
            return None;
        }
        let futs: Vec<_> = receivers
            .iter_mut()
            .map(|(_, rx)| Box::pin(rx.recv()))
            .collect();
        // Destructure inside select! so _rest is dropped before the match below.
        let result = tokio::select! {
            _ = cancel_token.cancelled() => return None,
            (outcome, _idx, _rest) = futures::future::select_all(futs) => outcome,
        };
        match result {
            Some(msg) => return Some(msg),
            None => {
                receivers.retain_mut(|(_, rx)| !rx.is_closed());
            }
        }
    }
}

async fn accept_loop(
    listener: TcpListener,
    default_interface_rx: watch::Receiver<DefaultInterface>,
    tunnel_addrs_rx: watch::Receiver<InterfaceAddresses>,
    db_rx: watch::Receiver<Arc<RoutingDatabase>>,
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
                        // Snapshot the current routing database at accept time.
                        let db = db_rx.borrow().clone();
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

    if routing_decision == RoutingDecision::Reject {
        let _ = proto.reply_error(&ReplyError::NetworkUnreachable).await;
        bail!(
            "Refusing non-excluded {target_addr} (from {peer_addr}): no active VPN tunnel to carry it"
        );
    }

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
