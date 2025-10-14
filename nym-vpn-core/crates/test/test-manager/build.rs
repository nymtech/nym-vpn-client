// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::env::var;

fn main() {
    // Rebuild if SSH provision script changes
    println!("cargo::rerun-if-changed=../scripts/ssh-setup.sh");

    let link_statically = var("TEST_MANAGER_STATIC").is_ok_and(|x| x != "0");

    if link_statically {
        println!("cargo::rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
        println!("cargo::rustc-link-lib=static=pcap");
    }
}
