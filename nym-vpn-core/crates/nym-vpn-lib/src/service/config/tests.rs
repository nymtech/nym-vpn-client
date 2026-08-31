// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    GeoExclusionSettings, NetworkStatisticsConfig, SplitApp, SplitTunnelSettings,
};
use std::{net::IpAddr, str::FromStr};

use pretty_assertions::assert_eq;
use tempfile::tempdir;

use super::*;

// Test migrating from TOML to the latest JSON version
async fn run_migrate_toml_test(
    toml_content: &str,
    json_latest_content: &str,
    entry_point: nym_vpn_lib_types::EntryPoint,
    exit_point: nym_vpn_lib_types::ExitPoint,
) {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path();

    let network_config_path = config_path.join("tulips");
    let _ = fs::create_dir_all(&network_config_path).await;
    let toml_path = network_config_path.join(DEFAULT_CONFIG_FILE_TOML);
    let json_path = network_config_path.join(DEFAULT_CONFIG_FILE_JSON);

    // Write the TOML config file
    fs::write(&toml_path, toml_content).await.unwrap();

    // Read the TOML config and migrate it to latest JSON version.  The latest version of the
    // JSON will be written straight back to disk.
    let config_manager = VpnServiceConfigManager::new(&network_config_path, None)
        .await
        .unwrap();
    let config = config_manager.config();
    assert_eq!(config.entry_point, entry_point);
    assert_eq!(config.exit_point, exit_point);

    // The TOML file should be deleted and replaced with a JSON version
    assert!(!toml_path.exists());
    assert!(json_path.exists());

    // Read the JSON config
    let config_manager = VpnServiceConfigManager::new(&network_config_path, None)
        .await
        .unwrap();
    let config = config_manager.config();
    assert_eq!(config.entry_point, entry_point);
    assert_eq!(config.exit_point, exit_point);

    // Check the JSON is the right version and all snake-case
    let read_json_content = fs::read_to_string(&json_path).await.unwrap();
    assert_eq!(json_latest_content, read_json_content);
}

// Test migrating from an old JSON version to the latest JSON version
async fn run_migrate_json_test(json_old_content: &str, json_latest_content: &str) {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path();

    let network_config_path = config_path.join("tulips");
    let _ = fs::create_dir_all(&network_config_path).await;
    let json_path = network_config_path.join(DEFAULT_CONFIG_FILE_JSON);

    // Write the old JSON config file
    fs::write(&json_path, json_old_content).await.unwrap();

    // Read the old JSON config and migrate it to latest JSON.  The latest version of the
    // JSON will be written straight back to disk.
    let _config_manager = VpnServiceConfigManager::new(&network_config_path, None)
        .await
        .unwrap();

    // Check the JSON is the right version and all snake-case (ignore whitespace/order)
    let read_json_content = fs::read_to_string(&json_path).await.unwrap();
    let expected: serde_json::Value = serde_json::from_str(json_latest_content).unwrap();
    let actual: serde_json::Value = serde_json::from_str(&read_json_content).unwrap();
    assert_eq!(expected, actual);
}

// Test serializing and deserializing the config produces the same result
async fn run_serialize_test(config: nym_vpn_lib_types::VpnServiceConfig) {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path();

    let network_config_path = config_path.join("tulips");

    // Write the config to disk
    let mut config_manager = VpnServiceConfigManager::new(&network_config_path, None)
        .await
        .unwrap();
    config_manager.set_config(config.clone()).await;
    assert!(config_manager.write_to_file().await);
    drop(config_manager);

    // Read it back and compare it
    let config_manager = VpnServiceConfigManager::new(&network_config_path, None)
        .await
        .unwrap();
    let read_config = config_manager.config();
    assert_eq!(&config, read_config);
}

