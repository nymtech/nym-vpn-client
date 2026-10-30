// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashSet, fmt, net::IpAddr, ops::RangeInclusive, path::PathBuf};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const LOOP_COVER_DELAY_RANGE: RangeInclusive<u32> = 0..=200;
const AVG_PACKET_DELAY_RANGE: RangeInclusive<u32> = 0..=200;
const MESSAGE_SENDING_DELAY_RANGE: RangeInclusive<u32> = 5..=50;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

use crate::{
    EntryPoint, ExitPoint, GatewayIndependence, GatewaySelectionAlgorithmConfig,
    NetworkStatisticsConfig, NymNetworkDetails, NymVpnNetwork,
};

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "uniffi-bindings",
    derive(uniffi::Record),
    uniffi::export(Display, Eq)
)]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct VpnServiceConfig {
    #[cfg_attr(feature = "cbor", n(0))]
    pub entry_point: EntryPoint,
    #[cfg_attr(feature = "cbor", n(1))]
    pub exit_point: ExitPoint,
    #[cfg_attr(feature = "cbor", n(2))]
    pub allow_lan: bool,
    #[cfg_attr(feature = "cbor", n(3))]
    pub disable_ipv6: bool,
    #[cfg_attr(feature = "cbor", n(4))]
    pub enable_two_hop: bool,
    #[cfg_attr(feature = "cbor", n(5))]
    pub bridge_mode: BridgeMode,
    #[cfg_attr(feature = "cbor", n(6))]
    pub enable_ad_blocking: bool,
    #[cfg_attr(feature = "cbor", n(7))]
    pub enable_conflict_detection: bool,
    #[cfg_attr(feature = "cbor", n(8))]
    pub fronting_mode: FrontingMode,
    #[cfg_attr(feature = "cbor", n(9))]
    pub netstack: bool,
    #[cfg_attr(feature = "cbor", n(10))]
    pub min_gateway_vpn_performance: Option<u8>,
    #[cfg_attr(feature = "cbor", n(11))]
    pub residential_exit: bool,
    #[cfg_attr(feature = "cbor", n(12))]
    pub enable_custom_dns: bool,
    #[cfg_attr(feature = "cbor", n(13))]
    pub custom_dns: Vec<IpAddr>,
    #[cfg_attr(feature = "cbor", n(14))]
    pub mixnet_traffic: MixnetTrafficConfig,
    #[cfg_attr(feature = "cbor", n(15))]
    pub network_stats: NetworkStatisticsConfig,
    #[cfg_attr(feature = "cbor", n(16))]
    pub split_tunnel: SplitTunnelSettings,
    #[cfg_attr(feature = "cbor", n(17))]
    pub geo_exclusion: GeoExclusionSettings,
    #[cfg_attr(feature = "cbor", n(18))]
    pub gateway_selection_algorithm_config: GatewaySelectionAlgorithmConfig,
    #[cfg_attr(feature = "cbor", n(19))]
    pub gateway_independence: GatewayIndependence,
}

impl fmt::Display for VpnServiceConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "entry point: {:?}, exit point: {:?}",
            self.entry_point, self.exit_point,
        )?;
        writeln!(
            f,
            "allow_lan: {}, disable_ipv6: {}, enable_two_hop: {},",
            self.allow_lan, self.disable_ipv6, self.enable_two_hop,
        )?;
        writeln!(
            f,
            "enable_ad_blocking: {}, enable_conflict_detection: {}, netstack: {}",
            self.enable_ad_blocking, self.enable_conflict_detection, self.netstack
        )?;

        writeln!(
            f,
            "min_gateway_vpn_performance: {:?}",
            self.min_gateway_vpn_performance
        )?;
        writeln!(f, "residential_exit: {}", self.residential_exit)?;
        writeln!(
            f,
            "enable_custom_dns: {}, custom_dns: {}",
            self.enable_custom_dns,
            self.custom_dns
                .iter()
                .map(|ip| ip.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        writeln!(f, "mixnet traffic config: {}", self.mixnet_traffic)?;
        writeln!(f, "networks stats config: {}", self.network_stats)?;
        writeln!(f, "split tunnel settings: {}", self.split_tunnel)?;
        writeln!(f, "geo exclusion settings: {}", self.geo_exclusion)?;
        writeln!(
            f,
            "gateway selection algorithm: {}",
            self.gateway_selection_algorithm_config
        )?;
        writeln!(f, "gateway independence: {}", self.gateway_independence)?;

        Ok(())
    }
}

