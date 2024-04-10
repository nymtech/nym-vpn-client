// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../nym-vpnd/proto/commands.proto")?;

    EmitBuilder::builder()
        .all_build()
        .all_git()
        .all_rustc()
        .all_cargo()
        .emit()
        .expect("failed to extract build metadata");
    Ok(())
}
