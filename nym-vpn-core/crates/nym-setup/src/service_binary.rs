// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Returns the absolute path to nym-vpnd upon success.
pub fn service_binary_path() -> Result<PathBuf> {
    let mut binary_path =
        std::env::current_exe().with_context(|| "Cannot obtain current executable path")?;
    binary_path.pop();
    binary_path.push("nym-vpnd");
    binary_path.set_extension(std::env::consts::EXE_EXTENSION);
    Ok(binary_path)
}
