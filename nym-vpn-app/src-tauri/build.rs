use std::fs;

use toml::Table;
const VPND_COMPAT_FILE: &str = "vpnd_compat.toml";

fn set_vpnd_compat() -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("../{}", VPND_COMPAT_FILE);
    let content = fs::read_to_string(&path)
        .inspect_err(|e| println!("cargo::warning=failed to read file `{}`: {e}", &path))?;
    let parsed = content
        .parse::<Table>()
        .inspect_err(|e| println!("cargo::warning=failed to parse `{}`: {e}", &path))?;
    let req = parsed.get("version").as_ref().and_then(|v| v.as_str());
    if let Some(v) = req {
        println!("cargo:rustc-env=VPND_COMPAT_REQ={}", v);
    } else {
        println!("cargo::warning=no version req found in `{}`", &path);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../{}", VPND_COMPAT_FILE);
    set_vpnd_compat().ok();
    build_info_build::build_script();

    tauri_build::build();
    Ok(())
}
