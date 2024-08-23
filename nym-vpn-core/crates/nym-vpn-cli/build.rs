// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use vergen::EmitBuilder;

fn main() {
    EmitBuilder::builder()
        .all_build()
        .all_git()
        .all_rustc()
        .all_cargo()
        .emit()
        .expect("failed to extract build metadata");
}
