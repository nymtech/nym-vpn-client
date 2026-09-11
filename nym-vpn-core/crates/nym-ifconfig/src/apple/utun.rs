// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::os::fd::{AsFd, BorrowedFd};
#[cfg(target_os = "macos")]
use std::os::fd::{AsRawFd, OwnedFd};

#[cfg(target_os = "macos")]
use nix::sys::socket::{AddressFamily, SockFlag, SockProtocol, SockType, SysControlAddr};
use nix::sys::{socket, socket::sockopt::UtunIfname};

use crate::{Error, ErrorKind, Result};

// Name registered by the utun kernel control
// usr/include/net/if_utun.h
#[cfg(target_os = "macos")]
const UTUN_CONTROL_NAME: &str = "com.apple.net.utun_control";

#[derive(Debug)]
pub struct Utun<'a, T: AsFd> {
    tun_fd: T,
    _phantom: std::marker::PhantomData<&'a ()>,
}

#[cfg(target_os = "macos")]
impl Utun<'_, OwnedFd> {
    /// Create a new tunnel instance.
    pub fn new() -> Result<Self> {
        let tun_fd = socket::socket(
            AddressFamily::System,
            SockType::Datagram,
            SockFlag::empty(),
            SockProtocol::KextControl,
        )?;

        let ctl_addr = SysControlAddr::from_name(tun_fd.as_raw_fd(), UTUN_CONTROL_NAME, 0)?;

        socket::connect(tun_fd.as_raw_fd(), &ctl_addr)?;

        Ok(Self {
            tun_fd,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Consume `Utun` and return the tunnel file descriptor.
    pub fn into_owned_fd(self) -> OwnedFd {
        self.tun_fd
    }
}

impl<'a> Utun<'a, BorrowedFd<'a>> {
    /// Initialize `Utun` using borrowed tunnel file descriptor.
    pub fn new_from_borrowed_fd(tun_fd: BorrowedFd<'a>) -> Utun<'a, BorrowedFd<'a>> {
        Self {
            tun_fd,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: AsFd> Utun<'_, T> {
    /// Get tunnel interface name
    pub fn name(&self) -> Result<String> {
        socket::getsockopt(&self.tun_fd, UtunIfname)?
            .into_string()
            .map_err(|e| Error::new(ErrorKind::ConvertInterfaceNameIntoString, Box::new(e)))
    }
}

impl<'a, T: AsFd> AsFd for Utun<'a, T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.tun_fd.as_fd()
    }
}
