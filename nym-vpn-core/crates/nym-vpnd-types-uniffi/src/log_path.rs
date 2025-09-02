// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

uniffi::use_remote_type!(nym_vpn_lib_types_uniffi::PathBuf);

#[derive(uniffi::Record)]
pub struct LogPath {
    pub dir: PathBuf,
    pub filename: String,
}

impl From<nym_vpnd_types::log_path::LogPath> for LogPath {
    fn from(log_path: nym_vpnd_types::log_path::LogPath) -> Self {
        Self {
            dir: log_path.dir,
            filename: log_path.filename,
        }
    }
}