// Test reading a broken config falls back to a default config
async fn run_fallback_test(broken_json_content: &str) {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path();

    let network_config_path = config_path.join("tulips");
    let _ = fs::create_dir_all(&network_config_path).await;
    let json_path = network_config_path.join(DEFAULT_CONFIG_FILE_JSON);

    // Write the broken JSON config file
    fs::write(&json_path, broken_json_content).await.unwrap();

    // Ensure reading the broken config falls back to default config
    let config_manager = VpnServiceConfigManager::new(&network_config_path, None)
        .await
        .unwrap();

    assert_eq!(
        config_manager.config(),
        &nym_vpn_lib_types::VpnServiceConfig::default()
    );
}

#[tokio::test]
async fn test_service_config_migrate_toml_location() {
    let toml_content = r#"
[entry_point.Location]
location = "FR"

[exit_point.Location]
location = "BE"
"#;

    let json_content = r#"{
  "version": "v12",
  "entry_point": {
    "country": {
      "two_letter_iso_country_code": "FR"
    }
  },
  "exit_point": {
    "country": {
      "two_letter_iso_country_code": "BE"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "enable_custom_dns": false,
  "custom_dns": [],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true
  },
  "gateway_independence": {
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }
}"#;

    let entry_point = nym_vpn_lib_types::EntryPoint::Country {
        two_letter_iso_country_code: "FR".to_string(),
    };

    let exit_point = nym_vpn_lib_types::ExitPoint::Country {
        two_letter_iso_country_code: "BE".to_string(),
    };

    run_migrate_toml_test(toml_content, json_content, entry_point, exit_point).await;
}

#[tokio::test]
async fn test_service_config_migrate_gateway() {
    let toml_content = r#"
[entry_point.Gateway]
identity = [ 92, 25, 33, 77, 4, 117, 82, 117, 246, 239, 233, 11, 129, 183, 86, 194, 140, 95, 21, 196, 121, 130, 232, 195, 71, 173, 66, 124, 5, 14, 114, 107, ]

[exit_point.Gateway]
identity = [ 99, 23, 98, 234, 66, 161, 195, 63, 155, 161, 250, 207, 17, 158, 136, 114, 215, 90, 236, 161, 231, 176, 140, 190, 147, 182, 64, 171, 145, 31, 245, 186, ]
"#;

    let json_content = r#"{
  "version": "v12",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "gateway": {
      "identity": "7fp3cmzCvgeRgbB1ycTnK6RokjHNqPmCCSBG23gyxshj"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "enable_custom_dns": false,
  "custom_dns": [],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true
  },
  "gateway_independence": {
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }
}"#;

    let entry_point = nym_vpn_lib_types::EntryPoint::Gateway {
        identity: nym_vpn_lib_types::NodeIdentity::from_str(
            "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42",
        )
        .unwrap(),
    };

    let exit_point = nym_vpn_lib_types::ExitPoint::Gateway {
        identity: nym_vpn_lib_types::NodeIdentity::from_str(
            "7fp3cmzCvgeRgbB1ycTnK6RokjHNqPmCCSBG23gyxshj",
        )
        .unwrap(),
    };

    run_migrate_toml_test(toml_content, json_content, entry_point, exit_point).await;
}

