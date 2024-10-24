// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{env, path::PathBuf};

fn main() {
    let manifest_path = env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir is not set");
    let target = env::var("TARGET").expect("target is not set");
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("target os is not set");

    let mut build_dir = PathBuf::from(manifest_path)
        .join("../../../build/lib")
        .canonicalize()
        .expect("failed to canonicalize build dir path");

    build_dir.push(target);

    // CI may only provide universal builds
    if target_os == "macos" {
        let target_dir_exists = build_dir
            .try_exists()
            .expect("failed to check existence of target dir");

        if !target_dir_exists {
            build_dir.pop();
            build_dir.push("universal-apple-darwin");
        }
    }

    println!("cargo::rustc-link-search={}", build_dir.display());

    let link_type = match target_os.as_str() {
        "android" => "",
        "linux" | "macos" | "ios" => "=static",
        "windows" => "dylib",
        _ => panic!("Unsupported platform: {}", target_os),
    };
    println!("cargo:rustc-link-lib{}=wg", link_type);
}
