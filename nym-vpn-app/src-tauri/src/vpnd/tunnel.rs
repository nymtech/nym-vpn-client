use nym_vpn_lib_types as lib;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use ts_rs::TS;

use super::tunnel_error::TunnelError;

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub nym_address: String,
    pub gateway_id: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct WgNode {
    pub endpoint: String,
    pub public_key: String,
    pub private_ipv4: String,
    pub private_ipv6: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct MixnetData {
    pub nym_address: Address,
    pub exit_ipr: Address,
    pub ipv4: String,
    pub ipv6: Option<String>,
    pub entry_ip: String,
    pub exit_ip: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct BridgeAddress {
    pub listen_addr: String,
    pub remote_addr: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct WireguardData {
    pub entry_bridge_addr: Option<BridgeAddress>,
    pub entry: WgNode,
    pub exit: WgNode,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(untagged)]
pub enum TunnelData {
    Mixnet(MixnetData),
    Wireguard(WireguardData),
}

#[derive(Serialize, Clone, PartialEq, Debug, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct Tunnel {
    pub entry_gw_id: String,
    pub exit_gw_id: String,
    pub connected_at: i64, // unix timestamp
    pub data: TunnelData,
}

#[derive(Default, Debug, Clone, Serialize, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "kebab-case")]
pub enum TunnelType {
    #[default]
    Wg,
    Mixnet,
}

#[derive(Default, Debug, Clone, Serialize, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "kebab-case")]
pub enum ConnectingProgress {
    #[default]
    ResolvingApiAddresses,
    AwaitingAccountReadiness,
    AwaitingCredentialsAvailability,
    RefreshingGateways,
    SelectingGateways,
    RegisteringWithGateways,
    ConnectingTunnel,
}

#[derive(Serialize, Clone, PartialEq, Debug, TS, Default)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct ConnectingState {
    pub tunnel_type: TunnelType,
    pub progress: ConnectingProgress,
    pub tunnel: Option<TunnelData>,
    pub retry_attempt: u32,
    pub entry_gw_id: Option<String>,
    pub exit_gw_id: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TTunnelState")]
#[serde(rename_all = "camelCase")]
pub enum TunnelState {
    #[default]
    Disconnected,
    Connected(Tunnel),
    Connecting(ConnectingState),
    Disconnecting(Option<TunnelAction>),
    Error(TunnelError),
    Offline {
        reconnect: bool,
    },
}

impl TunnelState {
    pub fn from_lib(tunnel: lib::TunnelState) -> TunnelState {
        match tunnel {
            lib::TunnelState::Disconnected => TunnelState::Disconnected,
            lib::TunnelState::Connecting {
                connection_data,
                retry_attempt,
                state,
                tunnel_type,
            } => {
                let entry_gw_id = connection_data.as_ref().map(|d| d.entry_gateway.id.clone());
                let exit_gw_id = connection_data.as_ref().map(|d| d.exit_gateway.id.clone());
                let tunnel = connection_data.and_then(|d| d.tunnel.map(TunnelData::from));
                TunnelState::Connecting(ConnectingState {
                    tunnel_type: tunnel_type.into(),
                    progress: state.into(),
                    tunnel,
                    retry_attempt,
                    entry_gw_id,
                    exit_gw_id,
                })
            }
            lib::TunnelState::Connected { connection_data } => {
                TunnelState::Connected(connection_data.into())
            }
            lib::TunnelState::Disconnecting { after_disconnect } => {
                TunnelState::Disconnecting(TunnelAction::from_lib(after_disconnect))
            }
            lib::TunnelState::Error(e) => TunnelState::Error(e.into()),
            lib::TunnelState::Offline { reconnect } => TunnelState::Offline { reconnect },
        }
    }
}

impl From<lib::NymAddress> for Address {
    fn from(a: lib::NymAddress) -> Self {
        Address {
            nym_address: a.nym_address,
            gateway_id: a.gateway_id,
        }
    }
}

impl From<lib::WireguardNode> for WgNode {
    fn from(node: lib::WireguardNode) -> Self {
        WgNode {
            endpoint: node.endpoint.to_string(),
            public_key: node.public_key,
            private_ipv4: node.private_ipv4.to_string(),
            private_ipv6: node.private_ipv6.map(|ip| ip.to_string()),
        }
    }
}

impl From<lib::MixnetConnectionData> for MixnetData {
    fn from(data: lib::MixnetConnectionData) -> Self {
        MixnetData {
            nym_address: data.nym_address.into(),
            exit_ipr: data.exit_ipr.into(),
            entry_ip: data.entry_ip.to_string(),
            exit_ip: data.exit_ip.to_string(),
            ipv4: data.ipv4.to_string(),
            ipv6: data.ipv6.map(|ip| ip.to_string()),
        }
    }
}

impl From<lib::BridgeAddress> for BridgeAddress {
    fn from(addr: lib::BridgeAddress) -> Self {
        BridgeAddress {
            listen_addr: addr.listen_addr.to_string(),
            remote_addr: addr.remote_addr.to_string(),
        }
    }
}

impl From<lib::WireguardConnectionData> for WireguardData {
    fn from(data: lib::WireguardConnectionData) -> Self {
        WireguardData {
            entry_bridge_addr: data.entry_bridge_addr.map(BridgeAddress::from),
            entry: data.entry.into(),
            exit: data.exit.into(),
        }
    }
}

impl From<lib::TunnelConnectionData> for TunnelData {
    fn from(tunnel: lib::TunnelConnectionData) -> Self {
        match tunnel {
            lib::TunnelConnectionData::Mixnet(data) => TunnelData::Mixnet(data.into()),
            lib::TunnelConnectionData::Wireguard(data) => TunnelData::Wireguard(data.into()),
        }
    }
}

impl From<lib::ConnectionData> for Tunnel {
    fn from(data: lib::ConnectionData) -> Self {
        Tunnel {
            entry_gw_id: data.entry_gateway.id,
            exit_gw_id: data.exit_gateway.id,
            connected_at: data.connected_at.unix_timestamp(),
            data: data.tunnel.into(),
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, TS, strum::Display)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum TunnelAction {
    Error,
    Reconnect,
    Offline,
}

impl TunnelAction {
    fn from_lib(action: lib::ActionAfterDisconnect) -> Option<Self> {
        let action: OptionalTunnelAction = action.into();
        match action {
            OptionalTunnelAction(Some(action)) => Some(action),
            _ => None,
        }
    }
}

// trick to bypass Rust's coherence/orphan Rule (:
pub struct OptionalTunnelAction(Option<TunnelAction>);

impl From<lib::ActionAfterDisconnect> for OptionalTunnelAction {
    fn from(action: lib::ActionAfterDisconnect) -> Self {
        match action {
            lib::ActionAfterDisconnect::Error => OptionalTunnelAction(Some(TunnelAction::Error)),
            lib::ActionAfterDisconnect::Reconnect => {
                OptionalTunnelAction(Some(TunnelAction::Reconnect))
            }
            lib::ActionAfterDisconnect::Offline => {
                OptionalTunnelAction(Some(TunnelAction::Offline))
            }
            _ => OptionalTunnelAction(None),
        }
    }
}

impl Display for TunnelState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TunnelState::Disconnected => write!(f, "disconnected"),
            TunnelState::Connected(_) => write!(f, "connected"),
            TunnelState::Connecting(_) => write!(f, "connecting"),
            TunnelState::Disconnecting(a) => {
                if let Some(action) = a {
                    write!(f, "disconnecting - next action ({action})")
                } else {
                    write!(f, "disconnecting")
                }
            }
            TunnelState::Error(e) => {
                write!(f, "error - {e}")
            }
            TunnelState::Offline { reconnect } => {
                write!(f, "offline - reconnect ({reconnect})")
            }
        }
    }
}

impl From<lib::TunnelType> for TunnelType {
    fn from(kind: lib::TunnelType) -> Self {
        match kind {
            lib::TunnelType::Mixnet => TunnelType::Mixnet,
            lib::TunnelType::Wireguard => TunnelType::Wg,
        }
    }
}

impl From<lib::EstablishConnectionState> for ConnectingProgress {
    fn from(state: lib::EstablishConnectionState) -> Self {
        match state {
            lib::EstablishConnectionState::ResolvingApiAddresses => {
                ConnectingProgress::ResolvingApiAddresses
            }
            lib::EstablishConnectionState::AwaitingAccountReadiness => {
                ConnectingProgress::AwaitingAccountReadiness
            }
            lib::EstablishConnectionState::AwaitingCredentialsAvailability => {
                ConnectingProgress::AwaitingCredentialsAvailability
            }
            lib::EstablishConnectionState::RefreshingGateways => {
                ConnectingProgress::RefreshingGateways
            }
            lib::EstablishConnectionState::SelectingGateways => {
                ConnectingProgress::SelectingGateways
            }
            lib::EstablishConnectionState::RegisteringWithGateways => {
                ConnectingProgress::RegisteringWithGateways
            }
            lib::EstablishConnectionState::ConnectingTunnel => ConnectingProgress::ConnectingTunnel,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct SplitTunnelSettings {
    pub enabled: bool,
    pub apps: Vec<SplitApp>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct GeoExclusionSettings {
    pub enabled: bool,
    pub listen_port: u16,
    pub excluded_countries: Vec<String>,
}

impl From<lib::GeoExclusionSettings> for GeoExclusionSettings {
    fn from(settings: lib::GeoExclusionSettings) -> Self {
        GeoExclusionSettings {
            enabled: settings.enabled,
            listen_port: settings.listen_port,
            excluded_countries: settings.excluded_countries,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct SplitApp {
    pub path: String,
}

impl From<lib::SplitTunnelSettings> for SplitTunnelSettings {
    fn from(settings: lib::SplitTunnelSettings) -> Self {
        SplitTunnelSettings {
            enabled: settings.enabled,
            apps: settings.apps.into_iter().map(SplitApp::from).collect(),
        }
    }
}

impl From<lib::SplitApp> for SplitApp {
    fn from(app: lib::SplitApp) -> Self {
        SplitApp { path: app.path }
    }
}

impl From<SplitApp> for lib::SplitApp {
    fn from(app: SplitApp) -> Self {
        lib::SplitApp { path: app.path }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub enum FrontingMode {
    Off,
    OnRetry,
    Always,
}

impl From<lib::FrontingMode> for FrontingMode {
    fn from(mode: lib::FrontingMode) -> Self {
        match mode {
            lib::FrontingMode::Off => FrontingMode::Off,
            lib::FrontingMode::OnRetry => FrontingMode::OnRetry,
            lib::FrontingMode::Always => FrontingMode::Always,
        }
    }
}

impl From<FrontingMode> for lib::FrontingMode {
    fn from(mode: FrontingMode) -> Self {
        match mode {
            FrontingMode::Off => lib::FrontingMode::Off,
            FrontingMode::OnRetry => lib::FrontingMode::OnRetry,
            FrontingMode::Always => lib::FrontingMode::Always,
        }
    }
}

impl Display for FrontingMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FrontingMode::Off => write!(f, "off"),
            FrontingMode::OnRetry => write!(f, "on retry"),
            FrontingMode::Always => write!(f, "always"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub enum Profile {
    Safest,
    MostPrivate,
    Fastest,
    Random,
}

impl From<Profile> for lib::Profile {
    fn from(profile: Profile) -> Self {
        match profile {
            Profile::Safest => lib::Profile::Safest,
            Profile::MostPrivate => lib::Profile::MostPrivate,
            Profile::Fastest => lib::Profile::Fastest,
            Profile::Random => lib::Profile::Random,
        }
    }
}
