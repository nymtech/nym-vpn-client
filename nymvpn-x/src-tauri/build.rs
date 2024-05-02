fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO restore this
    // build_info_build::build_script();

    tauri_build::build();
    Ok(())
}
