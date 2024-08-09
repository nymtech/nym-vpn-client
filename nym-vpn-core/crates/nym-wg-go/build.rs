use std::path::PathBuf;

fn main() {
    let target = std::env::var("TARGET").expect("TARGET is not set");
    let build_dir = wg_build_dir(&target);

    if let Some(build_dir) = build_dir {
        let abs_build_dir = build_dir
            .canonicalize()
            .expect("failed to canonicalize build dir path");

        println!("cargo::rustc-link-search={}", abs_build_dir.display());
    }

    println!("cargo:rustc-link-lib=wg");
}

fn wg_build_dir(target: &str) -> Option<PathBuf> {
    let manifest_path = std::env::var_os("CARGO_MANIFEST_DIR")?;
    let mut build_dir = PathBuf::from(manifest_path).join("../../../build/lib");
    build_dir.push(target);
    Some(build_dir)
}
