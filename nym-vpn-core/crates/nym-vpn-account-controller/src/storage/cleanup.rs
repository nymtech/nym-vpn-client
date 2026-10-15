// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    io,
    path::{Path, PathBuf},
};

use futures::StreamExt;
use tokio_stream::wrappers::ReadDirStream;

use nym_common::trace_err_chain;
use nym_sdk::mixnet::StoragePaths;
use nym_vpn_store::keys::{
    device::{DEFAULT_PRIVATE_DEVICE_KEY_FILENAME, DEFAULT_PUBLIC_DEVICE_KEY_FILENAME},
    wireguard::DB_NAME,
};

use crate::Error;

// TODO: implement functionality where the owning code of these files delete them instead. To
// protect us against the names drifting out of sync.

pub async fn remove_files_for_account(
    data_dir: &Path,
    remove_wireguard_keys_db: bool,
) -> Result<(), Error> {
    let storage_paths =
        StoragePaths::new_from_dir(data_dir).map_err(|err| Error::StoragePaths(Box::new(err)))?;

    let files_to_remove = [
        // Files specific to the VPN client
        data_dir.join(DEFAULT_PRIVATE_DEVICE_KEY_FILENAME),
        data_dir.join(DEFAULT_PUBLIC_DEVICE_KEY_FILENAME),
        // Files specific to the mixnet client
        storage_paths.private_identity,
        storage_paths.public_identity,
        storage_paths.private_encryption,
        storage_paths.public_encryption,
        storage_paths.ack_key,
    ];

    for file_path in files_to_remove {
        remove_file(&file_path).await;
    }

    let mut databases_to_remove = vec![
        storage_paths.reply_surb_database_path,
        storage_paths.gateway_registrations,
    ];

    if remove_wireguard_keys_db {
        databases_to_remove.push(data_dir.join(DB_NAME));
    }

    for db_path in databases_to_remove {
        remove_sqlite_database(&db_path).await;
    }

    // For the persistent reply store we also have backups of corrupted files. They have the same
    // filename as the original file, but appended with `_1234567890.corrupted`. Make sure to
    // delete all of these as well.

    let corrupted_files = get_list_of_corrupted_files(data_dir)
        .await
        .inspect_err(|err| {
            tracing::error!("Failed to get list of corrupted files: {err}");
        });

    if let Ok(corrupted_files) = corrupted_files {
        for file in corrupted_files {
            tracing::info!("Removing corrupted file: {}", file.display());
            match tokio::fs::remove_file(&file).await {
                Ok(_) => tracing::info!("Removed corrupted file: {}", file.display()),
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    tracing::debug!("Corrupted file not found, skipping: {}", file.display());
                }
                Err(err) => {
                    trace_err_chain!(err, "Failed to remove corrupted file {}", file.display());
                }
            }
        }
    }

    // Warn if there are any files left in the data directory
    let mut dir_stream = ReadDirStream::new(
        tokio::fs::read_dir(data_dir)
            .await
            .map_err(Error::internal)?,
    );

    while let Some(entry_result) = dir_stream.next().await {
        if let Ok(entry) = entry_result {
            tracing::info!("File left in data directory: {}", entry.path().display());
        }
    }

    Ok(())
}

/// Removes the SQLite database, then its `-wal` and `-shm` files once the database is gone.
async fn remove_sqlite_database(db_path: &Path) {
    if !remove_file(db_path).await {
        return;
    }

    for suffix in ["-wal", "-shm"] {
        let mut path = db_path.as_os_str().to_owned();
        path.push(suffix);
        remove_file(Path::new(&path)).await;
    }
}

/// Returns `false` if the file exists but could not be removed.
async fn remove_file(file_path: &Path) -> bool {
    tracing::info!("Removing file: {}", file_path.display());
    match tokio::fs::remove_file(file_path).await {
        Ok(_) => {
            tracing::info!("Removed file: {}", file_path.display());
            true
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            tracing::debug!("File not found, skipping: {}", file_path.display());
            true
        }
        Err(err) => {
            trace_err_chain!(err, "Failed to remove file {}", file_path.display());
            false
        }
    }
}

async fn get_list_of_corrupted_files(data_dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let base_name = StoragePaths::new_from_dir(data_dir)
        .map_err(|err| Error::StoragePaths(Box::new(err)))?
        .reply_surb_database_path;

    let Some(starts_with) = base_name.file_name().and_then(|bn| bn.to_str()) else {
        return Ok(vec![]);
    };

    let mut dir_stream = ReadDirStream::new(
        tokio::fs::read_dir(data_dir)
            .await
            .map_err(Error::internal)?,
    );

    // Delete files of the form `base_name._[0-9]*.corrupted`
    let mut corrupted_files = vec![];
    while let Some(entry_result) = dir_stream.next().await {
        if let Ok(entry) = entry_result
            && entry
                .file_name()
                .to_str()
                .is_some_and(|s| s.starts_with(starts_with) || s.ends_with(".corrupted"))
        {
            corrupted_files.push(entry.path());
        }
    }
    Ok(corrupted_files)
}
