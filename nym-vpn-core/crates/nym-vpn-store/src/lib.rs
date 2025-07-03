// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod keys;
pub mod mnemonic;
mod types;

pub trait VpnStorage: mnemonic::MnemonicStorage + keys::device::DeviceKeyStore {}
