// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::IpAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use nym_common::trace_err_chain;
use nym_http_api_client::{Client, FrontPolicy};
use nym_registration_client::MixnetClientConfig;
use nym_vpn_lib_types::MixnetTrafficConfigValidationError;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use nym_vpn_lib_types::SplitApp;
use tokio::{fs, sync::broadcast};

use crate::{
    DEFAULT_MIN_GATEWAY_PERFORMANCE, DEFAULT_MIN_MIXNODE_PERFORMANCE,
    service::{
        config::{
            DEFAULT_CONFIG_FILE_JSON, DEFAULT_CONFIG_FILE_TOML, VpnServiceConfigExt,
            VpnServiceConfigVersion, geo_exclusion_settings, legacy,
        },
        error::{Error, GeoExclusionConfigError, Result},
        read_json_config_file, read_toml_config_file, write_json_config_file,
    },
    tunnel_state_machine::{
        DnsOptions, GatewayPerformanceOptions, MixnetTunnelOptions, TunnelSettings,
        WireguardMultihopMode, WireguardTunnelOptions,
    },
};

pub struct VpnServiceConfigManager {
    json_config_path: Option<PathBuf>,
    config: Box<nym_vpn_lib_types::VpnServiceConfig>,

    // Runtime-only Android app bypass (steering) configuration.
    //
    // Deliberately kept out of `config`: it is derived from platform state
    // (lockdown flag, resolved UIDs, the underlying network's DNS servers) that
    // is only valid for the lifetime of the process, and the platform layer
    // re-sends it on every connect. Persisting it would resurrect stale UIDs
    // and DNS servers on the next launch.
    #[cfg(target_os = "android")]
    app_bypass: Option<crate::tunnel_provider::AppBypassConfig>,

    // Used to send `ConfigChanged` events when the config is updated.
    // It's only optional to simplify testing.
    tunnel_event_tx: Option<broadcast::Sender<nym_vpn_lib_types::TunnelEvent>>,
}

