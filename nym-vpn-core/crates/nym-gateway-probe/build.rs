// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rust2go::Builder::new()
        .with_go_src("./netstack_ping")
        .build();

    EmitBuilder::builder()
        .all_build()
        .all_git()
        .all_rustc()
        .all_cargo()
        .emit()
        .expect("failed to extract build metadata");
    Ok(())
}