#[tokio::test]
#[ignore] // Temporarily disabled due to issues with ExitPoint::Address (de)serialisation.
async fn test_service_config_migrate_toml_address() {
    let toml_content = r#"
[entry_point.Gateway]
identity = [92, 25, 33, 77, 4, 117, 82, 117, 246, 239, 233, 11, 129, 183, 86, 194, 140, 95, 21, 196, 121, 130, 232, 195, 71, 173, 66, 124, 5, 14, 114, 107]

[exit_point.Address]
address = [5, 56, 84, 195, 94, 238, 210, 124, 65, 143, 209, 144, 22, 255, 91, 188, 35, 50, 144, 234, 226, 114, 99, 40, 10, 102, 200, 170, 19, 162, 86, 134, 84, 20, 195, 193, 42, 194, 230, 153, 163, 90, 214, 216, 196, 166, 87, 132, 206, 215, 91, 89, 51, 98, 72, 156, 159, 248, 109, 225, 152, 204, 80, 97, 9, 62, 22, 108, 155, 95, 153, 29, 143, 48, 208, 5, 101, 231, 176, 93, 107, 229, 11, 225, 145, 1, 14, 219, 44, 88, 199, 206, 40, 185, 150, 151]
"#;

    let json_content = r#"{
  "version": "v12",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
            "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "custom_dns": null,
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true
  },
  "gateway_independence": {
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }
}"#;

    let entry_point = nym_vpn_lib_types::EntryPoint::Gateway {
        identity: nym_vpn_lib_types::NodeIdentity::from_str(
            "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42",
        )
        .unwrap(),
    };

    let exit_point = nym_vpn_lib_types::ExitPoint::Address {
            address: Box::new(
                nym_vpn_lib_types::Recipient::from_str("MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e").unwrap(),
            )
        };

    run_migrate_toml_test(toml_content, json_content, entry_point, exit_point).await;
}

#[tokio::test]
async fn test_service_config_migrate_toml_random() {
    let toml_content = r#"
entry_point = "Random"
exit_point = "Random"
"#;

    let json_content = r#"{
  "version": "v12",
  "entry_point": "random",
  "exit_point": "random",
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "enable_custom_dns": false,
  "custom_dns": [],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true
  },
  "gateway_independence": {
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }
}"#;

    let entry_point = nym_vpn_lib_types::EntryPoint::Random;
    let exit_point = nym_vpn_lib_types::ExitPoint::Random;

    run_migrate_toml_test(toml_content, json_content, entry_point, exit_point).await;
}

#[tokio::test]
async fn test_service_config_migrate_from_v1() {
    let json_v1_content = r#"{
  "version": "v1",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  }
}"#;

    let json_latest_content = r#"{
  "version": "v12",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "enable_custom_dns": false,
  "custom_dns": [],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true
  },
  "gateway_independence": {
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }
}"#;

    run_migrate_json_test(json_v1_content, json_latest_content).await;
}

#[tokio::test]
async fn test_service_config_migrate_from_v2() {
    let json_v2_content = r#"{
  "version": "v2",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "dns": "192.168.50.1",
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "netstack": false,
  "disable_poisson_rate": false,
  "disable_background_cover_traffic": false,
  "min_mixnode_performance": null,
  "min_gateway_mixnet_performance": null,
  "min_gateway_vpn_performance": null,
  "residential_exit": false
}"#;

    let json_latest_content = r#"{
  "version": "v12",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "enable_custom_dns": true,
  "custom_dns": [ "192.168.50.1" ],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true
  },
  "gateway_independence": {
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }
}"#;

    run_migrate_json_test(json_v2_content, json_latest_content).await;
}

#[tokio::test]
async fn test_service_config_migrate_from_v3() {
    let json_v3_content = r#"{
  "version": "v3",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "netstack": false,
  "disable_poisson_rate": false,
  "disable_background_cover_traffic": false,
  "min_mixnode_performance": null,
  "min_gateway_mixnet_performance": null,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "custom_dns": [
    "192.168.50.1",
    "2001:db8:85a3::8a2e:370:7334"
  ]
}"#;

    let json_latest_content = r#"{
  "version": "v12",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "enable_custom_dns": true,
  "custom_dns": [
    "192.168.50.1",
    "2001:db8:85a3::8a2e:370:7334"
  ],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true
  },
  "gateway_independence": {
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }
}"#;

    run_migrate_json_test(json_v3_content, json_latest_content).await;
}

