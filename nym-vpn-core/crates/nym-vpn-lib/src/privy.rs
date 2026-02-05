// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub const PRIVY_DERIVATION_MESSAGE: &str = "DeriveAccount:NymVPN";

pub fn message_to_sign() -> String {
    hex::encode(PRIVY_DERIVATION_MESSAGE.as_bytes())
}
