// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
use std::os::fd::BorrowedFd;
#[cfg(target_os = "android")]
use std::sync::Arc;
use std::time::Duration;

#[cfg(target_os = "linux")]
use nix::sys::socket::{SetSockOpt, sockopt::Mark};
use nym_bridges::types::TransportAssociation;
use nym_vpn_lib_types::{BridgeAddress, BridgeTransport};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{Result, tunnel::SelectedGateways, tunnel::transports};

/// Cap on waiting for a generation's forwarder to exit gracefully before aborting it.
const GRACEFUL_EXIT_TIMEOUT: Duration = Duration::from_secs(3);

/// Connects a bridge transport and launches its UDP forwarder. `bind_addr` is `None` on the
/// first attempt and `Some(listen_addr)` on every redial, so wireguard-go's peer endpoint
/// never changes. Takes owned values instead of a `TunnelMonitor` reference so a spawned redial
/// task can call it.
#[allow(clippy::too_many_arguments)]
pub(super) async fn connect_and_launch_bridge(
    quic: &[transports::ClientConfig],
    fallback: &[transports::ClientConfig],
    bind_addr: Option<std::net::SocketAddr>,
    close_tx: mpsc::UnboundedSender<()>,
    shutdown_token: &CancellationToken,
    #[cfg(target_os = "linux")] fwmark: u32,
    #[cfg(target_os = "android")] tun_provider: Arc<dyn crate::tunnel_provider::AndroidTunProvider>,
) -> Result<(BridgeAddress, JoinHandle<()>)> {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    let on_bridge_socket_open = move |fd| {
        #[cfg(target_os = "android")]
        {
            tracing::debug!("Bypass bridge socket");
            tun_provider.bypass(fd);
        }

        #[cfg(target_os = "linux")]
        {
            tracing::debug!("Bypass bridge socket");
            let borrowed_fd = unsafe { &BorrowedFd::borrow_raw(fd) };
            if let Err(err) = Mark.set(borrowed_fd, &fwmark) {
                tracing::error!("Could not set fwmark for bridge socket fd: {err}");
            }
        }
    };
    // QUIC first: TLS and SSH run over TCP and are slower, so they are raced only once QUIC fails.
    let quic_timeout = if fallback.is_empty() {
        transports::BRIDGE_CONNECT_TIMEOUT
    } else {
        transports::QUIC_FALLBACK_TIMEOUT
    };
    let bridge_conn = match transports::BridgeConn::try_connect_parallel(
        quic.to_vec(),
        shutdown_token.child_token(),
        #[cfg(any(target_os = "linux", target_os = "android"))]
        &on_bridge_socket_open,
        Some(quic_timeout),
    )
    .await
    {
        Ok(conn) => conn,
        Err(err) if fallback.is_empty() => return Err(err.into()),
        Err(err) => {
            tracing::warn!("QUIC bridge failed, trying TLS and SSH: {err}");
            transports::BridgeConn::try_connect_parallel(
                fallback.to_vec(),
                shutdown_token.child_token(),
                #[cfg(any(target_os = "linux", target_os = "android"))]
                &on_bridge_socket_open,
                Some(transports::BRIDGE_CONNECT_TIMEOUT),
            )
            .await?
        }
    };

    tracing::info!(
        "{} transport connected",
        bridge_conn.params().transport_name()
    );

    let remote_addr = bridge_conn.endpoint();
    let transport = match bridge_conn.params() {
        transports::ClientConfig::QuicPlain(_) => BridgeTransport::Quic,
        transports::ClientConfig::TlsPlain(_) => BridgeTransport::Tls,
        transports::ClientConfig::SshPlain(_) => BridgeTransport::Ssh,
    };
    let (listen_addr, join_handle) = transports::UdpForwarder::launch_initiator(
        bridge_conn,
        bind_addr,
        Some(close_tx),
        shutdown_token.child_token(),
        Some(transports::INITIAL_FWD_CONNECTION_TIMEOUT),
    )
    .await?;

    tracing::info!("bridge udp forwarder open on {listen_addr}",);

    Ok((
        BridgeAddress {
            listen_addr,
            remote_addr,
            transport,
        },
        join_handle,
    ))
}

