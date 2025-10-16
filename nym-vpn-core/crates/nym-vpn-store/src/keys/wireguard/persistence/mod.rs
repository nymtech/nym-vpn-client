// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::keys::wireguard::WireguardKeys;

pub(crate) mod ephemeral;
pub(crate) mod on_disk;

fn random_keys() -> WireguardKeys {
    let mut rng = rand::rngs::OsRng;
    WireguardKeys::generate_new(&mut rng)
}
