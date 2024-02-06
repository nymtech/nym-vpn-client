use std::fmt;

use futures::channel::mpsc::UnboundedSender;
use nym_vpn_lib::NymVpnCtrlMessage;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use ts_rs::TS;

use crate::fs::data::AppData;

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

impl From<&AppData> for AppState {
    fn from(app_data: &AppData) -> Self {
        AppState {
            entry_node_location: app_data.entry_node_location.clone().unwrap_or_default(),
            exit_node_location: app_data.exit_node_location.clone().unwrap_or_default(),
            vpn_mode: app_data.vpn_mode.clone().unwrap_or_default(),
            ..Default::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum NodeLocation {
    #[default]
    Fastest,
    Country(Country),
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct Country {
    pub name: String,
    pub code: String,
}

impl fmt::Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Country: {} [{}]", self.name, self.code)
    }
}

impl fmt::Display for NodeLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeLocation::Fastest => write!(f, "NodeLocation: Fastest"),
            NodeLocation::Country(country) => write!(f, "NodeLocation: {}", country),
        }
    }
}
