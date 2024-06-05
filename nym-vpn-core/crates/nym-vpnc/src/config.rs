// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

pub(super) fn get_socket_path() -> PathBuf {
    #[cfg(unix)]
    return Path::new("/var/run/nym-vpn.sock").to_path_buf();

    #[cfg(windows)]
    return Path::new(r"\\.\pipe\nym-vpn").to_path_buf();
}

pub(crate) fn default_endpoint() -> String {
    "http://[::1]:53181".to_string()
}
