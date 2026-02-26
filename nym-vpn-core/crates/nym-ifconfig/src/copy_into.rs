// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::c_char;

use nix::libc::IFNAMSIZ;

pub trait CopyInto<S> {
    fn copy_into(&self, out: &mut S);
}

impl CopyInto<[c_char; IFNAMSIZ]> for &str {
    fn copy_into(&self, buf: &mut [c_char; IFNAMSIZ]) {
        // Take IFNAMESIZ-1 bytes at most leaving space for nul terminator
        // Stops taking bytes at the first nul byte
        let ifname = self
            .as_bytes()
            .iter()
            .copied()
            .take_while(|x| *x != 0)
            .take(IFNAMSIZ - 1)
            .collect::<Vec<u8>>();

        let mut bytes = [0u8; IFNAMSIZ];
        bytes[0..ifname.len()].copy_from_slice(&ifname);

        // SAFETY: both arrays are of the same size
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr() as _, buf.as_mut_ptr(), bytes.len())
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix::libc::{IFNAMSIZ, c_char};

    #[test]
    fn test_copy_ifname_without_nul() {
        let mut buf: [c_char; IFNAMSIZ] = [0; _];
        "tun".copy_into(&mut buf);

        assert_eq!(
            [
                't' as c_char,
                'u' as c_char,
                'n' as c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0
            ] as [c_char; IFNAMSIZ],
            buf
        );
    }
    #[test]
    fn test_copy_ifname_with_interior_nul() {
        let mut buf: [c_char; IFNAMSIZ] = [0; _];
        "tun\0name".copy_into(&mut buf);

        assert_eq!(
            [
                't' as c_char,
                'u' as c_char,
                'n' as c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0
            ] as [c_char; IFNAMSIZ],
            buf
        );
    }

    #[test]
    fn test_copy_empty_string() {
        let mut buf: [c_char; IFNAMSIZ] = [0; _];
        "".copy_into(&mut buf);

        assert_eq!([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], buf);
    }
}
