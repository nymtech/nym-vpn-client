// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::{env, path::PathBuf};

fn main() {
    let manifest_path = env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir is not set");
    let target = env::var("TARGET").expect("target is not set");
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("target os is not set");

    let build_dir = PathBuf::from(manifest_path)
        .join("../../../build/lib")
        .join(target);
    let abs_build_dir = build_dir
        .canonicalize()
        .expect("failed to canonicalize build dir path");
    println!("cargo::rustc-link-search={}", abs_build_dir.display());

    let link_type = match target_os.as_str() {
        "android" => "",
        "linux" | "macos" | "ios" => "=static",
        "windows" => "dylib",
        _ => panic!("Unsupported platform: {}", target_os),
    };
    println!("cargo:rustc-link-lib{}=wg", link_type);
}
