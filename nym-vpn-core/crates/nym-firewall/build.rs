// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{env, path::PathBuf};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("target os is not set");

    if target_os == "windows" {
        let manifest_path = env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir is not set");
        let target = env::var("TARGET").expect("target is not set");
        let mut build_dir = PathBuf::from(manifest_path)
            .join("../../../build/winfw")
            .canonicalize()
            .expect("failed to canonicalize build dir path");
        build_dir.push(target);

        //println!("cargo::rustc-link-search={}", build_dir.display());
        //println!("cargo:rustc-link-lib=static=winfw");
    }
}
