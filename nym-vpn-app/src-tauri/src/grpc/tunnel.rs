use nym_vpn_proto as p;
use p::tunnel_state::{ActionAfterDisconnect, State};
use serde::Serialize;
use std::fmt::{Display, Formatter};
use tracing::warn;
use ts_rs::TS;

use super::tunnel_error::TunnelError;

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub nym_address: String,
    pub gateway_id: String,
}

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
    pub nym_address: Option<Address>,
    pub exit_ipr: Option<Address>,
    pub ipv4: String,
    pub ipv6: String,
    pub entry_ip: String,
    pub exit_ip: String,
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

#[derive(Default, Debug, Clone, Serialize, PartialEq, TS)]
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
            State::Error(e) => TunnelState::Error(e.error_state_reason.into()),
            State::Offline(o) => TunnelState::Offline {
                reconnect: o.reconnect,
            },
        })
    }
}

impl From<p::Address> for Address {
    fn from(a: p::Address) -> Self {
        Address {
            nym_address: a.nym_address,
            gateway_id: a.gateway_id,
        }
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
            nym_address: p_data.nym_address.map(Address::from),
            exit_ipr: p_data.exit_ipr.map(Address::from),
            ipv4: p_data.ipv4,
            ipv6: p_data.ipv6,
            entry_ip: p_data.entry_ip,
            exit_ip: p_data.exit_ip,
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

#[derive(Serialize, Clone, Debug, PartialEq, TS, strum::Display)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
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

impl Display for TunnelState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TunnelState::Disconnected => write!(f, "disconnected"),
            TunnelState::Connected(_) => write!(f, "connected"),
            TunnelState::Connecting(_) => write!(f, "connecting"),
            TunnelState::Disconnecting(a) => {
                if let Some(action) = a {
                    write!(f, "disconnecting - next action ({})", action)
                } else {
                    write!(f, "disconnecting")
                }
            }
            TunnelState::Error(e) => {
                write!(f, "error - {}", e)
            }
            TunnelState::Offline { reconnect } => {
                write!(f, "offline - reconnect ({})", reconnect)
            }
        }
    }
}
