use std::path::PathBuf;

fn main() {
    let build_dir = wg_build_dir();

    if let Some(build_dir) = build_dir {
        println!("Add link search path: {}", build_dir.display());
        println!("cargo::rustc-link-search={}", build_dir.display());
    } else {
        println!("Could not locate libwg build dir.");
    }

    println!("cargo:rustc-link-lib=wg");
}

fn wg_build_dir() -> Option<PathBuf> {
    let manifest_path = std::env::var_os("CARGO_MANIFEST_DIR")?;
    let mut build_dir = PathBuf::from(manifest_path).join("../../../build/lib");

    let target_dir = if cfg!(target_os = "ios") {
        Some("universal-apple-ios")
    } else if cfg!(target_os = "macos") {
        Some("universal-apple-darwin")
    } else {
        // todo: support other platforms
        None
    }?;

    build_dir.push(target_dir);
    Some(build_dir)
}
