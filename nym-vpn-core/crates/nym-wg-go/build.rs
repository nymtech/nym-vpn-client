// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::{env, path::PathBuf};

fn main() {
    let manifest_path = env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir is not set");
    let target = env::var("TARGET").expect("target is not set");
    let build_dir = PathBuf::from(manifest_path)
        .join("../../../build/lib")
        .join(target);
    let abs_build_dir = build_dir
        .canonicalize()
        .expect("failed to canonicalize build dir path");

    println!(
        "cargo::rustc-link-search=native={}",
        abs_build_dir.display()
    );
    println!("cargo::rustc-link-lib=static=wg");
}
