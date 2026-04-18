// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{io::Error, mem::size_of_val, os::fd::AsRawFd};

use anyhow::{Result, bail};
use libc::{SO_MARK, SOL_SOCKET, c_int, c_void, socklen_t};
use nym_firewall_config::SPLIT_TUNNEL_MARK;
use tokio::net::TcpSocket;

/// Set SPLIT_TUNNEL_MARK on the socket so the firewall routes packets through the default interface.
pub fn set_socket_split_tunnel_mark(socket: &TcpSocket) -> Result<()> {
    let fd = socket.as_raw_fd();
    let mark: c_int = SPLIT_TUNNEL_MARK as c_int;

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
        bail!("setsockopt(SO_MARK, {SPLIT_TUNNEL_MARK:#x}) failed: {err}");
    }

    Ok(())
}
