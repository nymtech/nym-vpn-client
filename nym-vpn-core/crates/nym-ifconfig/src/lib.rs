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

#[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
mod copy_into;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ErrorKind {
    Io,
    InvalidAddress,
    InvalidPrefixLength,
    ConvertInterfaceNameIntoString,
    #[cfg(target_os = "linux")]
    InterfaceNotFound,
    #[cfg(target_os = "linux")]
    Netlink,
    #[cfg(target_os = "linux")]
    AddrFamilyMismatch,
    #[cfg(target_os = "linux")]
    MtuNotFound,
}

type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    source: Option<BoxedError>,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, source: BoxedError) -> Self {
        Self {
            kind,
            source: Some(source),
        }
    }

    #[allow(unused)]
    pub(crate) fn without_source(kind: ErrorKind) -> Self {
        Self { kind, source: None }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn io_error(&self) -> Option<std::io::Error> {
        self.source
            .as_ref()
            .and_then(|v| v.downcast_ref::<nix::Error>())
            .map(|err| std::io::Error::from(*err))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.kind {
            ErrorKind::Io => "Io error",
            ErrorKind::InvalidAddress => "Invalid address",
            ErrorKind::InvalidPrefixLength => "Invalid prefix length",
            ErrorKind::ConvertInterfaceNameIntoString => {
                "Failed to convert interface name into utf8 string"
            }
            #[cfg(target_os = "linux")]
            ErrorKind::InterfaceNotFound => "Interface not found",
            #[cfg(target_os = "linux")]
            ErrorKind::Netlink => "Netlink error",
            #[cfg(target_os = "linux")]
            ErrorKind::AddrFamilyMismatch => "Address family mismatch",
            #[cfg(target_os = "linux")]
            ErrorKind::MtuNotFound => "MTU not found",
        })
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Some(err) = self.source.as_ref() {
            Some(err.as_ref())
        } else {
            None
        }
    }
}

#[cfg(unix)]
impl From<nix::Error> for Error {
    fn from(value: nix::Error) -> Self {
        Self::new(ErrorKind::Io, Box::new(value))
    }
}

#[cfg(target_os = "linux")]
impl From<rtnetlink::Error> for Error {
    fn from(value: rtnetlink::Error) -> Self {
        Self::new(ErrorKind::Netlink, Box::new(value))
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
