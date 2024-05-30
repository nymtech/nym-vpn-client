use nym_vpn_proto::ConnectionStatus;
use serde::{Deserialize, Serialize};
use std::fmt;
use time::OffsetDateTime;
use ts_rs::TS;

use crate::{
    cli::Cli,
    country::{Country, DEFAULT_ENTRY_COUNTRY, DEFAULT_EXIT_COUNTRY},
    db::{Db, Key},
    fs::config::AppConfig,
    grpc::client::VpndStatus,
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
    pub vpnd_status: VpndStatus,
    pub state: ConnectionState,
    pub error: Option<String>,
    pub vpn_mode: VpnMode,
    pub entry_node_location: NodeLocation,
    pub exit_node_location: NodeLocation,
    pub tunnel: Option<TunnelConfig>,
    pub connection_start_time: Option<OffsetDateTime>,
    pub dns_server: Option<String>,
}

impl TryFrom<(&Db, &AppConfig, &Cli)> for AppState {
    type Error = anyhow::Error;

    fn try_from(store: (&Db, &AppConfig, &Cli)) -> Result<Self, Self::Error> {
        // retrieve the saved app data from the embedded db
        let entry_node_location = store.0.get_typed::<NodeLocation>(Key::EntryNodeLocation)?;
        let exit_node_location = store.0.get_typed::<NodeLocation>(Key::ExitNodeLocation)?;
        let vpn_mode = store.0.get_typed::<VpnMode>(Key::VpnMode)?;
        let dns_server: Option<String> = store.2.dns.clone().or(store.1.dns_server.clone());

        // restore any state from the saved app data (previous user session)
        Ok(AppState {
            entry_node_location: entry_node_location
                .unwrap_or(NodeLocation::Country(DEFAULT_ENTRY_COUNTRY.clone())),
            exit_node_location: exit_node_location
                .unwrap_or(NodeLocation::Country(DEFAULT_EXIT_COUNTRY.clone())),
            vpn_mode: vpn_mode.unwrap_or_default(),
            dns_server,
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

impl From<ConnectionStatus> for ConnectionState {
    fn from(status: ConnectionStatus) -> Self {
        match status {
            ConnectionStatus::Connected => ConnectionState::Connected,
            ConnectionStatus::NotConnected => ConnectionState::Disconnected,
            ConnectionStatus::Connecting => ConnectionState::Connecting,
            ConnectionStatus::Disconnecting => ConnectionState::Disconnecting,
            ConnectionStatus::Unknown => ConnectionState::Unknown,
            ConnectionStatus::StatusUnspecified => ConnectionState::Unknown,
            // this variant means "Not connected, but with an error"
            // so it should be treated as disconnected
            ConnectionStatus::ConnectionFailed => ConnectionState::Disconnected,
        }
    }
}
