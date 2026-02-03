// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, net::IpAddr, ops::RangeInclusive};

const LOOP_COVER_DELAY_RANGE: RangeInclusive<u32> = 0..=200;
const AVG_PACKET_DELAY_RANGE: RangeInclusive<u32> = 0..=200;
const MESSAGE_SENDING_DELAY_RANGE: RangeInclusive<u32> = 5..=50;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

use crate::{EntryPoint, ExitPoint, NetworkStatisticsConfig, NymNetworkDetails, NymVpnNetwork};

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
pub struct VpnServiceConfig {
    pub entry_point: EntryPoint,
    pub exit_point: ExitPoint,
    pub allow_lan: bool,
    pub disable_ipv6: bool,
    pub enable_two_hop: bool,
    pub enable_bridges: bool,
    pub enable_lewes_protocol: bool,
    pub netstack: bool,
    pub min_gateway_vpn_performance: Option<u8>,
    pub residential_exit: bool,
    pub enable_custom_dns: bool,
    pub custom_dns: Vec<IpAddr>,
    pub mixnet_traffic: MixnetTrafficConfig,
    pub network_stats: NetworkStatisticsConfig,
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
            "allow_lan: {}, disable_ipv6: {}, enable_two_hop: {}, enable_lewes_protocol: {}, netstack: {}",
            self.allow_lan,
            self.disable_ipv6,
            self.enable_two_hop,
            self.enable_lewes_protocol,
            self.netstack
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

        Ok(())
    }
}

impl Default for VpnServiceConfig {
    fn default() -> Self {
        Self {
            entry_point: EntryPoint::Country {
                two_letter_iso_country_code: "CH".to_owned(),
            },
            exit_point: ExitPoint::Country {
                two_letter_iso_country_code: "CH".to_owned(),
            },
            allow_lan: false,
            disable_ipv6: false,
            enable_two_hop: true,
            enable_bridges: false,
            enable_lewes_protocol: false,
            netstack: false,
            min_gateway_vpn_performance: None,
            residential_exit: false,
            enable_custom_dns: false,
            custom_dns: vec![],
            network_stats: Default::default(),
            mixnet_traffic: MixnetTrafficConfig::default(),
        }
    }
}

#[cfg(feature = "uniffi-bindings")]
pub type BoxedVpnServiceConfig = Box<VpnServiceConfig>;
#[cfg(feature = "uniffi-bindings")]
uniffi::custom_type!(BoxedVpnServiceConfig, VpnServiceConfig, {
    remote,
    try_lift: |val| Ok(Box::new(val)),
    lower: |val| *val
});

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct MixnetTrafficConfig {
    pub poisson_parameter_for_loop_cover_stream: Option<u32>,
    pub average_packet_delay: Option<u32>,
    pub message_sending_average_delay: Option<u32>,

    pub disable_poisson_rate: bool,
    pub disable_background_cover_traffic: bool,

    pub min_mixnode_performance: Option<u8>,
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
        writeln!(
            f,
            "min_mixnode_performance: {:?}, min_gateway_mixnet_performance: {:?}",
            self.min_mixnode_performance, self.min_gateway_mixnet_performance
        )?;
        Ok(())
    }
}

impl MixnetTrafficConfig {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(v) = self.poisson_parameter_for_loop_cover_stream
            && !LOOP_COVER_DELAY_RANGE.contains(&v)
        {
            return Err(format!(
                "poisson_parameter_for_loop_cover_stream must be between {} and {} ms (got {})",
                LOOP_COVER_DELAY_RANGE.start(),
                LOOP_COVER_DELAY_RANGE.end(),
                v
            ));
        }

        if let Some(v) = self.average_packet_delay
            && !AVG_PACKET_DELAY_RANGE.contains(&v)
        {
            return Err(format!(
                "average_packet_delay must be between {} and {} ms (got {})",
                AVG_PACKET_DELAY_RANGE.start(),
                AVG_PACKET_DELAY_RANGE.end(),
                v
            ));
        }

        if let Some(v) = self.message_sending_average_delay
            && !MESSAGE_SENDING_DELAY_RANGE.contains(&v)
        {
            return Err(format!(
                "message_sending_average_delay must be between {} and {} ms (got {})",
                MESSAGE_SENDING_DELAY_RANGE.start(),
                MESSAGE_SENDING_DELAY_RANGE.end(),
                v
            ));
        }

        Ok(())
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
pub enum TargetState {
    /// Unsecure the device.
    Unsecured,

    /// Secure the device.
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
pub struct VpnServiceInfo {
    pub version: String,
    #[cfg_attr(feature = "typescript-bindings", ts(as = "String"))]
    #[cfg_attr(feature = "serde", serde(with = "time::serde::iso8601::option"))]
    pub build_timestamp: Option<OffsetDateTime>,
    pub triple: String,
    pub platform: String,
    pub git_commit: String,
    pub nym_network: NymNetworkDetails,
    pub nym_vpn_network: NymVpnNetwork,
}
