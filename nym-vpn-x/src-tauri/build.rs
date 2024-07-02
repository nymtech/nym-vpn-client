fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_info_build::build_script();

    tauri_build::build();
    Ok(())
}
