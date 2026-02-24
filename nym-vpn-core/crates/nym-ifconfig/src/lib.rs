// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Network interface configuration

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod apple;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;

#[cfg(target_os = "macos")]
pub use apple::session::{
    AddAddressRequest, AddAddressRequestV4, AddAddressRequestV6, InterfaceIpAddrEntry,
    Ipv6AddrFlags, Ipv6AddrLifetime, ND6_INFINITE_LIFETIME, ND6_MAX_LIFETIME, Nd6Flags, Nd6Info,
    Session,
};
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use apple::utun::Utun;

#[cfg(target_os = "linux")]
pub use linux::session::{InterfaceIpAddrEntry, Session};
#[cfg(any(target_os = "linux", target_os = "android"))]
pub use linux::tun::{Tun, TunError, TunErrorKind};

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod copy_into;

#[cfg(not(windows))]
mod error;
#[cfg(not(windows))]
pub use error::{Error, ErrorKind, Result};
