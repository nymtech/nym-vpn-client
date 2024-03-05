use std::fmt;

use anyhow::anyhow;
use futures::channel::mpsc::UnboundedSender;
use nym_vpn_lib::NymVpnCtrlMessage;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use ts_rs::TS;

use crate::{
    country::{Country, DEFAULT_COUNTRY_CODE},
    db::{Db, Key},
    fs::config::AppConfig,
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

impl TryFrom<(&Db, &AppConfig)> for AppState {
    type Error = anyhow::Error;

    fn try_from(store: (&Db, &AppConfig)) -> Result<Self, Self::Error> {
        // retrieve default entry and exit node locations set from
        // the config file
        let default_entry_node_location = Country::try_from(
            store
                .1
                .default_entry_node_location_code
                .as_deref()
                .unwrap_or(DEFAULT_COUNTRY_CODE),
        )
        .map_err(|e| anyhow!("failed to retrieve default entry node location: {e}"))?;

        let default_exit_node_location = Country::try_from(
            store
                .1
                .default_exit_node_location_code
                .as_deref()
                .unwrap_or(DEFAULT_COUNTRY_CODE),
        )
        .map_err(|e| anyhow!("failed to retrieve default exit node location: {e}"))?;

        // retrieve the saved app data from the embedded db
        let entry_node_location = store.0.get_typed::<NodeLocation>(Key::EntryNodeLocation)?;
        let exit_node_location = store.0.get_typed::<NodeLocation>(Key::ExitNodeLocation)?;
        let vpn_mode = store.0.get_typed::<VpnMode>(Key::VpnMode)?;

        // restore any state from the saved app data (previous user session)
        // fallback to config file for locations if not present
        Ok(AppState {
            entry_node_location: entry_node_location
                .unwrap_or(NodeLocation::Country(default_entry_node_location)),
            exit_node_location: exit_node_location
                .unwrap_or(NodeLocation::Country(default_exit_node_location)),
            vpn_mode: vpn_mode.unwrap_or_default(),
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
