// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

pub mod on_disk;

pub trait StatsStorageError: Error + Send + Sync + 'static {}

pub trait StatsStorage {
    type StorageError: StatsStorageError;
}
