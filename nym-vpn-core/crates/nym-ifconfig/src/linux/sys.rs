// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// This call takes a pointer to ifreq but must be defined as taking integer.
nix::ioctl_write_int!(tunsetiff, b'T', 202);
// This call returns a pointer to ifreq but must be defined returning integer.
nix::ioctl_read!(tungetiff, b'T', 210, nix::libc::c_int);
nix::ioctl_write_int!(tunsetpersist, b'T', 203);
nix::ioctl_write_int!(tunsetowner, b'T', 204);
nix::ioctl_write_int!(tunsetgroup, b'T', 206);
