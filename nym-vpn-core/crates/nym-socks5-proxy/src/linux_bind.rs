// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{io::Error, mem::size_of_val, os::fd::AsRawFd};

use libc::{SO_MARK, SOL_SOCKET, c_int, c_void, socklen_t};
use nym_firewall_config::SPLIT_TUNNEL_MARK;
use tokio::net::TcpSocket;

pub fn set_split_tunnel_mark(socket: &TcpSocket) {
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

    if rc == 0 {
        tracing::debug!("Set SO_MARK={SPLIT_TUNNEL_MARK:#x} on socket successfully");
    } else {
        let err = Error::last_os_error();
        tracing::warn!(
            "setsockopt(SO_MARK, {SPLIT_TUNNEL_MARK:#x}) failed: {err}; falling back to bind-based routing"
        );
    }
}
