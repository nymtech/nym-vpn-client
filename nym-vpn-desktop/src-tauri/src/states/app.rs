use std::fmt;

use futures::channel::mpsc::UnboundedSender;
use nym_vpn_lib::NymVpnCtrlMessage;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::trace;
use ts_rs::TS;

use crate::{
    country::{Country, DEFAULT_ENTRY_COUNTRY, DEFAULT_EXIT_COUNTRY},
    db::{Db, Key},
    fs::config::AppConfig,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export)]
pub enum ConnectionState {
    // TODO: once the frontend can handle it, include the connection info as part of the connection
    // state.
    //Connected(ConnectionInfo),
    Connected,
    #[default]
    Disconnected,
    Connecting,
    Disconnecting,
    Unknown,
}

#[allow(unused)]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export)]
pub struct ConnectionInfo {
    pub gateway: String,
}

#[derive(Default, Debug, Serialize, Deserialize, TS, Clone, PartialEq, Eq)]
#[ts(export)]
pub enum VpnMode {
    #[default]
    Mixnet,
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
    pub vpn_mode: VpnMode,
    pub entry_node_location: NodeLocation,
    pub exit_node_location: NodeLocation,
    pub connection_start_time: Option<OffsetDateTime>,
    pub vpn_ctrl_tx: Option<UnboundedSender<NymVpnCtrlMessage>>,
}

impl AppState {
    pub fn set_connected(&mut self, start_time: OffsetDateTime, _gateway: String) {
        trace!("update connection state [Connected]");
        // TODO: once the frontend can handle it, set the gateway as part of the connection state
        //self.state = ConnectionState::Connected(ConnectionInfo { gateway });
        self.state = ConnectionState::Connected;
        self.connection_start_time = Some(start_time);
    }
}

impl TryFrom<(&Db, &AppConfig)> for AppState {
    type Error = anyhow::Error;

    fn try_from(store: (&Db, &AppConfig)) -> Result<Self, Self::Error> {
        // retrieve the saved app data from the embedded db
        let entry_node_location = store.0.get_typed::<NodeLocation>(Key::EntryNodeLocation)?;
        let exit_node_location = store.0.get_typed::<NodeLocation>(Key::ExitNodeLocation)?;
        let vpn_mode = store.0.get_typed::<VpnMode>(Key::VpnMode)?;

        // restore any state from the saved app data (previous user session)
        // fallback to config file for locations if not present
        Ok(AppState {
            entry_node_location: entry_node_location
                .unwrap_or(NodeLocation::Country(DEFAULT_ENTRY_COUNTRY.clone())),
            exit_node_location: exit_node_location
                .unwrap_or(NodeLocation::Country(DEFAULT_EXIT_COUNTRY.clone())),
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
