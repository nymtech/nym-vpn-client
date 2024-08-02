fn main() {
    let mut cargo_licenses = std::process::Command::new("make");
    cargo_licenses
        .output()
        .expect("Failed to generate licenses");
}
