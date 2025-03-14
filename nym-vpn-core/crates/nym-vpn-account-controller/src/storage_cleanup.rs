// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use nym_sdk::mixnet::StoragePaths;
use nym_vpn_store::keys::persistence::{
    DEFAULT_PRIVATE_DEVICE_KEY_FILENAME, DEFAULT_PUBLIC_DEVICE_KEY_FILENAME,
};
use nym_wg_gateway_client::{
    DEFAULT_FREE_PRIVATE_ENTRY_WIREGUARD_KEY_FILENAME,
    DEFAULT_FREE_PRIVATE_EXIT_WIREGUARD_KEY_FILENAME,
    DEFAULT_FREE_PUBLIC_ENTRY_WIREGUARD_KEY_FILENAME,
    DEFAULT_FREE_PUBLIC_EXIT_WIREGUARD_KEY_FILENAME, DEFAULT_PRIVATE_ENTRY_WIREGUARD_KEY_FILENAME,
    DEFAULT_PRIVATE_EXIT_WIREGUARD_KEY_FILENAME, DEFAULT_PUBLIC_ENTRY_WIREGUARD_KEY_FILENAME,
    DEFAULT_PUBLIC_EXIT_WIREGUARD_KEY_FILENAME,
};

use crate::Error;

// TODO: implement functionality where the owning code of these files delete them instead. To
// protect us against the names drifting out of sync.

pub fn remove_files_for_account(data_dir: &Path) -> Result<(), Error> {
    // Files specific to the VPN client
    let device_key = [
        DEFAULT_PRIVATE_DEVICE_KEY_FILENAME,
        DEFAULT_PUBLIC_DEVICE_KEY_FILENAME,
    ];

    let wireguard_keys = [
        DEFAULT_PRIVATE_ENTRY_WIREGUARD_KEY_FILENAME,
        DEFAULT_PUBLIC_ENTRY_WIREGUARD_KEY_FILENAME,
        DEFAULT_PRIVATE_EXIT_WIREGUARD_KEY_FILENAME,
        DEFAULT_PUBLIC_EXIT_WIREGUARD_KEY_FILENAME,
        DEFAULT_FREE_PRIVATE_ENTRY_WIREGUARD_KEY_FILENAME,
        DEFAULT_FREE_PUBLIC_ENTRY_WIREGUARD_KEY_FILENAME,
        DEFAULT_FREE_PRIVATE_EXIT_WIREGUARD_KEY_FILENAME,
        DEFAULT_FREE_PUBLIC_EXIT_WIREGUARD_KEY_FILENAME,
    ];

    let vpn_paths = device_key
        .iter()
        .chain(wireguard_keys.iter())
        .map(|file| data_dir.join(file));

    // Files specific to the mixnet client
    let storage_paths = StoragePaths::new_from_dir(data_dir).map_err(Error::StoragePaths)?;
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
        match fs::remove_file(&file_path) {
            Ok(_) => tracing::info!("Removed file: {}", file_path.display()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                tracing::debug!("File not found, skipping: {}", file_path.display());
            }
            Err(err) => tracing::error!("Failed to remove file {}: {err}", file_path.display()),
        }
    }

    // For the persistent reply store we also have backups of corrupted files. They have the same
    // filename as the original file, but appended with `_1234567890.corrupted`. Make sure to
    // delete all of these as well.

    let corrupted_files = get_list_of_corrupted_files(data_dir).inspect_err(|err| {
        tracing::error!("Failed to get list of corrupted files: {err}");
    });

    if let Ok(corrupted_files) = corrupted_files {
        for file in corrupted_files {
            tracing::info!("Removing corrupted file: {}", file.display());
            match fs::remove_file(&file) {
                Ok(_) => tracing::info!("Removed corrupted file: {}", file.display()),
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    tracing::debug!("Corrupted file not found, skipping: {}", file.display());
                }
                Err(err) => {
                    tracing::error!("Failed to remove corrupted file {}: {err}", file.display())
                }
            }
        }
    }

    // Warn if there are any files left in the data directory
    let remaining_files = fs::read_dir(data_dir)
        .map_err(Error::internal)?
        .filter_map(|file| file.ok())
        .map(|file| file.path());
    for file in remaining_files {
        tracing::info!("File left in data directory: {}", file.display());
    }

    Ok(())
}

fn get_list_of_corrupted_files(data_dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let base_name = StoragePaths::new_from_dir(data_dir)
        .map_err(Error::StoragePaths)?
        .reply_surb_database_path;

    if let Some(starts_with) = base_name.file_name().and_then(|bn| bn.to_str()) {
        // Delete files of the form `base_name._[0-9]*.corrupted`
        let corrupted_files = fs::read_dir(data_dir)
            .map_err(Error::internal)?
            .filter_map(|file| file.ok())
            .filter(|file| {
                file.file_name()
                    .to_str()
                    .map(|name| name.starts_with(starts_with))
                    .unwrap_or(false)
            })
            .filter(|file| {
                file.file_name()
                    .to_str()
                    .map(|name| name.ends_with(".corrupted"))
                    .unwrap_or(false)
            })
            .map(|file| file.path())
            .collect();
        Ok(corrupted_files)
    } else {
        Ok(vec![])
    }
}
