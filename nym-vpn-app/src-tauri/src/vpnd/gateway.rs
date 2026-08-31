use crate::country::Country;

use anyhow::{Result, anyhow};
use nym_vpn_lib_types as lib;
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{instrument, warn};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, strum::Display, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "kebab-case")]
pub enum GatewayType {
    MxEntry,
    MxExit,
    Wg,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, TS, Default)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "kebab-case")]
pub enum Score {
    #[default]
    Offline,
    Low,
    Medium,
    High,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, TS, Default)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "kebab-case")]
pub enum AsnType {
    #[default]
    Other,
    Residential,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct Asn {
    pub asn: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: AsnType,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub city: String,
    pub region: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct Performance {
    pub score: Score,
    pub load: Score,
    pub last_updated_utc: String,
    /// uptime percentage on the last 24 hours
    pub uptime_24h: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct Gateway {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: GatewayType,
    pub name: String,
    pub country: Country,
    pub location: Location,
    pub description: Option<String>,
    pub asn: Option<Asn>,
    pub mx_score: Score,
    pub wg_score: Score,
    pub wg_performance: Option<Performance>,
    pub exit_ipv4: Option<String>,
    pub exit_ipv6: Option<String>,
    pub build_version: Option<String>,
    pub quic: bool,
    pub node_family_name: Option<String>,
}

/// Convert daemon gateways into their app representation, dropping any that
/// cannot be parsed (a missing location makes a gateway unusable to the UI).
pub fn parse_gateways(gateways: Vec<lib::Gateway>, gw_type: GatewayType) -> Vec<Gateway> {
    gateways
        .into_iter()
        .filter_map(|gateway| {
            Gateway::from_lib(gateway, gw_type)
                .inspect_err(|e| warn!("failed to parse gateway from lib: {e}"))
                .ok()
        })
        .collect()
}

#[derive(Serialize, Deserialize, Clone, Debug, TS, Default)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct RecentGateways {
    pub entry: Vec<Gateway>,
    pub exit: Vec<Gateway>,
}

impl Gateway {
    #[instrument]
    pub fn from_lib(gateway: lib::Gateway, gw_type: GatewayType) -> Result<Self> {
        let Some(location) = gateway.location else {
            warn!("missing gateway location in vpnd Gateway");
            return Err(anyhow!("missing gateway location in vpnd Gateway"));
        };

        let mx_score = gateway
            .performance
            .as_ref()
            .map(|perf| perf.mixnet_score.into())
            .unwrap_or(Score::Offline);

        let wg_score = gateway
            .performance
            .as_ref()
            .map(|perf| perf.score.into())
            .unwrap_or(Score::Offline);

        let quic = gateway
            .bridge_params
            .as_ref()
            .map(|info| {
                info.transports
                    .iter()
                    .any(|p| matches!(p, lib::BridgeParameters::QuicPlain(_)))
            })
            .unwrap_or(false);

        let asn = location.asn.clone().map(|a| a.into());
        let exit_ipv4 = gateway.exit_ipv4s.first().map(|addr| addr.to_string());
        let exit_ipv6 = gateway.exit_ipv6s.first().map(|addr| addr.to_string());

        Ok(Self {
            id: gateway.identity_key,
            kind: gw_type,
            name: gateway.name,
            country: Country::try_from(&location)?,
            location: location.into(),
            asn,
            mx_score,
            wg_score,
            wg_performance: gateway.performance.map(|p| p.into()),
            description: gateway.description,
            exit_ipv4,
            exit_ipv6,
            build_version: gateway.build_version,
            quic,
            node_family_name: gateway.node_family_name,
        })
    }
}

impl From<lib::Score> for Score {
    fn from(score: lib::Score) -> Self {
        match score {
            lib::Score::Offline => Score::Offline,
            lib::Score::Low => Score::Low,
            lib::Score::Medium => Score::Medium,
            lib::Score::High => Score::High,
        }
    }
}

impl From<lib::GatewayType> for GatewayType {
    fn from(gw_type: lib::GatewayType) -> Self {
        match gw_type {
            lib::GatewayType::MixnetEntry => GatewayType::MxEntry,
            lib::GatewayType::MixnetExit => GatewayType::MxExit,
            lib::GatewayType::Wg => GatewayType::Wg,
        }
    }
}

impl From<GatewayType> for lib::GatewayType {
    fn from(gw_type: GatewayType) -> Self {
        match gw_type {
            GatewayType::MxEntry => lib::GatewayType::MixnetEntry,
            GatewayType::MxExit => lib::GatewayType::MixnetExit,
            GatewayType::Wg => lib::GatewayType::Wg,
        }
    }
}

impl From<lib::Location> for Location {
    fn from(l: lib::Location) -> Self {
        Location {
            latitude: l.latitude,
            longitude: l.longitude,
            city: l.city,
            region: l.region,
        }
    }
}

impl TryFrom<&lib::Location> for Country {
    type Error = anyhow::Error;

    fn try_from(location: &lib::Location) -> Result<Country, Self::Error> {
        Country::try_new_from_code(&location.two_letter_iso_country_code).ok_or_else(|| {
            let msg = format!(
                "invalid country code {}",
                location.two_letter_iso_country_code
            );
            warn!(msg);
            anyhow!(msg)
        })
    }
}

impl From<lib::Asn> for Asn {
    fn from(asn: lib::Asn) -> Self {
        Asn {
            asn: asn.asn,
            name: asn.name,
            kind: match asn.kind {
                lib::AsnKind::Residential => AsnType::Residential,
                lib::AsnKind::Other => AsnType::Other,
            },
        }
    }
}

impl From<lib::Performance> for Performance {
    fn from(perf: lib::Performance) -> Self {
        Performance {
            score: Score::from(perf.score),
            load: Score::from(perf.load),
            last_updated_utc: perf.last_updated_utc,
            uptime_24h: perf.uptime_percentage_last_24_hours,
        }
    }
}

impl fmt::Display for Gateway {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] ({}) {}", self.id, self.name, self.country)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct GatewaySelectionAlgorithmConfig {
    pub enable_geo_location: bool,
}

impl From<lib::GatewaySelectionAlgorithmConfig> for GatewaySelectionAlgorithmConfig {
    fn from(config: lib::GatewaySelectionAlgorithmConfig) -> Self {
        GatewaySelectionAlgorithmConfig {
            enable_geo_location: config.enable_geo_location,
        }
    }
}
