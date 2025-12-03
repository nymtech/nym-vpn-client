// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{
    config::{
        DEFAULT_CONFIG_FILE_JSON, DEFAULT_CONFIG_FILE_TOML, VpnServiceConfigExt,
        VpnServiceConfigVersion, legacy,
    },
    error::{Error, Result},
    read_json_config_file, read_toml_config_file, write_json_config_file,
};
use nym_common::trace_err_chain;
use nym_registration_client::MixnetClientConfig;
use nym_vpn_lib::{
    DEFAULT_MIN_GATEWAY_PERFORMANCE, DEFAULT_MIN_MIXNODE_PERFORMANCE,
    tunnel_state_machine::{
        DnsOptions, GatewayPerformanceOptions, MixnetTunnelOptions, TunnelSettings,
        WireguardMultihopMode, WireguardTunnelOptions,
    },
};
use std::{
    net::IpAddr,
    path::{Path, PathBuf},
};
use tokio::{fs, sync::broadcast};

pub struct VpnServiceConfigManager {
    json_config_path: PathBuf,
    config: nym_vpn_lib_types::VpnServiceConfig,

    // Used to send `ConfigChanged` events when the config is updated.
    // It's only optional to simplify testing.
    tunnel_event_tx: Option<broadcast::Sender<nym_vpn_lib_types::TunnelEvent>>,
}

impl VpnServiceConfigManager {
    pub async fn new(
        network_config_dir: &Path,
        tunnel_event_tx: Option<broadcast::Sender<nym_vpn_lib_types::TunnelEvent>>,
    ) -> Result<Self> {
        let toml_config_path = network_config_dir.join(DEFAULT_CONFIG_FILE_TOML);
        let json_config_path = network_config_dir.join(DEFAULT_CONFIG_FILE_JSON);
        let (config, version) =
            match Self::read_from_file(&toml_config_path, &json_config_path).await {
                Ok((config, version)) => (config, version),
                Err(e) => {
                    trace_err_chain!(
                        e,
                        "Failed to read service config file {}; using default",
                        json_config_path.display()
                    );
                    (nym_vpn_lib_types::VpnServiceConfig::default(), None)
                }
            };

        let config_manager = Self {
            json_config_path,
            config,
            tunnel_event_tx,
        };

        // If we didn't read the latest version then write the config straight back to file
        if version != Some(VpnServiceConfigVersion::latest()) {
            config_manager.write_to_file().await;
        }

        // If the deprecated TOML file exists then remove it
        if toml_config_path.exists() {
            tracing::info!(
                "Removing deprecated config file {}",
                toml_config_path.display()
            );
            if let Err(e) = fs::remove_file(&toml_config_path).await {
                trace_err_chain!(e, "Failed to remove deprecated config file");
            }
        }

        Ok(config_manager)
    }

    pub fn config(&self) -> &nym_vpn_lib_types::VpnServiceConfig {
        &self.config
    }

