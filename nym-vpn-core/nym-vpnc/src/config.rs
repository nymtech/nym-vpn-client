// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

pub(crate) fn get_socket_path() -> PathBuf {
    Path::new("/var/run/nym-vpn.sock").to_path_buf()
}

pub(crate) fn default_endpoint() -> String {
    "http://[::1]:53181".to_string()
}
