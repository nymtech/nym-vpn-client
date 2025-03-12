use std::os::fd::BorrowedFd;
#[cfg(target_os = "android")]
use std::{ffi::CStr, os::fd::AsRawFd};

#[cfg(target_os = "android")]
use nix::libc::ifreq;
#[cfg(target_os = "ios")]
use nix::sys::socket::{getsockopt, sockopt};

#[cfg(target_os = "android")]
// This call takes a pointer to ifreq but must be defined as taking integer.
nix::ioctl_read!(tungetiff, b'T', 210, nix::libc::c_int);

#[derive(thiserror::Error, Debug)]
pub enum GetTunNameError {
    #[error("Syscall error")]
    Syscall(#[source] nix::Error),

    #[error("Failed to copy interface name")]
    CopyInterfaceName(#[source] std::ffi::FromBytesUntilNulError),

    #[error("Failed to convert interface name to utf-8")]
    ConvertInterfaceNameToUtf8(#[source] std::str::Utf8Error),
}

pub type Result<T, E = GetTunNameError> = std::result::Result<T, E>;

/// Returns tunnel interface name for the given tunnel file descriptor.
#[cfg(target_os = "ios")]
pub fn get_tun_name(fd: &BorrowedFd) -> Result<String> {
    getsockopt(fd, sockopt::UtunIfname)
        .map_err(GetTunNameError::Syscall)?
        .to_str()
        .map(|s| s.to_owned())
        .map_err(GetTunNameError::ConvertInterfaceNameToUtf8)
}

/// Returns tunnel interface name for the given tunnel file descriptor.
#[cfg(target_os = "android")]
pub fn get_tun_name(fd: &BorrowedFd) -> Result<String> {
    let mut ifr: ifreq = unsafe { std::mem::zeroed() };
    unsafe { tungetiff(fd.as_raw_fd(), &mut ifr as *mut _ as _) }
        .map_err(GetTunNameError::Syscall)?;

    CStr::from_bytes_until_nul(&ifr.ifr_name)
        .map_err(GetTunNameError::CopyInterfaceName)?
        .to_str()
        .map(|s| s.to_owned())
        .map_err(GetTunNameError::ConvertInterfaceNameToUtf8)
}
