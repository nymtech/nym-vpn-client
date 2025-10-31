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
    // Files specific to the VPN client
    let mut vpn_paths = vec![
        DEFAULT_PRIVATE_DEVICE_KEY_FILENAME,
        DEFAULT_PUBLIC_DEVICE_KEY_FILENAME,
    ];

    if remove_wireguard_keys_db {
        vpn_paths.push(DB_NAME);
    }

    let vpn_paths = vpn_paths.iter().map(|file| data_dir.join(file));

    // Files specific to the mixnet client
    let storage_paths =
        StoragePaths::new_from_dir(data_dir).map_err(|err| Error::StoragePaths(Box::new(err)))?;
    let mixnet_paths = storage_paths
        .reply_surb_database_paths()
        .into_iter()
        .chain(storage_paths.gateway_registrations_paths())
        .chain([
            storage_paths.private_identity,
            storage_paths.public_identity,
            storage_paths.private_encryption,
            storage_paths.public_encryption,
            storage_paths.ack_key,
        ]);

    let files_to_remove = vpn_paths.chain(mixnet_paths);

    for file_path in files_to_remove {
        tracing::info!("Removing file: {}", file_path.display());
        match tokio::fs::remove_file(&file_path).await {
            Ok(_) => tracing::info!("Removed file: {}", file_path.display()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                tracing::debug!("File not found, skipping: {}", file_path.display());
            }
            Err(err) => {
                trace_err_chain!(err, "Failed to remove file {}", file_path.display());
            }
        }
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
