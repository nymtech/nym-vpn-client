// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::os::fd::BorrowedFd;

/// Returns tunnel interface name for the given tunnel file descriptor.
#[cfg(target_os = "ios")]
pub fn get_tun_name(fd: BorrowedFd) -> nym_ifconfig::Result<String> {
    let tun = nym_ifconfig::Utun::new_from_borrowed_fd(fd);
    tun.name()
}

/// Returns tunnel interface name for the given tunnel file descriptor.
#[cfg(target_os = "android")]
pub fn get_tun_name(fd: BorrowedFd) -> nym_ifconfig::Result<String> {
    let tun = nym_ifconfig::Tun::new_from_borrowed_fd(fd);
    tun.name()
}
