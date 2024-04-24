// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // needed for reflection
    let descriptor_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("vpn_descriptor.bin");

    tonic_build::configure()
        .file_descriptor_set_path(descriptor_path)
        .compile(&["../../proto/nym/vpn.proto"], &["../../proto/nym/"])?;
    Ok(())
}