#[tokio::test]
async fn test_service_config_migrate_from_v4() {
    let json_v4_content = r#"{
  "version": "v4",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "netstack": false,
  "disable_poisson_rate": true,
  "disable_background_cover_traffic": true,
  "min_mixnode_performance": 56,
  "min_gateway_mixnet_performance": 78,
  "min_gateway_vpn_performance": 23,
  "residential_exit": false,
  "enable_custom_dns": true,
  "custom_dns": [
    "192.168.50.1",
    "2001:db8:85a3::8a2e:370:7334"
  ],
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  }
}"#;

    let json_latest_content = r#"{
  "version": "v12",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": 23,
  "residential_exit": false,
  "enable_custom_dns": true,
  "custom_dns": [
    "192.168.50.1",
    "2001:db8:85a3::8a2e:370:7334"
  ],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": true,
    "disable_background_cover_traffic": true,
    "min_mixnode_performance": 56,
    "min_gateway_mixnet_performance": 78
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true
  },
  "gateway_independence": {
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }
}"#;

    run_migrate_json_test(json_v4_content, json_latest_content).await;
}

#[tokio::test]
async fn test_service_config_migrate_from_v5() {
    let json_v5_content = r#"{
  "version": "v5",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "netstack": false,
  "min_gateway_vpn_performance": 23,
  "residential_exit": false,
  "enable_custom_dns": true,
  "custom_dns": [
    "192.168.50.1",
    "2001:db8:85a3::8a2e:370:7334"
  ],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": 12,
    "average_packet_delay": 34,
    "message_sending_average_delay": 56,
    "disable_poisson_rate": true,
    "disable_background_cover_traffic": true,
    "min_mixnode_performance": 78,
    "min_gateway_mixnet_performance": 90
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  }
}"#;

    let json_latest_content = r#"{
  "version": "v12",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": 23,
  "residential_exit": false,
  "enable_custom_dns": true,
  "custom_dns": [
    "192.168.50.1",
    "2001:db8:85a3::8a2e:370:7334"
  ],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": 12,
    "average_packet_delay": 34,
    "message_sending_average_delay": 56,
    "disable_poisson_rate": true,
    "disable_background_cover_traffic": true,
    "min_mixnode_performance": 78,
    "min_gateway_mixnet_performance": 90
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true
  },
  "gateway_independence": {
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }
}"#;

    run_migrate_json_test(json_v5_content, json_latest_content).await;
}

#[tokio::test]
async fn test_service_config_migrate_from_v6() {
    let json_v6_content = r#"{
  "version": "v6",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_lewes_protocol": false,
  "netstack": false,
  "min_gateway_vpn_performance": 23,
  "residential_exit": false,
  "enable_custom_dns": true,
  "custom_dns": [
    "192.168.50.1",
    "2001:db8:85a3::8a2e:370:7334"
  ],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": 12,
    "average_packet_delay": 34,
    "message_sending_average_delay": 56,
    "disable_poisson_rate": true,
    "disable_background_cover_traffic": true,
    "min_mixnode_performance": 78,
    "min_gateway_mixnet_performance": 90
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  }
}"#;

    let json_latest_content = r#"{
  "version": "v12",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": 23,
  "residential_exit": false,
  "enable_custom_dns": true,
  "custom_dns": [
    "192.168.50.1",
    "2001:db8:85a3::8a2e:370:7334"
  ],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": 12,
    "average_packet_delay": 34,
    "message_sending_average_delay": 56,
    "disable_poisson_rate": true,
    "disable_background_cover_traffic": true,
    "min_mixnode_performance": 78,
    "min_gateway_mixnet_performance": 90
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true
  },
  "gateway_independence": {
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }
}"#;

    run_migrate_json_test(json_v6_content, json_latest_content).await;
}

#[tokio::test]
async fn test_service_config_fallback_default() {
    let broken_json_content = r#"{"version": "v1"}"#;

    run_fallback_test(broken_json_content).await;
}

