// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, process::Command};

use vergen_gitcl::{BuildBuilder, CargoBuilder, Emitter, GitclBuilder, RustcBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_go()?;

    Ok(Emitter::default()
        .add_instructions(&BuildBuilder::all_build()?)?
        .add_instructions(&CargoBuilder::all_cargo()?)?
        .add_instructions(&GitclBuilder::all_git()?)?
        .add_instructions(&RustcBuilder::all_rustc()?)?
        .emit()?)
}

fn build_go() -> Result<(), Box<dyn std::error::Error>> {
    const LIB_NAME: &str = "netstack_ping";

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("target arch is not set");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("target os is not set");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is not set"));

    if target_os == "windows" {
        return Ok(());
    }

    let go_target = match target_os.as_str() {
        "macos" => "darwin".to_owned(),
        "linux" => target_os.to_owned(),
        _ => panic!("unsupported target: {target_os}"),
    };

    let go_arch = match target_arch.as_str() {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => panic!("unsupported architecture: {target_arch}"),
    };

    let src_dir = PathBuf::from("netstack_ping").canonicalize()?;

    println!("cargo::rerun-if-changed={}", src_dir.display());

    let binary_out_path = out_dir.join(format!("lib{LIB_NAME}.a"));

    let mut command = Command::new("go");

    if target_os == "macos" {
        let deployment_target =
            std::env::var_os("MACOSX_DEPLOYMENT_TARGET").unwrap_or("10.13".into());
        command.env("MACOSX_DEPLOYMENT_TARGET", deployment_target);
    }

    let mut child = command
        .env("CGO_ENABLED", "1")
        .env("GOOS", go_target)
        .env("GOARCH", go_arch)
        .current_dir(src_dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .arg("build")
        .arg("-trimpath")
        .arg("-buildvcs=false")
        .arg("-v")
        .arg("-o")
        .arg(binary_out_path)
        .arg("-buildmode")
        .arg("c-archive")
        .arg("lib.go")
        .spawn()?;
    let status = child.wait()?;
    if !status.success() {
        return Err(Box::new(std::io::Error::other(format!(
            "Failed to build {LIB_NAME}"
        ))));
    }

    println!("cargo::rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static={LIB_NAME}");

    Ok(())
}