impl Default for VpnServiceConfig {
    fn default() -> Self {
        Self {
            entry_point: EntryPoint::Auto {
                exclude_user_country: true,
            },
            exit_point: ExitPoint::Auto {
                exclude_entry_point_country: true,
                exclude_user_country: true,
            },
            allow_lan: false,
            disable_ipv6: false,
            enable_two_hop: true,
            bridge_mode: BridgeMode::default(),
            enable_ad_blocking: false,
            enable_conflict_detection: true,
            fronting_mode: FrontingMode::default(),
            netstack: false,
            min_gateway_vpn_performance: None,
            residential_exit: false,
            enable_custom_dns: false,
            custom_dns: vec![],
            network_stats: Default::default(),
            mixnet_traffic: MixnetTrafficConfig::default(),
            split_tunnel: SplitTunnelSettings::default(),
            geo_exclusion: GeoExclusionSettings::default(),
            gateway_selection_algorithm_config: Default::default(),
            gateway_independence: Default::default(),
        }
    }
}

/// Returns the default `VpnServiceConfig`.
#[cfg(feature = "uniffi-bindings")]
pub fn default_vpn_service_config() -> VpnServiceConfig {
    VpnServiceConfig::default()
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(
    feature = "uniffi-bindings",
    derive(uniffi::Record),
    uniffi::export(Display)
)]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct MixnetTrafficConfig {
    #[cfg_attr(feature = "cbor", n(0))]
    pub poisson_parameter_for_loop_cover_stream: Option<u32>,
    #[cfg_attr(feature = "cbor", n(1))]
    pub average_packet_delay: Option<u32>,
    #[cfg_attr(feature = "cbor", n(2))]
    pub message_sending_average_delay: Option<u32>,

    #[cfg_attr(feature = "cbor", n(3))]
    pub disable_poisson_rate: bool,
    #[cfg_attr(feature = "cbor", n(4))]
    pub disable_background_cover_traffic: bool,

    #[cfg_attr(feature = "cbor", n(5))]
    pub min_mixnode_performance: Option<u8>,
    #[cfg_attr(feature = "cbor", n(6))]
    pub min_gateway_mixnet_performance: Option<u8>,
}

impl fmt::Display for MixnetTrafficConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "poisson_parameter_for_loop_cover_stream: {:?}, average_packet_delay: {:?}, message_sending_average_delay: {:?}",
            self.poisson_parameter_for_loop_cover_stream,
            self.average_packet_delay,
            self.message_sending_average_delay,
        )?;
        writeln!(
            f,
            "disable_poisson_rate: {}, disable_background_cover_traffic: {}",
            self.disable_poisson_rate, self.disable_background_cover_traffic
        )?;
        write!(
            f,
            "min_mixnode_performance: {:?}, min_gateway_mixnet_performance: {:?}",
            self.min_mixnode_performance, self.min_gateway_mixnet_performance
        )?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Error))]
pub enum MixnetTrafficConfigValidationError {
    #[error(
        "poisson_parameter_for_loop_cover_stream must be between {start} and {end} ms (got {actual})"
    )]
    InvalidPoissonParameterForLoopCoverStream { start: u32, end: u32, actual: u32 },

    #[error("average_packet_delay must be between {start} and {end} ms (got {actual})")]
    InvalidAveragePacketDelay { start: u32, end: u32, actual: u32 },

    #[error("message_sending_average_delay must be between {start} and {end} ms (got {actual})")]
    InvalidMessageSendingAverageDelay { start: u32, end: u32, actual: u32 },
}

