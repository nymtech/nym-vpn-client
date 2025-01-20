use serde::Serialize;
use time::OffsetDateTime;
use ts_rs::TS;

use nym_vpn_proto as p;

#[derive(Serialize, Clone, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct WgNode {
    pub endpoint: String,
    pub public_key: String,
    pub private_ipv4: String,
    pub private_ipv6: String,
}

#[derive(Serialize, Clone, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MixnetData {
    pub nym_address: Option<String>,
    pub exit_ipr: Option<String>,
    pub ipv4: String,
    pub ipv6: String,
}

#[derive(Serialize, Clone, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct WireguardData {
    pub entry: WgNode,
    pub exit: WgNode,
}

#[derive(Serialize, Clone, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum TunnelData {
    Mixnet(MixnetData),
    Wireguard(WireguardData),
}

#[derive(Serialize, Clone, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Tunnel {
    pub entry_gw_id: String,
    pub exit_gw_id: String,
    pub connected_at: Option<i64>,
    pub data: TunnelData,
}

impl From<&p::WireguardNode> for WgNode {
    fn from(p_data: &p::WireguardNode) -> Self {
        WgNode {
            endpoint: p_data.endpoint.clone(),
            public_key: p_data.public_key.clone(),
            private_ipv4: p_data.private_ipv4.clone(),
            private_ipv6: p_data.private_ipv6.clone(),
        }
    }
}

impl From<&p::MixnetConnectionData> for MixnetData {
    fn from(p_data: &p::MixnetConnectionData) -> Self {
        MixnetData {
            nym_address: p_data.nym_address.as_ref().map(|a| a.nym_address.clone()),
            exit_ipr: p_data.exit_ipr.as_ref().map(|a| a.nym_address.clone()),
            ipv4: p_data.ipv4.clone(),
            ipv6: p_data.ipv6.clone(),
        }
    }
}

impl TryFrom<&p::WireguardConnectionData> for WireguardData {
    type Error = &'static str;

    fn try_from(p_data: &p::WireguardConnectionData) -> Result<Self, Self::Error> {
        Ok(WireguardData {
            entry: p_data
                .entry
                .as_ref()
                .ok_or("missing wg entry node data")?
                .into(),
            exit: p_data
                .exit
                .as_ref()
                .ok_or("missing wg exit node data")?
                .into(),
        })
    }
}

impl TryFrom<&p::TunnelConnectionData> for TunnelData {
    type Error = &'static str;

    fn try_from(tunnel: &p::TunnelConnectionData) -> Result<Self, Self::Error> {
        let tunnel = tunnel.state.as_ref().ok_or("missing tunnel state data")?;

        match tunnel {
            p::tunnel_connection_data::State::Mixnet(data) => Ok(TunnelData::Mixnet(
                data.data
                    .as_ref()
                    .ok_or("missing Mixnet connection data")?
                    .into(),
            )),
            p::tunnel_connection_data::State::Wireguard(data) => Ok(TunnelData::Wireguard(
                data.data
                    .as_ref()
                    .ok_or("missing Wireguard connection data")?
                    .try_into()?,
            )),
        }
    }
}

impl TryFrom<&p::ConnectionData> for Tunnel {
    type Error = &'static str;

    fn try_from(p_data: &p::ConnectionData) -> Result<Self, Self::Error> {
        let connected_at = p_data
            .connected_at
            .as_ref()
            .map(|t| OffsetDateTime::from_unix_timestamp(t.seconds))
            .ok_or("failed to parse connection timestamp")?;

        Ok(Tunnel {
            entry_gw_id: p_data
                .entry_gateway
                .as_ref()
                .ok_or("missing entry gateway ID")?
                .id
                .clone(),
            exit_gw_id: p_data
                .entry_gateway
                .as_ref()
                .ok_or("missing exit gateway ID")?
                .id
                .clone(),
            connected_at: connected_at.map(|t| t.unix_timestamp()).ok(),
            data: p_data
                .tunnel
                .as_ref()
                .ok_or("missing tunnel data")?
                .try_into()?,
        })
    }
}
