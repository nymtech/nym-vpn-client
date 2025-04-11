// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

pub fn get_socket_path() -> PathBuf {
    #[cfg(unix)]
    return Path::new("/var/run/nym-vpn.sock").to_path_buf();

    #[cfg(windows)]
    return Path::new(r"\\.\pipe\nym-vpn").to_path_buf();
}