#[cfg_attr(feature = "uniffi-bindings", uniffi::export)]
impl MixnetTrafficConfig {
    pub fn validate(&self) -> Result<(), MixnetTrafficConfigValidationError> {
        if let Some(v) = self.poisson_parameter_for_loop_cover_stream
            && !LOOP_COVER_DELAY_RANGE.contains(&v)
        {
            Err(
                MixnetTrafficConfigValidationError::InvalidPoissonParameterForLoopCoverStream {
                    start: *LOOP_COVER_DELAY_RANGE.start(),
                    end: *LOOP_COVER_DELAY_RANGE.end(),
                    actual: v,
                },
            )
        } else if let Some(v) = self.average_packet_delay
            && !AVG_PACKET_DELAY_RANGE.contains(&v)
        {
            Err(
                MixnetTrafficConfigValidationError::InvalidAveragePacketDelay {
                    start: *AVG_PACKET_DELAY_RANGE.start(),
                    end: *AVG_PACKET_DELAY_RANGE.end(),
                    actual: v,
                },
            )
        } else if let Some(v) = self.message_sending_average_delay
            && !MESSAGE_SENDING_DELAY_RANGE.contains(&v)
        {
            Err(
                MixnetTrafficConfigValidationError::InvalidMessageSendingAverageDelay {
                    start: *MESSAGE_SENDING_DELAY_RANGE.start(),
                    end: *MESSAGE_SENDING_DELAY_RANGE.end(),
                    actual: v,
                },
            )
        } else {
            Ok(())
        }
    }

    /// Calculate the expected round-trip latency (RTT) in milliseconds based on the current
    /// configuration parameters.
    ///
    /// Formula: RTT = 2 × (6 × 50ms + 3 × mixing_delay)
    /// - 6 × 50ms = base latency from 6 hops at 50ms each
    /// - 3 × mixing_delay = additional delay from mixing at 3 mix nodes
    /// - 2× for round-trip
    ///
    /// Example: 15ms mixing delay → 2 × (300 + 45) = 690ms
    pub fn calculate_traffic_latency(&self) -> f64 {
        const BASE_HOP_DELAY_MS: f64 = 50.0;
        const NUM_HOPS: f64 = 6.0;
        const NUM_MIX_NODES: f64 = 3.0;

        let mixing_delay = self.average_packet_delay.unwrap_or(0) as f64;
        let latency = 2.0 * (NUM_HOPS * BASE_HOP_DELAY_MS + NUM_MIX_NODES * mixing_delay);

        (latency / 10.0).round() * 10.0
    }
}

/// Type providing default values and constraints for mixnet traffic configuration.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Object))]
pub struct MixnetTrafficDefaults;

#[allow(unused)]
#[cfg_attr(feature = "uniffi-bindings", uniffi::export)]
impl MixnetTrafficDefaults {
    #[cfg(feature = "uniffi-bindings")]
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn default_mixing_delay(&self) -> MixingDelay {
        MixingDelay::default()
    }

    pub fn default_disable_poission_rate(&self) -> bool {
        false
    }

    pub fn default_background_traffic(&self) -> BackgroundCoverTrafficRate {
        BackgroundCoverTrafficRate::default()
    }

    pub fn default_continuous_traffic(&self) -> ContinuousTrafficSendingRate {
        ContinuousTrafficSendingRate::default()
    }

    pub fn all_background_traffic(&self) -> Vec<BackgroundCoverTrafficRate> {
        BackgroundCoverTrafficRate::iter().collect()
    }

    pub fn all_continuous_traffic(&self) -> Vec<ContinuousTrafficSendingRate> {
        ContinuousTrafficSendingRate::iter().collect()
    }
}

// Maps to average_packet_delay
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct MixingDelay {
    #[cfg_attr(feature = "cbor", n(0))]
    pub min_value: u32,
    #[cfg_attr(feature = "cbor", n(1))]
    pub max_value: u32,
    #[cfg_attr(feature = "cbor", n(2))]
    pub default_value: u32,
}

