// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::{HashMap, hash_map::Entry},
    os::fd::{AsFd, BorrowedFd, OwnedFd},
};

use nix::{
    Result,
    sys::socket::{AddressFamily, SockFlag, SockType, socket},
};

/// Control sockets connection manager
#[derive(Debug, Default)]
pub struct CtlSockets {
    inner: HashMap<AddressFamily, OwnedFd>,
}

impl CtlSockets {
    /// Returns control socket for IPv4 family.
    pub fn ctl_socket_v4<'a>(&'a mut self) -> Result<BorrowedFd<'a>> {
        self.ctl_socket(AddressFamily::Inet)
    }

    /// Returns control socket for IPv6 family.
    pub fn ctl_socket_v6<'a>(&'a mut self) -> Result<BorrowedFd<'a>> {
        self.ctl_socket(AddressFamily::Inet6)
    }

    /// Returns existing control socket for the given address family.
    /// If the socket does not exist, it will be created.
    pub fn ctl_socket<'a>(&'a mut self, family: AddressFamily) -> Result<BorrowedFd<'a>> {
        if let Entry::Vacant(e) = self.inner.entry(family) {
            let sock = Self::open_ctl_socket(family)?;
            e.insert(sock);
        }

        Ok(self
            .inner
            .get(&family)
            .expect("ctl_socket cannot be unset")
            .as_fd())
    }

    /// Create new control socket
    pub fn open_ctl_socket(family: AddressFamily) -> Result<OwnedFd> {
        let sock = socket(family, SockType::Datagram, SockFlag::empty(), None)?;
        Ok(sock)
    }
}