// A persisted config with an invalid geo_exclusion section must not fail loading the
// whole config - only geo_exclusion should reset to default, everything else should load
// as persisted.
async fn run_geo_exclusion_fallback_test(geo_exclusion_json: &str) {
    let json_content = format!(
        r#"{{
  "version": "v12",
  "entry_point": {{
    "gateway": {{
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }}
  }},
  "exit_point": {{
    "address": {{
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }}
  }},
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "enable_custom_dns": false,
  "custom_dns": [],
  "mixnet_traffic": {{
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  }},
  "network_stats": {{
    "enabled": true,
    "allow_disconnected": false
  }},
  "split_tunnel": {{
    "enabled": false,
    "apps": []
  }},
  "geo_exclusion": {geo_exclusion_json},
  "gateway_selection_algorithm_config": {{
    "enable_geo_location": true
  }},
  "gateway_independence": {{
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }}
}}"#
    );

    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path();
    let network_config_path = config_path.join("tulips");
    let _ = fs::create_dir_all(&network_config_path).await;
    let json_path = network_config_path.join(DEFAULT_CONFIG_FILE_JSON);

    fs::write(&json_path, json_content).await.unwrap();

    let config_manager = VpnServiceConfigManager::new(&network_config_path, None)
        .await
        .unwrap();
    let config = config_manager.config();

    // The rest of the config should load as persisted, not fall back to default.
    assert_eq!(
        config.entry_point,
        nym_vpn_lib_types::EntryPoint::Gateway {
            identity: nym_vpn_lib_types::NodeIdentity::from_str(
                "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
            )
            .unwrap(),
        }
    );
    assert!(config.enable_two_hop);

    // The invalid geo_exclusion settings should reset to default rather than failing the
    // whole config load.
    assert_eq!(
        config.geo_exclusion,
        nym_vpn_lib_types::GeoExclusionSettings::default()
    );
}

#[tokio::test]
async fn test_service_config_geo_exclusion_invalid_country_resets_only_geo_exclusion() {
    run_geo_exclusion_fallback_test(
        r#"{
    "enabled": true,
    "listen_port": 1081,
    "excluded_countries": [
      "ZZ"
    ]
  }"#,
    )
    .await;
}

#[tokio::test]
async fn test_service_config_geo_exclusion_zero_port_resets_only_geo_exclusion() {
    run_geo_exclusion_fallback_test(
        r#"{
    "enabled": true,
    "listen_port": 0,
    "excluded_countries": [
      "CN"
    ]
  }"#,
    )
    .await;
}

#[tokio::test]
async fn test_service_config_geo_exclusion_reserved_port_resets_only_geo_exclusion() {
    run_geo_exclusion_fallback_test(
        r#"{
    "enabled": true,
    "listen_port": 1080,
    "excluded_countries": [
      "CN"
    ]
  }"#,
    )
    .await;
}

#[tokio::test]
async fn test_service_config_serialize_defaults() {
    let config = nym_vpn_lib_types::VpnServiceConfig::default();
    run_serialize_test(config).await;
}

#[tokio::test]
async fn test_service_config_serialize_full() {
    let config = nym_vpn_lib_types::VpnServiceConfig {
        entry_point: nym_vpn_lib_types::EntryPoint::Country {
            two_letter_iso_country_code: "US".to_string(),
        },
        exit_point: nym_vpn_lib_types::ExitPoint::Gateway {
            identity: nym_vpn_lib_types::NodeIdentity::from_str(
                "7fp3cmzCvgeRgbB1ycTnK6RokjHNqPmCCSBG23gyxshj",
            )
            .unwrap(),
        },
        allow_lan: true,
        disable_ipv6: true,
        enable_two_hop: true,
        enable_bridges: false,
        enable_ad_blocking: true,
        netstack: true,
        min_gateway_vpn_performance: Some(1u8),
        residential_exit: true,
        enable_custom_dns: true,
        fronting_mode: nym_vpn_lib_types::FrontingMode::OnRetry,
        custom_dns: vec![
            IpAddr::from_str("192.168.50.1").unwrap(),
            IpAddr::from_str("2001:db8:85a3::8a2e:370:7334").unwrap(),
        ],
        mixnet_traffic: nym_vpn_lib_types::MixnetTrafficConfig {
            poisson_parameter_for_loop_cover_stream: Some(10),
            average_packet_delay: Some(200),
            message_sending_average_delay: Some(150),
            disable_poisson_rate: false,
            disable_background_cover_traffic: false,
            min_mixnode_performance: Some(65),
            min_gateway_mixnet_performance: Some(75),
        },
        network_stats: NetworkStatisticsConfig {
            enabled: true,
            allow_disconnected: false,
        },
        split_tunnel: SplitTunnelSettings {
            enabled: true,
            apps: vec![SplitApp {
                path: "/Applications/Firefox.app/Contents/MacOS/firefox".to_owned(),
            }],
        },
        geo_exclusion: GeoExclusionSettings {
            enabled: true,
            listen_port: 1081,
            excluded_countries: vec!["CN".to_string(), "RU".to_string()],
        },
        gateway_selection_algorithm_config:
            nym_vpn_lib_types::GatewaySelectionAlgorithmConfig::default(),
        gateway_independence: nym_vpn_lib_types::GatewayIndependence {
            different_node_family: true,
            different_asn: true,
            different_subnet: true,
            ..Default::default()
        },
    };
    run_serialize_test(config).await;
}

