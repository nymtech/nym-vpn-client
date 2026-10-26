// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod device_state;
pub mod encrypted_file;
pub mod encrypted_sqlite;
pub mod keys;
pub mod mnemonic;
pub mod secure_store_migration;
pub mod store_ciphers;

pub trait VpnStorage:
    mnemonic::MnemonicStorage + keys::device::DeviceKeyStore + device_state::DeviceStateStorage
{
}