/// Connects the bridge, then hands off to a task that quietly redials it in place - on device
/// wake or on the transport dying - without touching the tunnel state machine. Only a failed
/// redial falls back to today's behaviour: signal `bridge_close_tx`.
#[allow(clippy::too_many_arguments)]
pub(super) async fn start_bridges(
    selected_gateways: &SelectedGateways,
    bridge_fallback: bool,
    shutdown_token: CancellationToken,
    bridge_close_tx: mpsc::UnboundedSender<()>,
    #[cfg(target_os = "linux")] fwmark: u32,
    #[cfg(target_os = "android")] tun_provider: Arc<dyn crate::tunnel_provider::AndroidTunProvider>,
) -> Result<(BridgeAddress, JoinHandle<()>)> {
    let (quic, fallback) = selected_gateways
        .entry_gateway()
        .bridge_candidates(bridge_fallback);
    let ipv4_only = |params: Vec<_>| -> Vec<_> {
        params
            .into_iter()
            .map(transports::filter_bridge_params_ipv4_only)
            .collect()
    };
    let (quic, fallback) = (ipv4_only(quic), ipv4_only(fallback));

    tracing::info!("Establishing DVPN bridge tunnel");

    // Each generation of the connection gets its own child token, so a redial can ask just
    // that generation to exit, not the whole bridge subsystem.
    let mut bridge_shutdown_token = shutdown_token.child_token();

    let (inner_close_tx, mut inner_close_rx) = mpsc::unbounded_channel();
    let (bridge_addr, mut join_handle) = connect_and_launch_bridge(
        &quic,
        &fallback,
        None,
        inner_close_tx,
        &bridge_shutdown_token,
        #[cfg(target_os = "linux")]
        fwmark,
        #[cfg(target_os = "android")]
        tun_provider.clone(),
    )
    .await?;

    let listen_addr = bridge_addr.listen_addr;
    let supervisor = tokio::spawn(async move {
        let wake = crate::wake_signal::subscribe();
        loop {
            tokio::select! {
                _ = wake.notified() => {
                    tracing::info!("Device woke up, redialing bridge transport");
                }
                _ = inner_close_rx.recv() => {
                    tracing::info!("Bridge transport closed, redialing");
                }
                _ = shutdown_token.cancelled() => {
                    return;
                }
            }
            // A cancelled token can't be reused, so the next generation gets a fresh one.
            bridge_shutdown_token.cancel();
            match tokio::time::timeout(GRACEFUL_EXIT_TIMEOUT, &mut join_handle).await {
                Ok(Err(err)) if err.is_panic() => {
                    tracing::error!("bridge forwarder task panicked: {err}");
                }
                Ok(_) => {}
                Err(_) => {
                    tracing::warn!("bridge forwarder did not exit gracefully, aborting");
                    join_handle.abort();
                }
            }
            bridge_shutdown_token = shutdown_token.child_token();

            let (new_close_tx, new_close_rx) = mpsc::unbounded_channel();
            match connect_and_launch_bridge(
                &quic,
                &fallback,
                Some(listen_addr),
                new_close_tx,
                &bridge_shutdown_token,
                #[cfg(target_os = "linux")]
                fwmark,
                #[cfg(target_os = "android")]
                tun_provider.clone(),
            )
            .await
            {
                Ok((_bridge_addr, new_handle)) => {
                    join_handle = new_handle;
                    inner_close_rx = new_close_rx;
                }
                Err(err) => {
                    tracing::warn!("Bridge redial failed, giving up: {err}");
                    let _ = bridge_close_tx.send(());
                    return;
                }
            }
        }
    });

    Ok((bridge_addr, supervisor))
}
