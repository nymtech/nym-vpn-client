// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{ffi::OsStr, fs, path::Path};

use crate::Error;

// TODO: implement functionality where the owning code of these files delete them instead. To
// protect us against the names drifting out of sync.

pub fn remove_files_for_account(data_dir: &Path) -> Result<(), Error> {
    // Delete all files in data_dir that matches any of these patterns:
    // *.sqlite
    // *.pem

    for file in fs::read_dir(data_dir).map_err(Error::internal)? {
        let file = match file {
            Ok(file) => file,
            Err(err) => {
                tracing::error!("failed to read file in data directory: {:?}", err);
                continue;
            }
        };

        if Some(OsStr::new("sqlite")) == file.path().extension()
            || Some(OsStr::new("pem")) == file.path().extension()
        {
            tracing::info!("removing file: {:?}", file.path());
            fs::remove_file(file.path())
                .inspect_err(|err| tracing::error!("failed to remove file: {:?}", err))
                .ok();
        }
    }

    // Warn if there are any files left in the data directory
    let remaining_files = fs::read_dir(data_dir)
        .map_err(Error::internal)?
        .filter_map(|file| file.ok())
        .map(|file| file.path());
    for file in remaining_files {
        tracing::warn!("file left in data directory: {:?}", file);
    }

    Ok(())
}
