// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    ffi::{c_char, c_ulong, CStr},
    io,
    os::fd::{BorrowedFd, OwnedFd},
};

use nix::libc::{self, sockaddr, sockaddr_ctl, socklen_t, AF_SYSTEM};

const UTUN_CTL_NAME: &CStr = c"com.apple.net.utun_control";
const CTLIOCGINFO: c_ulong = 0xc0644e03;

#[repr(C)]
#[allow(non_camel_case_types)]
struct ctl_info {
    ctl_id: u32,
    ctl_name: [c_char; 96],
}

/// Returns a copy of tunnel file descriptor owned by packet tunnel provider.
pub fn get_tun_fd() -> io::Result<OwnedFd> {
    let mut ctl_info: ctl_info = unsafe { std::mem::zeroed() };
    unsafe {
        std::ptr::copy_nonoverlapping(
            UTUN_CTL_NAME.as_ptr(),
            ctl_info.ctl_name.as_mut_ptr(),
            UTUN_CTL_NAME.count_bytes(),
        )
    };

    // Probe first 1024 descriptors to find the tun descriptor owned by packet tunnel provider.
    let tun_fd = (0..1024)
        .find(|fd| {
            let mut ctl_addr: sockaddr_ctl = unsafe { std::mem::zeroed() };
            let mut len = std::mem::size_of_val(&ctl_addr) as socklen_t;
            let mut ret = unsafe {
                libc::getpeername(
                    *fd,
                    &mut ctl_addr as *mut sockaddr_ctl as *mut sockaddr,
                    &mut len,
                )
            };

            if ret == 0 && ctl_addr.sc_family as i32 == AF_SYSTEM {
                ret = unsafe { libc::ioctl(*fd, CTLIOCGINFO, &mut ctl_info) };
                ret == 0 && ctl_addr.sc_id == ctl_info.ctl_id
            } else {
                false
            }
        })
        .ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Cannot locate the tunnel device descriptor",
        ))?;

    // Borrow fd because the packet tunnel owns it, so we should never close the original file descriptor.
    let borrowed_fd = unsafe { BorrowedFd::borrow_raw(tun_fd) };

    // Internally makes a fcntl() call equivalent to dup() making a copy of file descriptor.
    borrowed_fd.try_clone_to_owned()
}
