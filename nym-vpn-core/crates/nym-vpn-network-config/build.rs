// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

const DEFAULT_DIR: &str = "default";
const MAINNET_DISCOVERY_JSON: &str = "mainnet_discovery.json";
const DEFAULT_ENVS_JSON: &str = "envs.json";

fn default_envs() {
    let json_path = Path::new(DEFAULT_DIR).join(DEFAULT_ENVS_JSON);

    let json_str = std::fs::read_to_string(json_path).expect("Failed to read JSON file");
    let networks: Vec<String> = serde_json::from_str(&json_str).expect("Failed to parse JSON file");

    let networks_literal = networks
        .iter()
        .map(|s| format!("\"{}\"", s))
        .collect::<Vec<String>>()
        .join(", ");

    let generated_code = format!(
        r#"
        impl Default for RegisteredNetworks {{
            fn default() -> Self {{
                RegisteredNetworks {{
                    inner: [ {networks_literal} ]
                        .iter()
                        .cloned()
                        .map(String::from)
                        .collect::<std::collections::HashSet<_>>(),
                }}
            }}
        }}
        "#,
        networks_literal = networks_literal,
    );

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("default_envs.rs");
    std::fs::write(&dest_path, generated_code).expect("Failed to write generated code");
}

fn default_mainnet_discovery() {
    let json_path = Path::new(DEFAULT_DIR).join(MAINNET_DISCOVERY_JSON);
    println!("cargo:rerun-if-changed={}", json_path.display());

    let json_str = std::fs::read_to_string(json_path).expect("Failed to read JSON file");
    let json_value: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse JSON file");

    let network_name = json_value["network_name"]
        .as_str()
        .expect("Failed to parse network name");
    let nym_api_url = json_value["nym_api_url"]
        .as_str()
        .expect("Failed to parse nym_api_url");
    let nym_vpn_api_url = json_value["nym_vpn_api_url"]
        .as_str()
        .expect("Failed to parse nym_vpn_api_url");

    let generated_code = format!(
        r#"
        impl Default for Discovery {{
            #[allow(clippy::expect_used)]
            fn default() -> Self {{
                Self {{
                    network_name: "{network_name}".to_string(),
                    nym_api_url: "{nym_api_url}".parse().expect("Failed to parse NYM API URL"),
                    nym_vpn_api_url: "{nym_vpn_api_url}".parse().expect("Failed to parse NYM VPN API URL"),
                    account_management: Default::default(),
                    feature_flags: Default::default(),
                    system_messages: Default::default(),
                }}
            }}
        }}


        impl Discovery {{
            pub(crate) const DEFAULT_VPN_API_URL: &str =  "{nym_vpn_api_url}";
        }}
        "#
    );

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("default_discovery.rs");
    std::fs::write(&dest_path, generated_code).expect("Failed to write generated code");
}

fn main() {
    default_envs();
    default_mainnet_discovery();

    println!("cargo:rerun-if-changed={DEFAULT_DIR}");
}