impl VpnServiceConfigManager {
    /// Returns ephemeral config manager that does not persist the config on disk.
    pub fn new_ephermeral(
        initial_config: Box<nym_vpn_lib_types::VpnServiceConfig>,
        tunnel_event_tx: Option<broadcast::Sender<nym_vpn_lib_types::TunnelEvent>>,
    ) -> Self {
        Self {
            json_config_path: None,
            config: initial_config,
            #[cfg(target_os = "android")]
            app_bypass: None,
            tunnel_event_tx,
        }
    }

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
            json_config_path: Some(json_config_path),
            config: Box::new(config),
            #[cfg(target_os = "android")]
            app_bypass: None,
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
        if *self.config != config {
            *self.config = config;
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

    /// Set the Android app bypass (steering) configuration.
    ///
    /// Unlike the other setters this one neither persists to disk nor emits a
    /// `ConfigChanged` event: the value is runtime tunnel configuration that
    /// isn't part of `VpnServiceConfig`, and the platform layer re-sends it on
    /// every connect.
    #[cfg(target_os = "android")]
    pub fn set_app_bypass(&mut self, app_bypass: Option<crate::tunnel_provider::AppBypassConfig>) {
        if self.app_bypass != app_bypass {
            match app_bypass.as_ref() {
                Some(config) => tracing::info!(
                    "App bypass enabled for {} uid(s) with {} underlying dns server(s)",
                    config.excluded_uids.len(),
                    config.underlying_dns.len()
                ),
                None => tracing::info!("App bypass disabled"),
            }
            self.app_bypass = app_bypass;
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

    pub async fn set_enable_ad_blocking(&mut self, enable_ad_blocking: bool) {
        if self.config.enable_ad_blocking != enable_ad_blocking {
            self.config.enable_ad_blocking = enable_ad_blocking;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_fronting_mode(&mut self, fronting_mode: nym_vpn_lib_types::FrontingMode) {
        if self.config.fronting_mode != fronting_mode {
            // Change the shared fronting policy
            let front_policy = match fronting_mode {
                nym_vpn_lib_types::FrontingMode::Off => FrontPolicy::Off,
                nym_vpn_lib_types::FrontingMode::OnRetry => FrontPolicy::OnRetry,
                nym_vpn_lib_types::FrontingMode::Always => FrontPolicy::Always,
            };
            Client::set_shared_front_policy(front_policy);

            self.config.fronting_mode = fronting_mode;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_residential_exit(&mut self, residential_only: bool) {
        if self.config.residential_exit != residential_only {
            self.config.residential_exit = residential_only;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_enable_custom_dns(&mut self, enable_custom_dns: bool) -> bool {
        if self.config.enable_custom_dns == enable_custom_dns {
            false
        } else {
            self.config.enable_custom_dns = enable_custom_dns;
            self.save_config_and_send_event().await;
            true
        }
    }

    pub async fn set_custom_dns(&mut self, custom_dns: Vec<IpAddr>) -> bool {
        if self.config.custom_dns == custom_dns {
            false
        } else {
            self.config.custom_dns = custom_dns;
            self.save_config_and_send_event().await;
            true
        }
    }

    pub async fn set_mixnet_traffic_config(
        &mut self,
        mixnet_traffic: nym_vpn_lib_types::MixnetTrafficConfig,
    ) -> Result<(), MixnetTrafficConfigValidationError> {
        mixnet_traffic.validate()?;
        if self.config.mixnet_traffic != mixnet_traffic {
            self.config.mixnet_traffic = mixnet_traffic;
            self.save_config_and_send_event().await;
        }
        Ok(())
    }

    pub async fn set_enable_geo_location(
        &mut self,
        enable_geo_location: bool,
    ) -> Result<(), MixnetTrafficConfigValidationError> {
        if self
            .config
            .gateway_selection_algorithm_config
            .enable_geo_location
            != enable_geo_location
        {
            self.config
                .gateway_selection_algorithm_config
                .enable_geo_location = enable_geo_location;
            self.save_config_and_send_event().await;
        }
        Ok(())
    }

    pub async fn set_enable_gateway_independence(&mut self, enable_gateway_independence: bool) {
        if (enable_gateway_independence && !self.config.gateway_independence.full_enabled())
            || (!enable_gateway_independence && !self.config.gateway_independence.full_disabled())
        {
            self.config
                .gateway_independence
                .set_enabled(enable_gateway_independence);
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_enable_gateway_independence_notifications(
        &mut self,
        enable_notifications: bool,
    ) {
        if enable_notifications != self.config.gateway_independence.enable_notifications {
            self.config.gateway_independence.enable_notifications = enable_notifications;
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

    pub async fn set_netstats_allow_disconnected(&mut self, allow_disconnected: bool) {
        if self.config.network_stats.allow_disconnected != allow_disconnected {
            self.config.network_stats.allow_disconnected = allow_disconnected;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_netstats_enabled(&mut self, enabled: bool) {
        if self.config.network_stats.enabled != enabled {
            self.config.network_stats.enabled = enabled;
            self.save_config_and_send_event().await;
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn set_enable_split_tunnel(&mut self, enabled: bool) {
        if self.config.split_tunnel.enabled != enabled {
            self.config.split_tunnel.enabled = enabled;
            self.save_config_and_send_event().await;
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn add_split_tunnel_app(&mut self, app: SplitApp) {
        self.config.split_tunnel.add_app(app);
        self.save_config_and_send_event().await;
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn remove_split_tunnel_app(&mut self, app: SplitApp) {
        self.config.split_tunnel.remove_app(app);
        self.save_config_and_send_event().await;
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn clear_split_tunnel_apps(&mut self) {
        self.config.split_tunnel.clear_apps();
        self.save_config_and_send_event().await;
    }

    pub async fn set_geo_exclusion_enabled(&mut self, enabled: bool) {
        if self.config.geo_exclusion.enabled != enabled {
            self.config.geo_exclusion.enabled = enabled;
            self.save_config_and_send_event().await;
        }
    }

    pub async fn set_geo_exclusion_listen_port(
        &mut self,
        listen_port: u16,
    ) -> Result<(), GeoExclusionConfigError> {
        geo_exclusion_settings::v9::validate_listen_port(listen_port)?;

        if self.config.geo_exclusion.listen_port != listen_port {
            self.config.geo_exclusion.listen_port = listen_port;
            self.save_config_and_send_event().await;
        }

        Ok(())
    }

    pub async fn set_geo_exclusion_excluded_countries(
        &mut self,
        excluded_countries: Vec<String>,
    ) -> Result<(), GeoExclusionConfigError> {
        geo_exclusion_settings::v9::validate_excluded_countries(&excluded_countries)?;

        if self.config.geo_exclusion.excluded_countries != excluded_countries {
            self.config.geo_exclusion.excluded_countries = excluded_countries;
            self.save_config_and_send_event().await;
        }

        Ok(())
    }

    async fn save_config_and_send_event(&self) {
        // This function already logs
        let _ = self.write_to_file().await;

        // Notify all clients that the config has changed
        if let Some(tx) = self.tunnel_event_tx.as_ref() {
            match tx.send(nym_vpn_lib_types::TunnelEvent::ConfigChanged(
                self.config.clone(),
            )) {
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
        let Some(json_config_path) = self.json_config_path.as_ref() else {
            return true;
        };

        let ext_config =
            match VpnServiceConfigExt::try_from(&*self.config).map_err(Error::ConfigSetup) {
                Ok(ext_config) => ext_config,
                Err(e) => {
                    tracing::error!("Failed to convert service config to JSON: {e}");
                    return false;
                }
            };
        let version = ext_config.version();

        match write_json_config_file(json_config_path, &ext_config)
            .await
            .map_err(Error::ConfigSetup)
        {
            Ok(_) => {
                tracing::info!(
                    "Writing service config version {version} to {}",
                    json_config_path.display()
                );
                true
            }
            Err(e) => {
                trace_err_chain!(
                    e,
                    "failed to write service config version {version} to {}",
                    json_config_path.display()
                );
                false
            }
        }
    }

    pub fn generate_tunnel_settings(&self) -> TunnelSettings {
        tracing::info!("Using config: {:?}", self.config);

        let gateway_options = GatewayPerformanceOptions {
            mixnet_min_performance: self.config.mixnet_traffic.min_gateway_mixnet_performance,
            vpn_min_performance: self.config.min_gateway_vpn_performance,
        };

        let mixnet_client_config = MixnetClientConfig {
            disable_real_traffic_poisson_process: self.config.mixnet_traffic.disable_poisson_rate,
            disable_background_cover_traffic: self
                .config
                .mixnet_traffic
                .disable_background_cover_traffic,
            min_mixnode_performance: Some(
                self.config
                    .mixnet_traffic
                    .min_mixnode_performance
                    .unwrap_or(DEFAULT_MIN_MIXNODE_PERFORMANCE),
            ),
            min_gateway_performance: Some(
                self.config
                    .mixnet_traffic
                    .min_gateway_mixnet_performance
                    .unwrap_or(DEFAULT_MIN_GATEWAY_PERFORMANCE),
            ),
            loop_cover_traffic_average_delay: self
                .config
                .mixnet_traffic
                .poisson_parameter_for_loop_cover_stream
                .map(|ms| Duration::from_millis(ms.into())),

            average_packet_delay: self
                .config
                .mixnet_traffic
                .average_packet_delay
                .map(|ms| Duration::from_millis(ms.into())),

            message_sending_average_delay: self
                .config
                .mixnet_traffic
                .message_sending_average_delay
                .map(|ms| Duration::from_millis(ms.into())),
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

        let geo_exclusion_settings = self.config.geo_exclusion.clone();

        #[cfg(any(target_os = "macos", target_os = "windows"))]
        let split_tunnel = {
            let mut split_tunnel = self.config.split_tunnel.clone();
            // If geo exclusion is enabled then Split Tunneling also needs to be enabled.
            if geo_exclusion_settings.enabled && !self.config.split_tunnel.enabled {
                tracing::warn!("Enabling Split Tunnel as Geo Exclusion is enabled");
                split_tunnel.enabled = true;
            }
            split_tunnel
        };

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let split_tunnel = self.config.split_tunnel.clone();

        TunnelSettings {
            enable_ipv6: !self.config.disable_ipv6,
            allow_lan: self.config.allow_lan,
            enable_ad_blocking: self.config.enable_ad_blocking,
            residential_exit: self.config.residential_exit,
            tunnel_type,
            mixnet_tunnel_options: MixnetTunnelOptions { mtu: None },
            wireguard_tunnel_options: WireguardTunnelOptions {
                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                multihop_mode: if self.config.netstack {
                    WireguardMultihopMode::Netstack
                } else {
                    WireguardMultihopMode::TunTun
                },
                #[cfg(any(target_os = "android", target_os = "ios"))]
                multihop_mode: WireguardMultihopMode::Netstack,
                enable_bridges: self.config.enable_bridges,
            },
            gateway_performance_options: gateway_options,
            mixnet_client_config: Some(mixnet_client_config),
            entry_point: Box::new(self.config.entry_point.clone()),
            exit_point: Box::new(self.config.exit_point.clone()),
            dns,
            split_tunnel,
            geo_exclusion_settings,
            gateway_selection_algorithm_config: self
                .config
                .gateway_selection_algorithm_config
                .clone(),
            gateway_independence: self.config.gateway_independence,
            #[cfg(target_os = "android")]
            app_bypass: self.app_bypass.clone().map(Box::new),
        }
    }
}
