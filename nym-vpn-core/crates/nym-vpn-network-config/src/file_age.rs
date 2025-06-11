// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, time::Duration};

#[derive(Debug, thiserror::Error)]
pub enum FileAgeError {
    #[error("Failed to obtain file metadata")]
    GetMetadata(#[source] std::io::Error),

    #[error("Failed to obtain modification date")]
    GetModificationTime(#[source] std::io::Error),

    #[error("Failed to calculate elapsed time")]
    SystemTime(#[source] std::time::SystemTimeError),
}

pub(crate) fn get_age_of_file(file_path: &PathBuf) -> Result<Option<Duration>, FileAgeError> {
    match std::fs::metadata(file_path) {
        Ok(metadata) => {
            let modification_date = metadata
                .modified()
                .map_err(FileAgeError::GetModificationTime)?;
            let elapsed = modification_date
                .elapsed()
                .map_err(FileAgeError::SystemTime)?;
            Ok(Some(elapsed))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(e) => Err(FileAgeError::GetMetadata(e)),
    }
}
