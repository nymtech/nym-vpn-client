// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use vergen::EmitBuilder;

#[derive(serde::Deserialize)]
struct Environments {
    environments: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    EmitBuilder::builder()
        .all_build()
        .all_git()
        .all_rustc()
        .all_cargo()
        .emit()
        .expect("failed to extract build metadata");

    // Ensure the build script runs when env.json changes
    println!("cargo:rerun-if-changed=env.json");

    // Read env.json file and store it as a constant
    let env_path = "../../env/env.json";
    let env_str = std::fs::read_to_string(env_path).expect("Failed to read env file");

    let envs: Environments = serde_json::from_str(&env_str).expect("Failed to parse env file");

    // Generate Rust code
    let environments: Vec<_> = envs
        .environments
        .iter()
        .map(|env| {
            let env_lit = syn::LitStr::new(env, proc_macro2::Span::call_site());
            quote::quote! { #env_lit.to_string() }
        })
        .collect();

    let generated_code = quote::quote! {
        use crate::Environments;

        pub fn get_environments() -> Environments {
            Environments {
                environments: vec![#(#environments),*],
            }
        }
    };

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::PathBuf::from(&out_dir).join("env.rs");
    std::fs::write(&dest_path, generated_code.to_string()).expect("Unable to write env.rs");

    Ok(())
}
