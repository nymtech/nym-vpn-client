use std::{
    io,
    os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd},
};

use nix::fcntl::{self, FcntlArg, OFlag};
use tun::Device;

pub trait DupFd {
    /// Duplicate tunnel file descriptor pointing to the same file description as the original one.
    /// Ensures that O_NONBLOCK is set.
    fn dup_fd(&self) -> io::Result<OwnedFd>;
}

impl<T: Device + AsRawFd> DupFd for T {
    fn dup_fd(&self) -> io::Result<OwnedFd> {
        dup_fd(self.as_raw_fd())
    }
}

fn dup_fd(raw_fd: RawFd) -> io::Result<OwnedFd> {
    let dup_fd = unsafe { nix::libc::dup(raw_fd) };
    if dup_fd == -1 {
        return Err(io::Error::last_os_error());
    }

    let owned_fd = unsafe { OwnedFd::from_raw_fd(dup_fd) };

    let flags = OFlag::from_bits_retain(fcntl::fcntl(owned_fd.as_raw_fd(), FcntlArg::F_GETFL)?);
    if !flags.contains(OFlag::O_NONBLOCK) {
        fcntl::fcntl(
            owned_fd.as_raw_fd(),
            FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK),
        )?;
    }

    Ok(owned_fd)
}
