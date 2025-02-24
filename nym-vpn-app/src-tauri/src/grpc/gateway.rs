use crate::country::Country;
use anyhow::{anyhow, Result};
use nym_vpn_proto as p;
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
    None,
    Low,
    Medium,
    High,
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
    pub mx_score: Score,
    pub wg_score: Score,
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
            .unwrap_or(p::Score::None);

        let wg_score = gateway
            .wg_score
            .map(|s| {
                p::Score::try_from(s)
                    .inspect_err(|e| error!("failed to parse proto gw wireguard score: {}", e))
            })
            .transpose()?
            .unwrap_or(p::Score::None);

        Ok(Self {
            id: id.id,
            kind: gw_type,
            name: gateway.moniker,
            country: Country::try_from(&location)?,
            mx_score: Score::from(mx_score),
            wg_score: Score::from(wg_score),
        })
    }
}

impl Score {
    fn from(score: p::Score) -> Self {
        match score {
            p::Score::None => Score::None,
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
            // this should never happen
            p::GatewayType::Unspecified => panic!("unspecified gateway type"),
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

impl fmt::Display for Gateway {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] ({}) {}", self.id, self.name, self.country)
    }
}