impl Default for MixingDelay {
    fn default() -> Self {
        Self {
            min_value: *AVG_PACKET_DELAY_RANGE.start(),
            max_value: *AVG_PACKET_DELAY_RANGE.end(),
            default_value: 15, // DEFAULT_AVERAGE_PACKET_DELAY
        }
    }
}

// Maps to poisson_parameter_for_loop_cover_stream
#[derive(Clone, Debug, Default, Eq, PartialEq, EnumIter)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
#[cfg_attr(feature = "cbor", cbor(index_only))]
pub enum BackgroundCoverTrafficRate {
    #[default]
    #[cfg_attr(feature = "cbor", n(0))]
    Ms200, // DEFAULT_LOOP_COVER_STREAM_AVERAGE_DELAY
    #[cfg_attr(feature = "cbor", n(1))]
    Ms40,
    #[cfg_attr(feature = "cbor", n(2))]
    Ms20,
    #[cfg_attr(feature = "cbor", n(3))]
    Ms10,
}

#[allow(unused)]
#[cfg_attr(feature = "uniffi-bindings", uniffi::export)]
impl BackgroundCoverTrafficRate {
    pub fn value(&self) -> u32 {
        match self {
            Self::Ms10 => 10,
            Self::Ms20 => 20,
            Self::Ms40 => 40,
            Self::Ms200 => 200,
        }
    }

    pub fn multiplier(&self) -> String {
        match self {
            Self::Ms200 => "Base",
            Self::Ms40 => "5x",
            Self::Ms20 => "10x",
            Self::Ms10 => "20x",
        }
        .to_owned()
    }
}

// Maps to message_sending_average_delay
#[derive(Clone, Debug, Default, Eq, PartialEq, EnumIter)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
#[cfg_attr(feature = "cbor", cbor(index_only))]
pub enum ContinuousTrafficSendingRate {
    #[cfg_attr(feature = "cbor", n(0))]
    Ms30,
    #[default]
    #[cfg_attr(feature = "cbor", n(1))]
    Ms20,
    #[cfg_attr(feature = "cbor", n(2))]
    Ms10,
}

#[allow(unused)]
#[cfg_attr(feature = "uniffi-bindings", uniffi::export)]
impl ContinuousTrafficSendingRate {
    pub fn value(&self) -> u32 {
        match self {
            Self::Ms10 => 10,
            Self::Ms20 => 20,
            Self::Ms30 => 30,
        }
    }

    pub fn throughput(&self) -> String {
        match self {
            Self::Ms10 => "2 Mbps",
            Self::Ms20 => "1 Mbps",
            Self::Ms30 => "0.7 Mbps",
        }
        .to_owned()
    }
}

/// How the WireGuard tunnel uses bridges (QUIC).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
#[cfg_attr(feature = "cbor", cbor(index_only))]
pub enum BridgeMode {
    /// Plain WireGuard first, then bridges when an attempt fails.
    #[cfg_attr(feature = "cbor", n(0))]
    Auto,
    /// Bridges only.
    #[cfg_attr(feature = "cbor", n(1))]
    On,
    /// Plain WireGuard only.
    #[default]
    #[cfg_attr(feature = "cbor", n(2))]
    Off,
}

impl From<bool> for BridgeMode {
    fn from(enable_bridges: bool) -> Self {
        if enable_bridges { Self::On } else { Self::Off }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
#[cfg_attr(feature = "cbor", cbor(index_only))]
pub enum FrontingMode {
    #[cfg_attr(feature = "cbor", n(0))]
    Off,
    #[default]
    #[cfg_attr(feature = "cbor", n(1))]
    OnRetry,
    #[cfg_attr(feature = "cbor", n(2))]
    Always,
}

/// Single application participating in split tunneling.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct SplitApp {
    /// Path to executable
    #[cfg_attr(feature = "typescript-bindings", ts(as = "String"))]
    #[cfg_attr(feature = "cbor", n(0))]
    pub path: String,
}

impl SplitApp {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn path_buf(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }
}

impl fmt::Display for SplitApp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct SplitTunnelSettings {
    /// Whether split tunneling is enabled.
    #[cfg_attr(feature = "cbor", n(0))]
    pub enabled: bool,

