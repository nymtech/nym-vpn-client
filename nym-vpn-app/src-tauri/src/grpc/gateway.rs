use crate::country::Country;

use anyhow::{Result, anyhow};
use nym_vpn_proto::proto as p;
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{error, instrument, warn};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, strum::Display, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum GatewayType {
    MxEntry,
    MxExit,
    Wg,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, TS, Default)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum Score {
    #[default]
    Offline,
    Low,
    Medium,
    High,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, TS, Default)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum AsnType {
    #[default]
    Other,
    Residential,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Asn {
    pub asn: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: AsnType,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub city: String,
    pub region: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Performance {
    pub score: Score,
    pub load: Score,
    pub last_updated_utc: String,
    /// uptime percentage on the last 24 hours
    pub uptime_24h: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Gateway {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: GatewayType,
    pub name: String,
    pub country: Country,
    pub location: Location,
    pub asn: Option<Asn>,
    pub mx_score: Score,
    pub wg_score: Score,
    pub wg_performance: Option<Performance>,
    pub exit_ipv4: Option<String>,
    pub exit_ipv6: Option<String>,
    pub build_version: Option<String>,
}

impl Gateway {
    #[instrument]
    pub fn from_proto(gateway: p::GatewayResponse, gw_type: GatewayType) -> Result<Self> {
        let Some(id) = gateway.id else {
            warn!("missing gateway ID in GatewayResponse");
            return Err(anyhow!("missing gateway ID in GatewayResponse"));
        };
        let Some(location) = gateway.location else {
            warn!("missing gateway location in GatewayResponse");
            return Err(anyhow!("missing gateway location in GatewayResponse"));
        };

        let mx_score = gateway
            .mixnet_score
            .map(|s| {
                p::Score::try_from(s)
                    .inspect_err(|e| error!("failed to parse proto gw mixnet score: {}", e))
            })
            .transpose()?
            .unwrap_or(p::Score::Offline);

        let wg_score = gateway
            .wg_performance
            .as_ref()
            .map(|s| {
                p::Score::try_from(s.score)
                    .inspect_err(|e| error!("failed to parse proto gw wireguard score: {}", e))
            })
            .transpose()?
            .unwrap_or(p::Score::Offline);

        let asn = location.asn.clone().map(|a| a.into());
        let exit_ipv4 = gateway.exit_ipv4s.first().cloned();
        let exit_ipv6 = gateway.exit_ipv6s.first().cloned();

        Ok(Self {
            id: id.id,
            kind: gw_type,
            name: gateway.moniker,
            country: Country::try_from(&location)?,
            location: location.into(),
            asn,
            mx_score: Score::from(mx_score),
            wg_score: Score::from(wg_score),
            wg_performance: gateway.wg_performance.map(|p| p.into()),
            exit_ipv4,
            exit_ipv6,
            build_version: gateway.build_version,
        })
    }
}

impl Score {
    fn from(score: p::Score) -> Self {
        match score {
            p::Score::Offline => Score::Offline,
            p::Score::Low => Score::Low,
            p::Score::Medium => Score::Medium,
            p::Score::High => Score::High,
        }
    }
}

impl From<p::GatewayType> for GatewayType {
    fn from(gw_type: p::GatewayType) -> Self {
        match gw_type {
            p::GatewayType::MixnetEntry => GatewayType::MxEntry,
            p::GatewayType::MixnetExit => GatewayType::MxExit,
            p::GatewayType::Wg => GatewayType::Wg,
        }
    }
}

impl From<GatewayType> for p::GatewayType {
    fn from(gw_type: GatewayType) -> Self {
        match gw_type {
            GatewayType::MxEntry => p::GatewayType::MixnetEntry,
            GatewayType::MxExit => p::GatewayType::MixnetExit,
            GatewayType::Wg => p::GatewayType::Wg,
        }
    }
}

impl From<p::Location> for Location {
    fn from(proto: p::Location) -> Self {
        Location {
            latitude: proto.latitude,
            longitude: proto.longitude,
            city: proto.city,
            region: proto.region,
        }
    }
}

impl From<p::Asn> for Asn {
    fn from(proto: p::Asn) -> Self {
        let asn_kind = &proto.kind();
        Asn {
            asn: proto.asn,
            name: proto.name,
            kind: match asn_kind {
                p::AsnKind::Residential => AsnType::Residential,
                p::AsnKind::Other => AsnType::Other,
            },
        }
    }
}

impl From<p::Performance> for Performance {
    fn from(proto: p::Performance) -> Self {
        Performance {
            score: Score::from(proto.score()),
            load: Score::from(proto.load()),
            last_updated_utc: proto.last_updated_utc,
            uptime_24h: proto.uptime_percentage_last_24_hours,
        }
    }
}

impl fmt::Display for Gateway {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] ({}) {}", self.id, self.name, self.country)
    }
}
