// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from("../../../proto");
    let vpn_proto = proto_dir.join("nym/vpn.proto");
    let vpn_proto_out = proto_dir.join("nym");

    tonic_build::configure().compile_protos(&[vpn_proto], &[vpn_proto_out])?;

    Ok(())
}
