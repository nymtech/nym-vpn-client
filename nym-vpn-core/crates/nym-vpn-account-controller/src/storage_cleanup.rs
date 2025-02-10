// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fs, path::Path};

use nym_client_core::config::disk_persistence::{
    DEFAULT_ACK_KEY_FILENAME, DEFAULT_GATEWAYS_DETAILS_DB_FILENAME,
    DEFAULT_PRIVATE_ENCRYPTION_KEY_FILENAME, DEFAULT_PRIVATE_IDENTITY_KEY_FILENAME,
    DEFAULT_PUBLIC_ENCRYPTION_KEY_FILENAME, DEFAULT_PUBLIC_IDENTITY_KEY_FILENAME,
    DEFAULT_REPLY_SURB_DB_FILENAME,
};
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
    let device_key = [
        DEFAULT_PRIVATE_DEVICE_KEY_FILENAME,
        DEFAULT_PUBLIC_DEVICE_KEY_FILENAME,
    ];

    let mixnet_keys = [
        DEFAULT_PRIVATE_IDENTITY_KEY_FILENAME,
        DEFAULT_PUBLIC_IDENTITY_KEY_FILENAME,
        DEFAULT_PRIVATE_ENCRYPTION_KEY_FILENAME,
        DEFAULT_PUBLIC_ENCRYPTION_KEY_FILENAME,
        DEFAULT_ACK_KEY_FILENAME,
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

    let mixnet_db = [
        DEFAULT_REPLY_SURB_DB_FILENAME,
        DEFAULT_GATEWAYS_DETAILS_DB_FILENAME,
    ];

    let files_to_remove = device_key
        .iter()
        .chain(mixnet_keys.iter())
        .chain(wireguard_keys.iter())
        .chain(mixnet_db.iter());

    for file in files_to_remove {
        let file_path = data_dir.join(file);
        if file_path.exists() {
            tracing::info!("removing file: {:?}", file);
            fs::remove_file(file_path)
                .inspect_err(|err| {
                    tracing::error!("failed to remove file: {err:?}");
                })
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
