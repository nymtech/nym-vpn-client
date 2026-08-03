// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::SocketAddr;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::net::TcpListener;

const DEFAULT_BACKLOG: i32 = 128;

pub fn new_tcp_listener(socket_addr: SocketAddr, reuse_addr: bool) -> std::io::Result<TcpListener> {
    let domain = Domain::for_address(socket_addr);
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP)).inspect_err(|err| {
        tracing::warn!("Failed to open TCP socket: {err}");
    })?;

    // SO_NONBLOCK is required for turning this into a tokio socket.
    socket.set_nonblocking(true).inspect_err(|err| {
        tracing::warn!("Failed to set TCP socket as nonblocking: {err}");
    })?;

    // SO_REUSEADDR allows us to bind to `127.x.y.z` even if another socket is bound to `0.0.0.0`.
    // Best-effort: allow binding even if wildcard is in use. Windows semantics differ but
    // this is harmless.
    if reuse_addr {
        socket.set_reuse_address(true).inspect_err(|err| {
            tracing::warn!("Failed to set SO_REUSEADDR on TCP socket: {err}");
        })?;
    }

    let sa = SockAddr::from(socket_addr);
    socket.bind(&sa).inspect_err(|err| {
        tracing::warn!("Failed to bind TCP socket to {socket_addr}: {err}");
    })?;

    socket.listen(DEFAULT_BACKLOG).inspect_err(|err| {
        tracing::warn!("Failed to listen TCP socket: {err}");
    })?;

    let tcp_listener =
        TcpListener::from_std(std::net::TcpListener::from(socket)).expect("socket is non-blocking");

    Ok(tcp_listener)
}
