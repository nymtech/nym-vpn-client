// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{env, path::PathBuf};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("target os is not set");
    if target_os != "windows" {
        return;
    }

    let manifest_path = env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir is not set");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("target arch is not set");
    let profile = env::var("PROFILE").expect("profile is not set");

    let build_dir = PathBuf::from(manifest_path)
        .join("../../../build/winfw")
        .canonicalize()
        .expect("failed to canonicalize build dir path");
    let cpp_arch = match arch.as_str() {
        "x86_64" => "x64",
        "aarch64" => "ARM64",
        other => {
            panic!("Unknown architecture: {}", other);
        }
    };
    let cpp_profile = match profile.as_str() {
        "release" => "Release",
        "debug" => "Debug",
        other => {
            panic!("Unknown profile: {}", other);
        }
    };
    let link_search_dir = build_dir.join(format!("{}-{}", cpp_arch, cpp_profile));

    println!("cargo::rustc-link-search={}", link_search_dir.display());
    println!("cargo:rustc-link-lib=dylib=winfw");
    println!(
        "cargo:rerun-if-changed={}\\winfw.dll",
        link_search_dir.display()
    );
}
