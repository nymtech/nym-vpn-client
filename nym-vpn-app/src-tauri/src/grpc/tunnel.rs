use nym_vpn_proto as p;
use p::tunnel_state::{ActionAfterDisconnect, ErrorStateReason, State};
use serde::Serialize;
use tracing::warn;
use ts_rs::TS;

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct WgNode {
    pub endpoint: String,
    pub public_key: String,
    pub private_ipv4: String,
    pub private_ipv6: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MixnetData {
    pub nym_address: Option<String>,
    pub exit_ipr: Option<String>,
    pub ipv4: String,
    pub ipv6: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct WireguardData {
    pub entry: WgNode,
    pub exit: WgNode,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export)]
#[serde(untagged)]
pub enum TunnelData {
    Mixnet(MixnetData),
    Wireguard(WireguardData),
}

#[derive(Serialize, Clone, PartialEq, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Tunnel {
    pub entry_gw_id: String,
    pub exit_gw_id: String,
    pub connected_at: Option<i64>, // unix timestamp
    pub data: TunnelData,
}

#[derive(Default, Debug, Clone, Serialize, PartialEq, TS, strum::Display)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum TunnelState {
    #[default]
    Disconnected,
    Connected(Tunnel),
    Connecting(Option<Tunnel>),
    Disconnecting(Option<TunnelAction>),
    Error(TunnelError),
    Offline {
        reconnect: bool,
    },
}

impl TunnelState {
    pub fn from_proto(tunnel: State) -> Result<TunnelState, &'static str> {
        Ok(match tunnel {
            State::Disconnected(_empty) => TunnelState::Disconnected,
            State::Connecting(c) => {
                let tunnel = c.connection_data.map(Tunnel::try_from).transpose()?;
                TunnelState::Connecting(tunnel)
            }
            State::Connected(c) => {
                let tunnel = c
                    .connection_data
                    .map(Tunnel::try_from)
                    .transpose()?
                    .ok_or("missing tunnel data")?;
                if tunnel.connected_at.is_none() {
                    // just in case
                    warn!("connected but missing connected_at timestamp");
                }
                TunnelState::Connected(tunnel)
            }
            State::Disconnecting(action) => {
                TunnelState::Disconnecting(TunnelAction::from_proto(action.after_disconnect()))
            }
            State::Error(e) => TunnelState::Error(e.reason().into()),
            State::Offline(o) => TunnelState::Offline {
                reconnect: o.reconnect,
            },
        })
    }
}

impl From<p::WireguardNode> for WgNode {
    fn from(p_data: p::WireguardNode) -> Self {
        WgNode {
            endpoint: p_data.endpoint,
            public_key: p_data.public_key,
            private_ipv4: p_data.private_ipv4,
            private_ipv6: p_data.private_ipv6,
        }
    }
}

impl From<p::MixnetConnectionData> for MixnetData {
    fn from(p_data: p::MixnetConnectionData) -> Self {
        MixnetData {
            nym_address: p_data.nym_address.map(|a| a.nym_address),
            exit_ipr: p_data.exit_ipr.map(|a| a.nym_address),
            ipv4: p_data.ipv4,
            ipv6: p_data.ipv6,
        }
    }
}

impl TryFrom<p::WireguardConnectionData> for WireguardData {
    type Error = &'static str;

    fn try_from(p_data: p::WireguardConnectionData) -> Result<Self, Self::Error> {
        Ok(WireguardData {
            entry: p_data.entry.ok_or("missing wg entry node data")?.into(),
            exit: p_data.exit.ok_or("missing wg exit node data")?.into(),
        })
    }
}

impl TryFrom<p::TunnelConnectionData> for TunnelData {
    type Error = &'static str;

    fn try_from(tunnel: p::TunnelConnectionData) -> Result<Self, Self::Error> {
        let tunnel = tunnel.state.ok_or("missing tunnel state data")?;

        match tunnel {
            p::tunnel_connection_data::State::Mixnet(data) => Ok(TunnelData::Mixnet(
                data.data.ok_or("missing Mixnet connection data")?.into(),
            )),
            p::tunnel_connection_data::State::Wireguard(data) => Ok(TunnelData::Wireguard(
                data.data
                    .ok_or("missing Wireguard connection data")?
                    .try_into()?,
            )),
        }
    }
}

impl TryFrom<p::ConnectionData> for Tunnel {
    type Error = &'static str;

    fn try_from(p_data: p::ConnectionData) -> Result<Self, Self::Error> {
        Ok(Tunnel {
            entry_gw_id: p_data
                .entry_gateway
                .ok_or("missing entry gateway ID")?
                .id
                .clone(),
            exit_gw_id: p_data
                .exit_gateway
                .ok_or("missing exit gateway ID")?
                .id
                .clone(),
            connected_at: p_data.connected_at.map(|t| t.seconds),
            data: p_data.tunnel.ok_or("missing tunnel data")?.try_into()?,
        })
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum TunnelError {
    Internal,
    Firewall,
    Routing,
    Dns,
    TunDevice,
    TunnelProvider,
    SameEntryAndExitGw,
    InvalidEntryGwCountry,
    InvalidExitGwCountry,
    BadBandwidthIncrease,
    DuplicateTunFd,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum TunnelAction {
    Error,
    Reconnect,
    Offline,
}

impl TunnelAction {
    fn from_proto(action: ActionAfterDisconnect) -> Option<Self> {
        let action: OptionalTunnelAction = action.into();
        match action {
            OptionalTunnelAction(Some(action)) => Some(action),
            _ => None,
        }
    }
}

// trick to bypass Rust's coherence/orphan Rule (:
pub struct OptionalTunnelAction(Option<TunnelAction>);

impl From<ActionAfterDisconnect> for OptionalTunnelAction {
    fn from(action: ActionAfterDisconnect) -> Self {
        match action {
            ActionAfterDisconnect::Error => OptionalTunnelAction(Some(TunnelAction::Error)),
            ActionAfterDisconnect::Reconnect => OptionalTunnelAction(Some(TunnelAction::Reconnect)),
            ActionAfterDisconnect::Offline => OptionalTunnelAction(Some(TunnelAction::Offline)),
            _ => OptionalTunnelAction(None),
        }
    }
}

impl From<ErrorStateReason> for TunnelError {
    fn from(reason: ErrorStateReason) -> Self {
        match reason {
            ErrorStateReason::Internal => TunnelError::Internal,
            ErrorStateReason::Firewall => TunnelError::Firewall,
            ErrorStateReason::Routing => TunnelError::Routing,
            ErrorStateReason::Dns => TunnelError::Dns,
            ErrorStateReason::TunDevice => TunnelError::TunDevice,
            ErrorStateReason::TunnelProvider => TunnelError::TunnelProvider,
            ErrorStateReason::SameEntryAndExitGateway => TunnelError::SameEntryAndExitGw,
            ErrorStateReason::InvalidEntryGatewayCountry => TunnelError::InvalidEntryGwCountry,
            ErrorStateReason::InvalidExitGatewayCountry => TunnelError::InvalidExitGwCountry,
            ErrorStateReason::BadBandwidthIncrease => TunnelError::BadBandwidthIncrease,
            ErrorStateReason::DuplicateTunFd => TunnelError::DuplicateTunFd,
        }
    }
}
