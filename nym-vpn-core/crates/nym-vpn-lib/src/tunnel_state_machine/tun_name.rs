use std::os::fd::BorrowedFd;

/// Returns tunnel interface name for the given tunnel file descriptor.
#[cfg(target_os = "ios")]
pub fn get_tun_name(fd: &BorrowedFd) -> Option<String> {
    use nix::sys::socket::{getsockopt, sockopt};

    match getsockopt(fd, sockopt::UtunIfname) {
        Ok(name_cstr) => Some(name_cstr.to_str().ok()?.to_owned()),
        Err(e) => {
            tracing::error!("Failed to obtain tunnel interface name: {}", e);
            None
        }
    }
}

/// Returns tunnel interface name for the given tunnel file descriptor.
#[cfg(target_os = "android")]
pub fn get_tun_name(fd: &BorrowedFd) -> Option<String> {
    use nix::{
        errno::Errno,
        libc::{ifreq, ioctl, IFNAMSIZ, TUNGETIFF},
        sys::socket::SockaddrStorage,
    };

    unsafe {
        let mut ifr: ifreq = std::mem::zeroed();
        if ioctl(fd.as_raw_fd(), TUNGETIFF as _, &mut ifr) < 0 {
            tracing::error!("Failed to obtain tunnel interface name: {}", Errno::last());
            return None;
        }

        // Extract and return the interface name
        let ifname = CStr::from_ptr(ifr.ifr_name.as_ptr());
        ifname.to_str().ok().map(|s| s.to_string())
    }
}
