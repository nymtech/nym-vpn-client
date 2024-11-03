// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

fn main() {
    let json_path = Path::new("env").join("mainnet_discovery.json");

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
            fn default() -> Self {{
                Self {{
                    network_name: "{}".to_string(),
                    nym_api_url: "{}".parse().expect("Failed to parse NYM API URL"),
                    nym_vpn_api_url: "{}".parse().expect("Failed to parse NYM VPN API URL"),
                }}
            }}
        }}
        "#,
        network_name, nym_api_url, nym_vpn_api_url
    );

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("default_discovery.rs");
    std::fs::write(&dest_path, generated_code).expect("Failed to write generated code");
}
