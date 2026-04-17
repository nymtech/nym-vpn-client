// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::os::fd::AsRawFd;

use nym_firewall_config::SPLIT_TUNNEL_MARK;
use tokio::net::TcpSocket;

pub fn set_socket_mark(socket: &TcpSocket) {
    let fd = socket.as_raw_fd();
    let mark: libc::c_int = SPLIT_TUNNEL_MARK as libc::c_int;

    let rc = unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_MARK,
            &mark as *const _ as *const libc::c_void,
            std::mem::size_of_val(&mark) as libc::socklen_t,
        )
    };

    if rc != 0 {
        let err = std::io::Error::last_os_error();
        tracing::warn!(
            "setsockopt(SO_MARK, {SPLIT_TUNNEL_MARK:#x}) failed: {err}; falling back to bind-based routing"
        );
    }
}
