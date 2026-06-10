// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=nym-file-updater.exe.manifest");

    // On Windows, the UAC "Application Installation Detection" heuristic treats any
    // executable whose name contains "update" or "updater" as an installer and
    // silently requires elevation.  Embed a manifest that explicitly declares
    // asInvoker so Windows does not apply the heuristic to test binaries or the
    // production binary.
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("nym-file-updater.exe.manifest");

        println!("cargo:rustc-link-arg=/MANIFEST:EMBED",);
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
    }
}
