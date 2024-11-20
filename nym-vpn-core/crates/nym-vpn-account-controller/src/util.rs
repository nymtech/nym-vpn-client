// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fs, path::PathBuf};

use crate::Error;

// TODO: implement functionality where the owning code of these files delete them instead. To
// protect us against the names drifting out of sync.

pub fn remove_files_for_account(data_dir: PathBuf) -> Result<(), Error> {
    // Delete all files in data_dir that matches any of these patterns:
    // credentials_database.db
    // *.sqlite
    // *.pem

    for file in fs::read_dir(data_dir).map_err(Error::internal)? {
        let file = match file {
            Ok(file) => file,
            Err(err) => {
                tracing::error!("failed to read file in data directory: {:?}", err);
                break;
            }
        };

        let file_name = match file.file_name().to_str() {
            Some(file_name) => file_name.to_owned(),
            None => {
                tracing::error!("failed to convert file name to string: {:?}", file);
                break;
            }
        };
        if file_name == "credentials_database.db"
            || file_name.ends_with(".sqlite")
            || file_name.ends_with(".pem")
        {
            tracing::info!("removing file: {:?}", file.path());
            fs::remove_file(file.path())
                .inspect_err(|err| tracing::error!("failed to remove file: {:?}", err))
                .ok();
        }
    }
    Ok(())
}
