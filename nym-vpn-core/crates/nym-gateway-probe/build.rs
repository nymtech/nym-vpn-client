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
    let out_dir = if target_os == "windows" {
        let manifest_path =
            std::env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir is not set");
        let build_dir = PathBuf::from(manifest_path)
            .join("../../../build/gateway-probe")
            .join(&target_arch);
        // canonicalize() will fail if the build directory does not exist
        if !build_dir.exists() {
            std::fs::create_dir_all(&build_dir).expect("failed to create build dir");
        }
        let build_dir = build_dir
            .canonicalize()
            .expect("failed to canonicalize build dir path");
        build_dir
    } else {
        PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is not set"))
    };

    let lib_prefix = if target_os == "windows" { "lib" } else { "" };
    let lib_ext = if target_os == "windows" { "dll" } else { "a" };

    let go_build_mode = if target_os == "windows" {
        "c-shared"
    } else {
        "c-archive"
    };

    let go_target = match target_os.as_str() {
        "macos" => "darwin".to_owned(),
        "linux" | "windows" => target_os.to_owned(),
        _ => panic!("unsupported target: {target_os}"),
    };

    let go_arch = match target_arch.as_str() {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => panic!("unsupported architecture: {target_arch}"),
    };

    let src_dir = PathBuf::from("netstack_ping").canonicalize()?;

    println!("cargo::rerun-if-changed={}", src_dir.display());

    let binary_out_path = out_dir
        .join(format!("lib{LIB_NAME}"))
        .with_extension(lib_ext);

    let mut command = Command::new("go");

    match target_os.as_str() {
        "macos" => {
            let deployment_target =
                std::env::var_os("MACOSX_DEPLOYMENT_TARGET").unwrap_or("10.13".into());
            command.env("MACOSX_DEPLOYMENT_TARGET", deployment_target);
        }
        "windows" => {
            let msys2_location = std::env::var_os("MSYS2_LOCATION")
                .map(PathBuf::from)
                .unwrap_or(PathBuf::from(r#"C:\msys64"#));

            let cc = match target_arch.as_str() {
                "aarch64" => Some(msys2_location.join(r#"clangarm64\bin\aarch64-w64-mingw32-cc"#)),
                "x86_64" => Some(msys2_location.join(r#"mingw64\bin\x86_64-w64-mingw32-cc"#)),
                _ => None,
            };
            if let Some(cc) = cc {
                command.env("CC", cc);
            }
        }
        _ => {}
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
        .arg(go_build_mode)
        .arg("lib.go")
        .spawn()?;
    let status = child.wait()?;
    if !status.success() {
        return Err(Box::new(std::io::Error::other(format!(
            "Failed to build {LIB_NAME}"
        ))));
    }

    println!("cargo::rustc-link-search={}", out_dir.display());

    let link_type = match target_os.as_str() {
        "linux" | "macos" => "=static",
        "windows" => "=dylib",
        _ => panic!("Unsupported platform: {target_os}"),
    };
    println!("cargo:rustc-link-lib{link_type}={lib_prefix}{LIB_NAME}");

    Ok(())
}
