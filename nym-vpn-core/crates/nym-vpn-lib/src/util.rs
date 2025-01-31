// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

pub(crate) fn construct_user_agent() -> nym_sdk::UserAgent {
    let bin_info = nym_bin_common::bin_info_local_vergen!();
    let name = sysinfo::System::name().unwrap_or("unknown".to_string());
    let os_long = sysinfo::System::long_os_version().unwrap_or("unknown".to_string());
    let arch = sysinfo::System::cpu_arch();
    let platform = format!("{}; {}; {}", name, os_long, arch);
    nym_sdk::UserAgent {
        application: bin_info.binary_name.to_string(),
        version: bin_info.build_version.to_string(),
        platform,
        git_commit: bin_info.commit_sha.to_string(),
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum DataDirError {
    #[error("data directory is not valid path")]
    DataDirNotValid,

    #[error("data directory does not exist")]
    DataDirDoesNotExist,

    #[error("unexpected content in data directory")]
    UnexpectedDataDirContent,
}

// Check that the directory contains at least some of the expected files.
// A usefule safety check for when deleting the directory.
pub fn assert_existence_of_expected_files<P: TryInto<PathBuf>>(
    data_dir: P,
) -> Result<(), DataDirError> {
    let data_dir = data_dir
        .try_into()
        .map_err(|_| DataDirError::DataDirNotValid)?;
    if !data_dir.exists() {
        return Err(DataDirError::DataDirDoesNotExist);
    }

    let expected_files = [
        "credentials_database.db",
        "ack_key.pem",
        "public_identity.pem",
        "public_entry_wireguard.pem",
        "public_exit_wireguard.pem",
        "mnemonic.json",
    ];

    let found_files = expected_files
        .iter()
        .map(|file| data_dir.join(file).exists())
        .fold(0, |acc, exists| acc + exists as usize);

    tracing::debug!(
        "Found {} out of {} expected files in data dir",
        found_files,
        expected_files.len()
    );

    // Not all files might be present, e.g if we just removed the account. Or if not all files have
    // created yet when starting out.
    if found_files > 1 {
        Ok(())
    } else {
        Err(DataDirError::UnexpectedDataDirContent)
    }
}
