use std::fmt;

use anyhow::anyhow;
use futures::channel::mpsc::UnboundedSender;
use nym_vpn_lib::NymVpnCtrlMessage;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use ts_rs::TS;

use crate::{
    country::{Country, DEFAULT_COUNTRY_CODE},
    fs::{config::AppConfig, data::AppData},
};

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export)]
pub enum ConnectionState {
    Connected,
    #[default]
    Disconnected,
    Connecting,
    Disconnecting,
    Unknown,
}

#[derive(Default, Debug, Serialize, Deserialize, TS, Clone, PartialEq, Eq)]
#[ts(export)]
pub enum VpnMode {
    Mixnet,
    #[default]
    TwoHop,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TunnelConfig {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub state: ConnectionState,
    pub error: Option<String>,
    pub vpn_mode: VpnMode,
    pub entry_node_location: NodeLocation,
    pub exit_node_location: NodeLocation,
    pub tunnel: Option<TunnelConfig>,
    pub connection_start_time: Option<OffsetDateTime>,
    pub vpn_ctrl_tx: Option<UnboundedSender<NymVpnCtrlMessage>>,
}

impl TryFrom<(&AppData, &AppConfig)> for AppState {
    type Error = anyhow::Error;

    fn try_from(saved_data: (&AppData, &AppConfig)) -> Result<Self, Self::Error> {
        // retrieve default entry and exit node locations set from
        // the config file
        let default_entry_node_location = Country::try_from(
            saved_data
                .1
                .default_entry_node_location_code
                .as_deref()
                .unwrap_or(DEFAULT_COUNTRY_CODE),
        )
        .map_err(|e| anyhow!("failed to retrieve default entry node location: {e}"))?;

        let default_exit_node_location = Country::try_from(
            saved_data
                .1
                .default_exit_node_location_code
                .as_deref()
                .unwrap_or(DEFAULT_COUNTRY_CODE),
        )
        .map_err(|e| anyhow!("failed to retrieve default exit node location: {e}"))?;

        // restore any state from the saved app data (previous user session)
        // fallback to config file for locations if not present
        Ok(AppState {
            entry_node_location: saved_data.0.entry_node_location.clone().unwrap_or_else(|| {
                #[cfg(not(feature = "fastest-location"))]
                return NodeLocation::Country(default_entry_node_location);
                #[cfg(feature = "fastest-location")]
                return NodeLocation::Fastest;
            }),
            exit_node_location: saved_data.0.exit_node_location.clone().unwrap_or_else(|| {
                #[cfg(not(feature = "fastest-location"))]
                return NodeLocation::Country(default_exit_node_location);
                #[cfg(feature = "fastest-location")]
                return NodeLocation::Fastest;
            }),
            vpn_mode: saved_data.0.vpn_mode.clone().unwrap_or_default(),
            ..Default::default()
        })
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum NodeLocation {
    #[default]
    Fastest,
    Country(Country),
}

impl fmt::Display for NodeLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeLocation::Fastest => write!(f, "NodeLocation: Fastest"),
            NodeLocation::Country(country) => write!(f, "NodeLocation: {}", country),
        }
    }
}
