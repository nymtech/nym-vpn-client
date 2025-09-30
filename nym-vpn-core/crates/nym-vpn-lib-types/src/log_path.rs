// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
pub struct LogPath {
    pub dir: PathBuf,
    pub filename: String,
}

impl LogPath {
    pub fn new(log_dir: PathBuf, log_file: String) -> Self {
        Self {
            dir: log_dir,
            filename: log_file,
        }
    }
}
