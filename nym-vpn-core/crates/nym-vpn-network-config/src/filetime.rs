// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::Path, time::Duration};

#[derive(Debug, thiserror::Error)]
pub enum FileTimeError {
    #[error("failed to obtain file metadata")]
    GetMetadata(#[source] std::io::Error),

    #[error("failed to obtain modification date")]
    GetModificationTime(#[source] std::io::Error),

    #[error("failed to calculate elapsed time")]
    ElapsedTime(#[source] std::time::SystemTimeError),
}

pub type Result<T, E = FileTimeError> = std::result::Result<T, E>;

/// Returns true if time elapsed since last file modification is equal or exceeds the value specified in `max_age` or if file does not exist.
pub fn is_stale_file(file_path: impl AsRef<Path>, max_age: Duration) -> Result<bool> {
    Ok(match time_since_modification(file_path)? {
        Some(elapsed) => elapsed >= max_age,
        None => true, // If the file does not exist, consider it stale
    })
}

/// Returns time elapsed since last file modification or `None` if file does not exist.
fn time_since_modification(file_path: impl AsRef<Path>) -> Result<Option<Duration>> {
    match std::fs::metadata(file_path) {
        Ok(metadata) => {
            let modification_date = metadata
                .modified()
                .map_err(FileTimeError::GetModificationTime)?;
            let elapsed = modification_date
                .elapsed()
                .map_err(FileTimeError::ElapsedTime)?;
            Ok(Some(elapsed))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(FileTimeError::GetMetadata(e)),
    }
}
