// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod ephemeral;
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod keychain;
mod on_disk;

pub use on_disk::{
    DEFAULT_PRIVATE_DEVICE_KEY_FILENAME, DEFAULT_PUBLIC_DEVICE_KEY_FILENAME, DeviceKeysPaths,
    OnDiskKeys, OnDiskKeysError,
};

pub use ephemeral::InMemEphemeralKeys;
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use keychain::{KeychainKeys, KeychainKeysError};
