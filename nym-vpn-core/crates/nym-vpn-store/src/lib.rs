// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod account;
pub mod account_summary;
pub mod keys;

pub trait VpnStorage:
    account::AccountInformationStorage
    + keys::device::DeviceKeyStore
    + account_summary::AccountSummaryStorage
{
}
