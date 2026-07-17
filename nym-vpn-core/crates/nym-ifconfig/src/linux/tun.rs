// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
use std::os::fd::OwnedFd;
use std::{
    ffi::CStr,
    os::fd::{AsFd, AsRawFd, BorrowedFd},
};

#[cfg(any(target_os = "android", target_os = "linux"))]
use nix::libc::ifreq;
#[cfg(target_os = "linux")]
use nix::{fcntl, libc::IFNAMSIZ, net::if_::InterfaceFlags};

use super::sys::*;
#[cfg(target_os = "linux")]
use crate::copy_into::CopyInto;

#[derive(Debug)]
pub struct Tun<'a, T: AsFd> {
    tun_fd: T,
    _phantom: std::marker::PhantomData<&'a ()>,
}

#[cfg(target_os = "linux")]
impl Tun<'_, OwnedFd> {
    /// Create new TUN interface.
    pub fn new() -> Result<Self> {
        Self::new_with_parameters(None, InterfaceFlags::IFF_TUN)
    }

    /// Create new TUN interface with interface flags.
    ///
    /// # Example
    ///
    /// ```rs
    /// use nix::net::if_::InterfaceFlags;
    /// let tun = Tun::new_with_parameters(None, InterfaceFlags::IFF_TUN | InterfaceFlags::IFF_NO_PI).unwrap();
    /// ```
    ///
    pub fn new_with_parameters(
        interface_name: Option<&str>,
        interface_flags: InterfaceFlags,
    ) -> Result<Self> {
        let mut req: ifreq = unsafe { std::mem::zeroed() };
        req.ifr_ifru.ifru_flags = interface_flags.bits() as i16;

        if let Some(interface_name) = interface_name {
            if interface_name.len() > IFNAMSIZ - 1 {
                return Err(TunError::new(
                    TunErrorKind::InterfaceNameTooLong,
                    std::io::Error::other("interface name is too long"),
                ));
            } else {
                interface_name.copy_into(&mut req.ifr_name);
            }
        }

        let tun_fd = fcntl::open(
            "/dev/net/tun",
            fcntl::OFlag::O_RDWR,
            nix::sys::stat::Mode::empty(),
        )
        .map_err(|e| TunError::new(TunErrorKind::OpenTun, e))?;
        unsafe {
            tunsetiff(tun_fd.as_raw_fd(), &mut req as *mut _ as _)
                .map_err(|e| TunError::new(TunErrorKind::CreateInterface, e))?
        };

        Ok(Self {
            tun_fd,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Consume `Tun` and return the tunnel file descriptor.
    pub fn into_owned_fd(self) -> OwnedFd {
        self.tun_fd
    }
}

impl<'a> Tun<'a, BorrowedFd<'a>> {
    /// Initialize `Utun` using borrowed tunnel file descriptor.
    pub fn new_from_borrowed_fd(tun_fd: BorrowedFd<'a>) -> Result<Tun<'a, BorrowedFd<'a>>> {
        Ok(Self {
            tun_fd,
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<T: AsFd> Tun<'_, T> {
    /// Get tunnel interface name
    pub fn name(&self) -> Result<String> {
        let mut req: ifreq = unsafe { std::mem::zeroed() };
        unsafe { tungetiff(self.tun_fd.as_fd().as_raw_fd(), &mut req as *mut _ as _)? };

        #[cfg(not(target_arch = "aarch64"))]
        let bytes = req
            .ifr_name
            .into_iter()
            .map(|c| c as u8)
            .collect::<Vec<_>>();

        #[cfg(target_arch = "aarch64")]
        let bytes = req.ifr_name;

        CStr::from_bytes_until_nul(&bytes)
            .map_err(|err| TunError::new(TunErrorKind::InterfaceNameIntoString, err))?
            .to_str()
            .map_err(|err| TunError::new(TunErrorKind::InterfaceNameIntoString, err))
            .map(ToOwned::to_owned)
    }

    /// Persist interface to prevent it from being destroyed after closing the tun descriptor.
    pub fn set_persistent(&self, persistent: bool) -> Result<()> {
        let data = if persistent { 1 } else { 0 };
        unsafe { tunsetpersist(self.as_fd().as_raw_fd(), data)? };
        Ok(())
    }

    /// Assign interface to a given user
    pub fn set_owner(&self, uid: u64) -> Result<()> {
        // the ioctl parameter is c_ulong, which is u32 on 32-bit targets; uids never exceed u32
        unsafe { tunsetowner(self.as_fd().as_raw_fd(), uid as _)? };
        Ok(())
    }

    /// Assign interface to a given group
    pub fn set_group(&self, gid: u64) -> Result<()> {
        // the ioctl parameter is c_ulong, which is u32 on 32-bit targets; gids never exceed u32
        unsafe { tunsetgroup(self.as_fd().as_raw_fd(), gid as _)? };
        Ok(())
    }
}

impl<'a, T: AsFd> AsFd for Tun<'a, T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.tun_fd.as_fd()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TunErrorKind {
    Io,
    InterfaceNameIntoString,
    InterfaceNameTooLong,
    OpenTun,
    CreateInterface,
}

#[derive(Debug)]
pub struct TunError {
    kind: TunErrorKind,
    source: Box<dyn std::error::Error + 'static>,
}

impl TunError {
    fn new(kind: TunErrorKind, source: impl Into<Box<dyn std::error::Error>>) -> Self {
        Self {
            kind,
            source: source.into(),
        }
    }

    pub fn kind(&self) -> TunErrorKind {
        self.kind
    }

    pub fn io_error(&self) -> Option<std::io::Error> {
        self.source
            .downcast_ref::<nix::Error>()
            .map(|err| std::io::Error::from(*err))
    }
}

impl std::fmt::Display for TunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.kind {
            TunErrorKind::Io => "io error",
            TunErrorKind::InterfaceNameIntoString => "failed to convert interface name to string",
            TunErrorKind::InterfaceNameTooLong => "interface name is too long",
            TunErrorKind::OpenTun => "failed to open tun",
            TunErrorKind::CreateInterface => "failed to create interface",
        })
    }
}

impl std::error::Error for TunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl From<nix::Error> for TunError {
    fn from(value: nix::Error) -> Self {
        Self::new(TunErrorKind::Io, value)
    }
}

pub type Result<T, E = TunError> = std::result::Result<T, E>;
