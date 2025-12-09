use nym_vpn_lib_types as lib;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct Socks5Settings {
    pub listen_address: Option<SocketAddr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct HttpRpcSettings {
    pub listen_address: Option<SocketAddr>,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "kebab-case")]
pub enum Socks5State {
    Disabled,
    Idle,
    Connected,
    Error,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct Socks5Status {
    pub state: Socks5State,
    pub socks5_settings: Option<Socks5Settings>,
    pub http_rpc_settings: Option<HttpRpcSettings>,
    pub error_message: Option<String>,
    pub active_connections: u32,
}

impl From<lib::Socks5Settings> for Socks5Settings {
    fn from(settings: lib::Socks5Settings) -> Self {
        Socks5Settings {
            listen_address: settings.listen_address,
        }
    }
}

impl From<lib::HttpRpcSettings> for HttpRpcSettings {
    fn from(settings: lib::HttpRpcSettings) -> Self {
        HttpRpcSettings {
            listen_address: settings.listen_address,
        }
    }
}

impl From<lib::Socks5State> for Socks5State {
    fn from(state: lib::Socks5State) -> Self {
        match state {
            lib::Socks5State::Disabled => Socks5State::Disabled,
            lib::Socks5State::Idle => Socks5State::Idle,
            lib::Socks5State::Connected => Socks5State::Connected,
            lib::Socks5State::Error => Socks5State::Error,
        }
    }
}

impl From<lib::Socks5Status> for Socks5Status {
    fn from(status: lib::Socks5Status) -> Self {
        Socks5Status {
            state: status.state.into(),
            socks5_settings: Some(status.socks5_settings.into()),
            http_rpc_settings: Some(status.http_rpc_settings.into()),
            error_message: status.error_message,
            active_connections: status.active_connections,
        }
    }
}
