// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Needed for reflection
    let vpn_fd = PathBuf::from(env::var("OUT_DIR").unwrap()).join("vpn_descriptor.bin");

    let proto_dir = PathBuf::from("../../../proto");
    let vpn_proto = proto_dir.join("nym/vpn.proto");
    let vpn_proto_out = proto_dir.join("nym");

    tonic_build::configure()
        .file_descriptor_set_path(vpn_fd)
        .compile_protos(&[vpn_proto], &[vpn_proto_out])?;

    Ok(())
}
