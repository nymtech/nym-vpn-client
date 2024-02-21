// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

fn main() {
    uniffi::generate_scaffolding("src/nym_vpn_lib_android.udl").unwrap();
    uniffi::generate_scaffolding("src/nym_vpn_lib_macos.udl").unwrap();
}