    #[cfg(test)]
    pub async fn set_config(&mut self, config: nym_vpn_lib_types::VpnServiceConfig) {
        if self.config != config {
            self.config = config;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_entry_point(&mut self, entry_point: nym_vpn_lib_types::EntryPoint) {
        if self.config.entry_point != entry_point {
            self.config.entry_point = entry_point;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_exit_point(&mut self, exit_point: nym_vpn_lib_types::ExitPoint) {
        if self.config.exit_point != exit_point {
            self.config.exit_point = exit_point;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_disable_ipv6(&mut self, disable_ipv6: bool) {
        if self.config.disable_ipv6 != disable_ipv6 {
            self.config.disable_ipv6 = disable_ipv6;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_enable_two_hop(&mut self, enable_two_hop: bool) {
        if self.config.enable_two_hop != enable_two_hop {
            self.config.enable_two_hop = enable_two_hop;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_netstack(&mut self, netstack: bool) {
        if self.config.netstack != netstack {
            self.config.netstack = netstack;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_allow_lan(&mut self, allow_lan: bool) {
        if self.config.allow_lan != allow_lan {
            self.config.allow_lan = allow_lan;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_enable_bridges(&mut self, enable_bridges: bool) {
        if self.config.enable_bridges != enable_bridges {
            self.config.enable_bridges = enable_bridges;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_residential_exit(&mut self, residential_only: bool) {
        if self.config.residential_exit != residential_only {
            self.config.residential_exit = residential_only;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_enable_custom_dns(&mut self, enable_custom_dns: bool) {
        if self.config.enable_custom_dns != enable_custom_dns {
            self.config.enable_custom_dns = enable_custom_dns;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_custom_dns(&mut self, custom_dns: Vec<IpAddr>) {
        if self.config.custom_dns != custom_dns {
            self.config.custom_dns = custom_dns;
            self.save_config_and_send_event().await;
        }
    }

    #[allow(unused)]
    pub async fn set_disable_poisson_rate(&mut self, disable_poisson_rate: bool) {
        if self.config.disable_poisson_rate != disable_poisson_rate {
            self.config.disable_poisson_rate = disable_poisson_rate;
            self.save_config_and_send_event().await;
        }
    }

    #[allow(unused)]
    pub async fn set_disable_background_cover_traffic(&mut self, disable: bool) {
        if self.config.disable_background_cover_traffic != disable {
            self.config.disable_background_cover_traffic = disable;
            self.save_config_and_send_event().await;
        }
    }

    #[allow(unused)]
    pub async fn set_min_mixnode_performance(&mut self, min_mixnode_performance: Option<u8>) {
        if self.config.min_mixnode_performance != min_mixnode_performance {
            self.config.min_mixnode_performance = min_mixnode_performance.map(|u| u.min(100));
            self.save_config_and_send_event().await;
        }
    }

    #[allow(unused)]
    pub async fn set_min_gateway_mixnet_performance(
        &mut self,
        min_gateway_mixnet_performance: Option<u8>,
    ) {
        if self.config.min_gateway_mixnet_performance != min_gateway_mixnet_performance {
            self.config.min_gateway_mixnet_performance =
                min_gateway_mixnet_performance.map(|u| u.min(100));
            self.save_config_and_send_event().await;
        }
    }

    #[allow(unused)]
    pub async fn set_min_gateway_vpn_performance(
        &mut self,
        min_gateway_vpn_performance: Option<u8>,
    ) {
        if self.config.min_gateway_vpn_performance != min_gateway_vpn_performance {
            self.config.min_gateway_vpn_performance =
                min_gateway_vpn_performance.map(|u| u.min(100));
            self.save_config_and_send_event().await;
        }
    }

    async fn save_config_and_send_event(&self) {
        // This function already logs
        let _ = self.write_to_file().await;

        // Notify all clients that the config has changed
        if let Some(tx) = self.tunnel_event_tx.as_ref() {
            match tx.send(nym_vpn_lib_types::TunnelEvent::ConfigChanged(Box::new(
                self.config.clone(),
            ))) {
                Ok(recv_count) => {
                    tracing::info!("Sent config changed event to {recv_count} receivers");
                }
                Err(e) => {
                    tracing::error!("Failed to send config changed event: {e}");
                }
            }
        }
    }

    /// Returns the configuration as well as the version read from file.
    async fn read_from_file(
        toml_config_path: &Path,
        json_config_path: &Path,
    ) -> Result<(
        nym_vpn_lib_types::VpnServiceConfig,
        Option<VpnServiceConfigVersion>,
    )> {
        let (config, version) = if json_config_path.exists() {
            let ext_config = read_json_config_file::<VpnServiceConfigExt>(json_config_path)
                .await
                .map_err(Error::ConfigSetup)?;
            let version = ext_config.version();

            tracing::info!(
                "Read service config version {version} from {}",
                json_config_path.display()
            );

            let config = nym_vpn_lib_types::VpnServiceConfig::try_from(ext_config)
                .map_err(Error::ConfigSetup)?;

            (config, Some(version))
        } else if toml_config_path.exists() {
            let legacy_config = read_toml_config_file::<legacy::VpnServiceConfig>(toml_config_path)
                .await
                .map_err(Error::ConfigSetup)?;

            tracing::info!("Read service config from {}", toml_config_path.display());

            let config = nym_vpn_lib_types::VpnServiceConfig::try_from(legacy_config)
                .map_err(Error::ConfigSetup)?;

            (config, None)
        } else {
            tracing::info!("Using default service config");

            (nym_vpn_lib_types::VpnServiceConfig::default(), None)
        };

        Ok((config, version))
    }

    // Only public for unit tests
    pub(crate) async fn write_to_file(&self) -> bool {
        let ext_config =
            match VpnServiceConfigExt::try_from(&self.config).map_err(Error::ConfigSetup) {
                Ok(ext_config) => ext_config,
                Err(e) => {
                    tracing::error!("Failed to convert service config to JSON: {e}");
                    return false;
                }
            };
        let version = ext_config.version();

        match write_json_config_file(&self.json_config_path, &ext_config)
            .await
            .map_err(Error::ConfigSetup)
        {
            Ok(_) => {
                tracing::info!(
                    "Writing service config version {version} to {}",
                    self.json_config_path.display()
                );
                true
            }
            Err(e) => {
                tracing::error!(
                    "Failed to write service config version {version} to {}: {e}",
                    self.json_config_path.display()
                );
                false
            }
        }
    }

    pub fn generate_tunnel_settings(&self) -> TunnelSettings {
        tracing::info!("Using config: {:?}", self.config);

        let gateway_options = GatewayPerformanceOptions {
            mixnet_min_performance: self.config.min_gateway_mixnet_performance,
            vpn_min_performance: self.config.min_gateway_vpn_performance,
        };

        let mixnet_client_config = MixnetClientConfig {
            disable_poisson_rate: self.config.disable_poisson_rate,
            disable_background_cover_traffic: self.config.disable_background_cover_traffic,
            min_mixnode_performance: Some(
                self.config
                    .min_mixnode_performance
                    .unwrap_or(DEFAULT_MIN_MIXNODE_PERFORMANCE),
            ),
            min_gateway_performance: Some(
                self.config
                    .min_gateway_mixnet_performance
                    .unwrap_or(DEFAULT_MIN_GATEWAY_PERFORMANCE),
            ),
        };

        let tunnel_type = if self.config.enable_two_hop {
            nym_vpn_lib_types::TunnelType::Wireguard
        } else {
            nym_vpn_lib_types::TunnelType::Mixnet
        };

        let dns = if self.config.enable_custom_dns && !self.config.custom_dns.is_empty() {
            DnsOptions::Custom(self.config.custom_dns.clone())
        } else {
            DnsOptions::default()
        };

        TunnelSettings {
            enable_ipv6: !self.config.disable_ipv6,
            allow_lan: self.config.allow_lan,
            residential_exit: self.config.residential_exit,
            tunnel_type,
            mixnet_tunnel_options: MixnetTunnelOptions { mtu: None },
            wireguard_tunnel_options: WireguardTunnelOptions {
                multihop_mode: if self.config.netstack {
                    WireguardMultihopMode::Netstack
                } else {
                    WireguardMultihopMode::TunTun
                },
                enable_bridges: self.config.enable_bridges,
            },
            gateway_performance_options: gateway_options,
            mixnet_client_config: Some(mixnet_client_config),
            entry_point: Box::new(self.config.entry_point.clone()),
            exit_point: Box::new(self.config.exit_point.clone()),
            dns,
        }
    }
}
