// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{AddrParseError, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
};

use nym_vpn_lib_types::{
    ActionAfterDisconnect, ConnectionData, ErrorStateReason, MixnetConnectionData,
    TunnelConnectionData, TunnelState, WireguardConnectionData, WireguardNode,
};
use prost::DecodeError;

use crate::{
    tunnel_connection_data::{
        Mixnet as ProtoMixnetConnectionDataVariant, State as ProtoTunnelConnectionDataState,
        Wireguard as ProtoWireguardConnectionDataVariant,
    },
    tunnel_state::{
        ActionAfterDisconnect as ProtoActionAfterDisconnect, Connected as ProtoConnected,
        Connecting as ProtoConnecting, Disconnected as ProtoDisconnected,
        Disconnecting as ProtoDisconnecting, Error as ProtoError,
        ErrorStateReason as ProtoErrorStateReason, Offline as ProtoOffline, State as ProtoState,
    },
    ConnectionData as ProtoConnectionData, MixnetConnectionData as ProtoMixnetConnectionData,
    TunnelConnectionData as ProtoTunnelConnectionData, TunnelState as ProtoTunnelState,
    WireguardConnectionData as ProtoWireguardConnectionData, WireguardNode as ProtoWireguardNode,
};

impl From<ProtoActionAfterDisconnect> for ActionAfterDisconnect {
    fn from(value: ProtoActionAfterDisconnect) -> Self {
        match value {
            ProtoActionAfterDisconnect::Error => Self::Error,
            ProtoActionAfterDisconnect::Nothing => Self::Nothing,
            ProtoActionAfterDisconnect::Offline => Self::Offline,
            ProtoActionAfterDisconnect::Reconnect => Self::Reconnect,
        }
    }
}

impl From<ProtoErrorStateReason> for ErrorStateReason {
    fn from(value: ProtoErrorStateReason) -> Self {
        match value {
            ProtoErrorStateReason::Firewall => Self::Firewall,
            ProtoErrorStateReason::Routing => Self::Routing,
            ProtoErrorStateReason::Dns => Self::Dns,
            ProtoErrorStateReason::TunDevice => Self::TunDevice,
            ProtoErrorStateReason::TunnelProvider => Self::TunnelProvider,
            ProtoErrorStateReason::SameEntryAndExitGateway => Self::SameEntryAndExitGateway,
            ProtoErrorStateReason::InvalidEntryGatewayCountry => Self::InvalidEntryGatewayCountry,
            ProtoErrorStateReason::InvalidExitGatewayCountry => Self::InvalidExitGatewayCountry,
            ProtoErrorStateReason::BadBandwidthIncrease => Self::BadBandwidthIncrease,
            ProtoErrorStateReason::DuplicateTunFd => Self::DuplicateTunFd,
            ProtoErrorStateReason::Internal => Self::Internal,
        }
    }
}

impl TryFrom<ProtoTunnelState> for TunnelState {
    type Error = FromProtobufTypeError;

    fn try_from(value: ProtoTunnelState) -> Result<TunnelState> {
        let state = value
            .state
            .ok_or(FromProtobufTypeError::NoValueSet("TunnelState.state"))?;

        Ok(match state {
            ProtoState::Disconnected(ProtoDisconnected {}) => Self::Disconnected,
            ProtoState::Disconnecting(ProtoDisconnecting { after_disconnect }) => {
                let proto_after_disconnect = ProtoActionAfterDisconnect::try_from(after_disconnect)
                    .map_err(|e| {
                        FromProtobufTypeError::Decode("TunnelState.after_disconnect", e)
                    })?;

                Self::Disconnecting {
                    after_disconnect: ActionAfterDisconnect::from(proto_after_disconnect),
                }
            }
            ProtoState::Connecting(ProtoConnecting { connection_data }) => {
                let connection_data = connection_data.map(ConnectionData::try_from).transpose()?;

                Self::Connecting { connection_data }
            }
            ProtoState::Connected(ProtoConnected { connection_data }) => {
                let connection_data = connection_data
                    .ok_or(FromProtobufTypeError::NoValueSet(
                        "TunnelState.connection_data",
                    ))
                    .and_then(ConnectionData::try_from)?;

                Self::Connected { connection_data }
            }
            ProtoState::Error(ProtoError { reason }) => {
                let reason = ProtoErrorStateReason::try_from(reason)
                    .map_err(|e| FromProtobufTypeError::Decode("TunnelState.after_disconnect", e))
                    .map(ErrorStateReason::from)?;
                Self::Error(reason)
            }
            ProtoState::Offline(ProtoOffline { reconnect }) => Self::Offline { reconnect },
        })
    }
}

impl TryFrom<ProtoConnectionData> for ConnectionData {
    type Error = FromProtobufTypeError;

