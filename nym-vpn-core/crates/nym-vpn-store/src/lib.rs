// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod account;
pub mod account_summary;
pub mod ciphers;
pub mod encrypted_file;
pub mod encrypted_sqlite;
pub mod keys;

pub trait VpnStorage:
    account::AccountInformationStorage
    + keys::device::DeviceKeyStore
    + account_summary::AccountSummaryStorage
{
}
