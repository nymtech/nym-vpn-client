use nym_vpn_proto::proto;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct Socks5Settings {
    pub listen_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct HttpRpcSettings {
    pub listen_address: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub enum Socks5State {
    Disabled = 0,
    Idle = 1,
    Connected = 2,
    Error = 3,
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

impl From<proto::Socks5Settings> for Socks5Settings {
    fn from(settings: proto::Socks5Settings) -> Self {
        Socks5Settings {
            listen_address: settings.listen_address,
        }
    }
}

impl From<proto::HttpRpcSettings> for HttpRpcSettings {
    fn from(settings: proto::HttpRpcSettings) -> Self {
        HttpRpcSettings {
            listen_address: settings.listen_address,
        }
    }
}

impl From<proto::socks5_status::State> for Socks5State {
    fn from(state: proto::socks5_status::State) -> Self {
        match state {
            proto::socks5_status::State::Disabled => Socks5State::Disabled,
            proto::socks5_status::State::Idle => Socks5State::Idle,
            proto::socks5_status::State::Connected => Socks5State::Connected,
            proto::socks5_status::State::Error => Socks5State::Error,
        }
    }
}

impl From<proto::Socks5Status> for Socks5Status {
    fn from(status: proto::Socks5Status) -> Self {
        Socks5Status {
            state: proto::socks5_status::State::try_from(status.state)
                .unwrap_or(proto::socks5_status::State::Disabled)
                .into(),
            socks5_settings: status.socks5_settings.map(|s| s.into()),
            http_rpc_settings: status.http_rpc_settings.map(|s| s.into()),
            error_message: status.error_message,
            active_connections: status.active_connections,
        }
    }
}

impl From<nym_vpn_lib_types::Socks5State> for Socks5State {
    fn from(state: nym_vpn_lib_types::Socks5State) -> Self {
        match state {
            nym_vpn_lib_types::Socks5State::Disabled => Socks5State::Disabled,
            nym_vpn_lib_types::Socks5State::Idle => Socks5State::Idle,
            nym_vpn_lib_types::Socks5State::Connected => Socks5State::Connected,
            nym_vpn_lib_types::Socks5State::Error => Socks5State::Error,
        }
    }
}