    fn try_from(value: ProtoConnectionData) -> Result<Self> {
        let connected_at = value
            .connected_at
            .map(|timestamp| {
                crate::conversions::prost::prost_timestamp_into_offset_datetime(timestamp)
            })
            .transpose()
            .map_err(|e| FromProtobufTypeError::ConvertTime("ConnectionData.connected_at", e))?;

        let tunnel_connection_data = value
            .tunnel
            .ok_or(FromProtobufTypeError::NoValueSet("ConnectionData.tunnel"))?;

        Ok(Self {
            connected_at,
            entry_gateway: value
                .entry_gateway
                .ok_or(FromProtobufTypeError::NoValueSet(
                    "ConnectionData.entry_gateway",
                ))?
                .id,
            exit_gateway: value
                .exit_gateway
                .ok_or(FromProtobufTypeError::NoValueSet(
                    "ConnectionData.exit_gateway",
                ))?
                .id,
            tunnel: TunnelConnectionData::try_from(tunnel_connection_data)?,
        })
    }
}

impl TryFrom<ProtoTunnelConnectionData> for TunnelConnectionData {
    type Error = FromProtobufTypeError;

    fn try_from(value: ProtoTunnelConnectionData) -> Result<Self> {
        let state = value.state.ok_or(FromProtobufTypeError::NoValueSet(
            "TunnelConnectionData.state",
        ))?;

        Ok(match state {
            ProtoTunnelConnectionDataState::Mixnet(ProtoMixnetConnectionDataVariant { data }) => {
                Self::Mixnet(MixnetConnectionData::try_from(data.ok_or(
                    FromProtobufTypeError::NoValueSet("TunnelConnectionData::Mixnet.data"),
                )?)?)
            }
            ProtoTunnelConnectionDataState::Wireguard(ProtoWireguardConnectionDataVariant {
                data,
            }) => Self::Wireguard(WireguardConnectionData::try_from(data.ok_or(
                FromProtobufTypeError::NoValueSet("TunnelConnectionData::Wireguard.data"),
            )?)?),
        })
    }
}

impl TryFrom<ProtoMixnetConnectionData> for MixnetConnectionData {
    type Error = FromProtobufTypeError;

    fn try_from(value: ProtoMixnetConnectionData) -> Result<Self> {
        Ok(Self {
            nym_address: value
                .nym_address
                .ok_or(FromProtobufTypeError::NoValueSet(
                    "MixnetConnectionData.nym_address",
                ))?
                .nym_address,
            exit_ipr: value
                .exit_ipr
                .ok_or(FromProtobufTypeError::NoValueSet(
                    "MixnetConnectionData.exit_ipr",
                ))?
                .nym_address,
            ipv4: Ipv4Addr::from_str(&value.ipv4)
                .map_err(|e| FromProtobufTypeError::ParseAddr("MixnetConnectionData.ipv4", e))?,
            ipv6: Ipv6Addr::from_str(&value.ipv6)
                .map_err(|e| FromProtobufTypeError::ParseAddr("MixnetConnectionData.ipv6", e))?,
        })
    }
}

impl TryFrom<ProtoWireguardConnectionData> for WireguardConnectionData {
    type Error = FromProtobufTypeError;

    fn try_from(value: ProtoWireguardConnectionData) -> Result<Self> {
        Ok(Self {
            entry: WireguardNode::try_from(value.entry.ok_or(
                FromProtobufTypeError::NoValueSet("WireguardConnectionData.entry"),
            )?)?,
            exit: WireguardNode::try_from(value.exit.ok_or(FromProtobufTypeError::NoValueSet(
                "WireguardConnectionData.exit",
            ))?)?,
        })
    }
}

impl TryFrom<ProtoWireguardNode> for WireguardNode {
    type Error = FromProtobufTypeError;

    fn try_from(value: ProtoWireguardNode) -> Result<Self> {
        Ok(Self {
            endpoint: SocketAddr::from_str(&value.endpoint)
                .map_err(|e| FromProtobufTypeError::ParseAddr("WireguardNode.endpoint", e))?,
            public_key: value.public_key,
            private_ipv4: Ipv4Addr::from_str(&value.private_ipv4)
                .map_err(|e| FromProtobufTypeError::ParseAddr("WireguardNode.private_ipv4", e))?,
            private_ipv6: Ipv6Addr::from_str(&value.private_ipv6)
                .map_err(|e| FromProtobufTypeError::ParseAddr("WireguardNode.private_ipv6", e))?,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FromProtobufTypeError {
    #[error("No value set for {0}")]
    NoValueSet(&'static str),

    #[error("Failed to decode {0}: {1}")]
    Decode(&'static str, #[source] DecodeError),

    #[error("Failed to convert time {0}: {1}")]
    ConvertTime(&'static str, #[source] time::Error),

    #[error("Failed to parse address {0}: {1}")]
    ParseAddr(&'static str, #[source] AddrParseError),
}

pub type Result<T, E = FromProtobufTypeError> = std::result::Result<T, E>;
