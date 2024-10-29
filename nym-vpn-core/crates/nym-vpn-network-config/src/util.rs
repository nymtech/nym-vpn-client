// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, time::Duration};

pub(crate) fn get_age_of_file(file_path: &PathBuf) -> anyhow::Result<Option<Duration>> {
    if !file_path.exists() {
        return Ok(None);
    }
    let metadata = std::fs::metadata(file_path)?;
    Ok(Some(metadata.modified()?.elapsed()?))
}