#[tokio::test]
async fn test_service_config_migrate_from_v7() {
    let json_v7_content = r#"{
  "version": "v7",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_lewes_protocol": false,
  "enable_ad_blocking": false,
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "enable_custom_dns": false,
  "custom_dns": [],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  }
}"#;

    let json_latest_content = r#"{
  "version": "v12",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "enable_custom_dns": false,
  "custom_dns": [],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true
  },
  "gateway_independence": {
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }
}"#;

    run_migrate_json_test(json_v7_content, json_latest_content).await;
}

#[tokio::test]
async fn test_service_config_migrate_from_v8() {
    let json_v8_content = r#"{
  "version": "v8",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_lewes_protocol": false,
  "enable_ad_blocking": false,
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "enable_custom_dns": false,
  "custom_dns": [],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  }
}"#;

    let json_latest_content = r#"{
  "version": "v12",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "enable_custom_dns": false,
  "custom_dns": [],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true
  },
  "gateway_independence": {
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }
}"#;

    run_migrate_json_test(json_v8_content, json_latest_content).await;
}

#[tokio::test]
async fn test_service_config_migrate_from_v9() {
    let json_v9_content = r#"{
  "version": "v9",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_lewes_protocol": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "enable_custom_dns": false,
  "custom_dns": [],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true,
    "gateway_selection_algorithm": "explicit"
  }
}"#;

    let json_latest_content = r#"{
  "version": "v12",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "allow_lan": false,
  "disable_ipv6": false,
  "enable_two_hop": true,
  "enable_bridges": false,
  "enable_ad_blocking": false,
  "fronting_mode": "on_retry",
  "netstack": false,
  "min_gateway_vpn_performance": null,
  "residential_exit": false,
  "enable_custom_dns": false,
  "custom_dns": [],
  "mixnet_traffic": {
    "poisson_parameter_for_loop_cover_stream": null,
    "average_packet_delay": null,
    "message_sending_average_delay": null,
    "disable_poisson_rate": false,
    "disable_background_cover_traffic": false,
    "min_mixnode_performance": null,
    "min_gateway_mixnet_performance": null
  },
  "network_stats": {
    "enabled": true,
    "allow_disconnected": false
  },
  "split_tunnel": {
    "enabled": false,
    "apps": []
  },
  "geo_exclusion": {
    "enabled": false,
    "listen_port": 1081,
    "excluded_countries": [
      "CN"
    ]
  },
  "gateway_selection_algorithm_config": {
    "enable_geo_location": true
  },
  "gateway_independence": {
    "enable_notifications": true,
    "different_node_family": true,
    "different_asn": true,
    "different_subnet": true
  }
}"#;

    run_migrate_json_test(json_v9_content, json_latest_content).await;
}