    /// Applications participating in split tunneling.
    #[cfg_attr(feature = "cbor", n(1))]
    pub apps: Vec<SplitApp>,
}

impl SplitTunnelSettings {
    pub fn add_app(&mut self, app: SplitApp) {
        if !self.apps.iter().any(|v| v.path == app.path) {
            self.apps.push(app);
        }
    }

    pub fn remove_app(&mut self, app: SplitApp) {
        self.apps.retain(|v| v.path != app.path);
    }

    pub fn clear_apps(&mut self) {
        self.apps.clear();
    }

    /// Returns the effective list of applications participating in split tunneling when split tunneling is enabled.
    /// Otherwise, returns an empty slice.
    pub fn effective_apps(&self) -> &[SplitApp] {
        if self.enabled { &self.apps } else { &[] }
    }

    /// Returns effective list of application paths participating in split tunneling when split tunneling is enabled.
    /// Otherwise, returns an empty set.
    pub fn effective_app_paths(&self) -> HashSet<PathBuf> {
        HashSet::from_iter(self.effective_apps().iter().map(|v| v.path_buf()))
    }
}

impl fmt::Display for SplitTunnelSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "enabled: {}", self.enabled)?;
        writeln!(f, "apps:")?;
        for app in self.apps.iter() {
            writeln!(f, "- {app}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct GeoExclusionSettings {
    #[cfg_attr(feature = "cbor", n(0))]
    pub enabled: bool,
    #[cfg_attr(feature = "cbor", n(1))]
    pub listen_port: u16,
    #[cfg_attr(feature = "cbor", n(2))]
    pub excluded_countries: Vec<String>,
}

impl fmt::Display for GeoExclusionSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "enabled: {}", self.enabled)?;
        writeln!(f, "listen_port: {}", self.listen_port)?;
        writeln!(
            f,
            "excluded_countries: {}",
            self.excluded_countries.join(", ")
        )?;
        Ok(())
    }
}

impl Default for GeoExclusionSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            listen_port: 1081,
            excluded_countries: vec!["CN".to_string()],
        }
    }
}

/// The target tunnel state.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
#[cfg_attr(feature = "cbor", cbor(index_only))]
pub enum TargetState {
    /// Unsecure the device.
    #[cfg_attr(feature = "cbor", n(0))]
    Unsecured,

    /// Secure the device.
    #[cfg_attr(feature = "cbor", n(1))]
    Secured,
}

impl fmt::Display for TargetState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TargetState::Unsecured => "Unsecured",
            TargetState::Secured => "Secured",
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct VpnServiceInfo {
    #[cfg_attr(feature = "cbor", n(0))]
    pub version: String,
    #[cfg_attr(feature = "typescript-bindings", ts(as = "String"))]
    #[cfg_attr(feature = "serde", serde(with = "time::serde::iso8601::option"))]
    #[cfg_attr(feature = "cbor", n(1))]
    #[cfg_attr(
        feature = "cbor",
        cbor(with = "crate::cbor::option_offset_date_time", has_nil)
    )]
    pub build_timestamp: Option<OffsetDateTime>,
    #[cfg_attr(feature = "cbor", n(2))]
    pub triple: String,
    #[cfg_attr(feature = "cbor", n(3))]
    pub platform: String,
    #[cfg_attr(feature = "cbor", n(4))]
    pub git_commit: String,
    #[cfg_attr(feature = "cbor", n(5))]
    pub nym_network: NymNetworkDetails,
    #[cfg_attr(feature = "cbor", n(6))]
    pub nym_vpn_network: NymVpnNetwork,
}
