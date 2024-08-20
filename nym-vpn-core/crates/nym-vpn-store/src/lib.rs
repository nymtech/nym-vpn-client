// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod keys;
pub mod mnemonic;

pub trait VpnStorage: mnemonic::MnemonicStorage + keys::KeyStore {}
