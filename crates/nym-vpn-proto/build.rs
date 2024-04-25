// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // needed for reflection
    let vpn_fd = PathBuf::from(env::var("OUT_DIR").unwrap()).join("vpn_descriptor.bin");
    tonic_build::configure()
        .file_descriptor_set_path(vpn_fd)
        .compile(&["../../proto/nym/vpn.proto"], &["../../proto/nym/"])?;

    tonic_build::configure()
        // server implementation is handled by tonic-health crate
        .build_server(false)
        .compile(&["../../proto/grpc/health.proto"], &["../../proto/grpc/"])?;
    Ok(())
}
