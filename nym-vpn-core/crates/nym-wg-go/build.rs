// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    env,
    path::{self, PathBuf},
};

fn main() {
    let target = env::var("TARGET").expect("target is not set");
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("target os is not set");

    // Support BUILD_TOP for sandbox builds where the source directory is read-only.
    // Falls back to the traditional relative path from CARGO_MANIFEST_DIR.
    let mut build_dir = path::absolute(if let Ok(build_top) = env::var("BUILD_TOP") {
        PathBuf::from(build_top).join("build/lib")
    } else {
        let manifest_path = env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir is not set");
        PathBuf::from(manifest_path).join("../../../build/lib")
    })
    .expect("failed to get absolute path to build directory");

    build_dir.push(&target);

    println!("cargo::rustc-link-search={}", build_dir.display());

    let lib_prefix = if target_os.as_str() == "windows" {
        "lib"
    } else {
        ""
    };

    let link_type = match target_os.as_str() {
        "android" => "",
        "linux" | "macos" | "ios" => "=static",
        "windows" => "=dylib",
        _ => panic!("Unsupported platform: {target_os}"),
    };
    println!("cargo:rustc-link-lib{link_type}={lib_prefix}wg");
}
