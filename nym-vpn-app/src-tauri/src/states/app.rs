use nym_vpn_proto::ConnectionStatus;
use serde::{Deserialize, Serialize};
use std::fmt;
use time::OffsetDateTime;
use tracing::error;
use ts_rs::TS;

use crate::{
    cli::Cli,
    country::Country,
    db::{Db, Key},
    fs::config::AppConfig,
    grpc::client::{VpndInfo, VpndStatus},
};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS, strum::Display)]
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

#[derive(Default, Debug, Serialize, Deserialize, TS, Clone, PartialEq, Eq)]
#[ts(export)]
pub enum VpnMode {
    Mixnet,
    // ⚠ keep this default in sync with the one declared in
    // src/constants.ts
    #[default]
    TwoHop,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub vpnd_status: VpndStatus,
    pub vpnd_info: Option<VpndInfo>,
    pub state: ConnectionState,
    pub vpn_mode: VpnMode,
    pub connection_start_time: Option<OffsetDateTime>,
    pub dns_server: Option<String>,
    pub credentials_mode: bool,
}

impl AppState {
    pub fn new(db: &Db, config: &AppConfig, cli: &Cli) -> Self {
        let vpn_mode = db
            .get_typed::<VpnMode>(Key::VpnMode)
            .inspect_err(|e| error!("failed to retrieve vpn mode from db: {e}"))
            .ok()
            .flatten()
            .unwrap_or_default();
        let dns_server: Option<String> = cli.dns.clone().or(config.dns_server.clone());

        // restore any state from the saved app data (previous user session)
        AppState {
            vpn_mode,
            dns_server,
            credentials_mode: cli.dev_mode,
            ..Default::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, TS)]
#[serde(untagged)]
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
