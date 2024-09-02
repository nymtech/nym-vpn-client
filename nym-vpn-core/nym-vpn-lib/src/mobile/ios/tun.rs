// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::{
    ffi::{c_char, c_ulong, CStr},
    os::fd::{BorrowedFd, RawFd},
};

use nix::{
    libc::{self, sockaddr, sockaddr_ctl, socklen_t, AF_SYSTEM},
    sys::socket,
};

const UTUN_CTL_NAME: &CStr = c"com.apple.net.utun_control";
const CTLIOCGINFO: c_ulong = 0xc0644e03;

#[repr(C)]
#[allow(non_camel_case_types)]
struct ctl_info {
    ctl_id: u32,
    ctl_name: [c_char; 96],
}

/// Returns tunnel file descriptor on iOS by incrementally probing first 1024 file descriptors.
pub fn get_tun_fd() -> Option<RawFd> {
    let mut ctl_info: ctl_info = unsafe { std::mem::zeroed() };
    unsafe {
        std::ptr::copy_nonoverlapping(
            UTUN_CTL_NAME.as_ptr(),
            ctl_info.ctl_name.as_mut_ptr(),
            UTUN_CTL_NAME.count_bytes(),
        )
    };

    (0..1024).find(|fd| {
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
}

/// Returns tunnel interface name
pub fn get_tun_ifname(tun_fd: RawFd) -> Option<String> {
    let borrowed_fd = unsafe { BorrowedFd::borrow_raw(tun_fd) };

    socket::getsockopt(&borrowed_fd, socket::sockopt::UtunIfname)
        .ok()?
        .into_string()
        .ok()
}
