fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    // XPC protocol definition
    if target_os == "macos" {
        // Rebuild if the ObjC file changes
        println!("cargo:rerun-if-changed=src/xpc/protocols.m");

        cc::Build::new()
            .file("src/xpc/protocols.m")
            .flag("-fobjc-arc")
            .compile("protocols");

        println!("cargo:rustc-link-lib=framework=Foundation");
    }
}
