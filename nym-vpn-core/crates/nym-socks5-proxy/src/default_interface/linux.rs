// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{io::Error, mem::size_of_val, os::fd::AsRawFd};

use anyhow::{Result, bail};
use libc::{SO_MARK, SOL_SOCKET, c_int, c_void, socklen_t};
use tokio::net::TcpSocket;

/// Firewall mark used for marking traffic that should bypass the tunnel.
// Copied from nym-vpn-lib to avoid a dependency on the entire crate.
pub const TUNNEL_FWMARK: u32 = 0x14d;

/// Set TUNNEL_FWMARK on the socket so the firewall routes packets through the default interface.
pub fn set_socket_tunnel_fwmark(socket: &TcpSocket) -> Result<()> {
    let fd = socket.as_raw_fd();
    let mark: c_int = TUNNEL_FWMARK as c_int;

    let rc = unsafe {
        libc::setsockopt(
            fd,
            SOL_SOCKET,
            SO_MARK,
            &mark as *const _ as *const c_void,
            size_of_val(&mark) as socklen_t,
        )
    };

    if rc != 0 {
        let err = Error::last_os_error();
        bail!("setsockopt(SO_MARK, {TUNNEL_FWMARK:#x}) failed: {err}");
    }

    Ok(())
}
