// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
};

use nym_vpn_lib_types::{
    ActionAfterDisconnect, ClientErrorReason, ConnectionData, Gateway, MixnetConnectionData,
    NymAddress, TunnelConnectionData, TunnelState, WireguardConnectionData, WireguardNode,
};

use crate::tunnel_state::ErrorStateReason;
use crate::{
    conversions::ConversionError,
    tunnel_connection_data::{
        Mixnet as ProtoMixnetConnectionDataVariant, State as ProtoTunnelConnectionDataState,
        Wireguard as ProtoWireguardConnectionDataVariant,
    },
    tunnel_state::{
        ActionAfterDisconnect as ProtoActionAfterDisconnect, Connected as ProtoConnected,
        Connecting as ProtoConnecting, Disconnected as ProtoDisconnected,
        Disconnecting as ProtoDisconnecting, Error as ProtoError, Offline as ProtoOffline,
        State as ProtoState,
    },
    Address as ProtoAddress, ConnectionData as ProtoConnectionData, Gateway as ProtoGateway,
    MixnetConnectionData as ProtoMixnetConnectionData,
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

impl From<ProtoError> for ClientErrorReason {
    fn from(value: ProtoError) -> Self {
        match value.reason() {
            ErrorStateReason::Firewall => ClientErrorReason::Firewall,
            ErrorStateReason::Routing => ClientErrorReason::Routing,
            ErrorStateReason::SameEntryAndExitGateway => ClientErrorReason::SameEntryAndExitGateway,
            ErrorStateReason::InvalidEntryGatewayCountry => {
                ClientErrorReason::InvalidEntryGatewayCountry
            }
            ErrorStateReason::InvalidExitGatewayCountry => {
                ClientErrorReason::InvalidExitGatewayCountry
            }
            ErrorStateReason::MaxDevicesReached => ClientErrorReason::MaxDevicesReached,
            ErrorStateReason::BandwidthExceeded => ClientErrorReason::BandwidthExceeded,
            ErrorStateReason::SubscriptionExpired => ClientErrorReason::SubscriptionExpired,
            ErrorStateReason::Dns => ClientErrorReason::Dns(value.detail),
            ErrorStateReason::Api => ClientErrorReason::Api(value.detail),
            ErrorStateReason::Internal => ClientErrorReason::Internal(value.detail),
        }
    }
}

impl TryFrom<ProtoTunnelState> for TunnelState {
    type Error = ConversionError;

    fn try_from(value: ProtoTunnelState) -> Result<Self, ConversionError> {
        let state = value
            .state
            .ok_or(ConversionError::NoValueSet("TunnelState.state"))?;

        Ok(match state {
            ProtoState::Disconnected(ProtoDisconnected {}) => Self::Disconnected,
            ProtoState::Disconnecting(ProtoDisconnecting { after_disconnect }) => {
                let proto_after_disconnect = ProtoActionAfterDisconnect::try_from(after_disconnect)
                    .map_err(|e| ConversionError::Decode("TunnelState.after_disconnect", e))?;

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
                    .ok_or(ConversionError::NoValueSet("TunnelState.connection_data"))
                    .and_then(ConnectionData::try_from)?;

                Self::Connected { connection_data }
            }
            ProtoState::Error(error_state_reason) => Self::Error(error_state_reason.into()),
            ProtoState::Offline(ProtoOffline { reconnect }) => Self::Offline { reconnect },
        })
    }
}

impl TryFrom<ProtoConnectionData> for ConnectionData {
    type Error = ConversionError;

    fn try_from(value: ProtoConnectionData) -> Result<Self, Self::Error> {
        let connected_at = value
            .connected_at
            .map(|timestamp| {
                crate::conversions::prost::prost_timestamp_into_offset_datetime(timestamp)
            })
            .transpose()
            .map_err(|e| ConversionError::ConvertTime("ConnectionData.connected_at", e))?;

        let tunnel_connection_data = value
            .tunnel
            .ok_or(ConversionError::NoValueSet("ConnectionData.tunnel"))?;

        Ok(Self {
            connected_at,
            entry_gateway: value
                .entry_gateway
                .map(Gateway::from)
                .ok_or(ConversionError::NoValueSet("ConnectionData.entry_gateway"))?,
            exit_gateway: value
                .exit_gateway
                .map(Gateway::from)
                .ok_or(ConversionError::NoValueSet("ConnectionData.exit_gateway"))?,
            tunnel: TunnelConnectionData::try_from(tunnel_connection_data)?,
        })
    }
}

impl TryFrom<ProtoTunnelConnectionData> for TunnelConnectionData {
    type Error = ConversionError;

    fn try_from(value: ProtoTunnelConnectionData) -> Result<Self, Self::Error> {
        let state = value
            .state
            .ok_or(ConversionError::NoValueSet("TunnelConnectionData.state"))?;

        Ok(match state {
            ProtoTunnelConnectionDataState::Mixnet(ProtoMixnetConnectionDataVariant { data }) => {
                Self::Mixnet(MixnetConnectionData::try_from(data.ok_or(
                    ConversionError::NoValueSet("TunnelConnectionData::Mixnet.data"),
                )?)?)
            }
            ProtoTunnelConnectionDataState::Wireguard(ProtoWireguardConnectionDataVariant {
                data,
            }) => Self::Wireguard(WireguardConnectionData::try_from(data.ok_or(
                ConversionError::NoValueSet("TunnelConnectionData::Wireguard.data"),
            )?)?),
        })
    }
}

impl TryFrom<ProtoMixnetConnectionData> for MixnetConnectionData {
    type Error = ConversionError;

    fn try_from(value: ProtoMixnetConnectionData) -> Result<Self, Self::Error> {
        Ok(Self {
            nym_address: value.nym_address.map(NymAddress::from).ok_or(
                ConversionError::NoValueSet("MixnetConnectionData.nym_address"),
            )?,
            exit_ipr: value
                .exit_ipr
                .map(NymAddress::from)
                .ok_or(ConversionError::NoValueSet("MixnetConnectionData.exit_ipr"))?,
            entry_ip: IpAddr::from_str(&value.entry_ip)
                .map_err(|e| ConversionError::ParseAddr("MixnetConnectionData.entry_ip", e))?,
            exit_ip: IpAddr::from_str(&value.exit_ip)
                .map_err(|e| ConversionError::ParseAddr("MixnetConnectionData.exit_ip", e))?,
            ipv4: Ipv4Addr::from_str(&value.ipv4)
                .map_err(|e| ConversionError::ParseAddr("MixnetConnectionData.ipv4", e))?,
            ipv6: Ipv6Addr::from_str(&value.ipv6)
                .map_err(|e| ConversionError::ParseAddr("MixnetConnectionData.ipv6", e))?,
        })
    }
}

impl TryFrom<ProtoWireguardConnectionData> for WireguardConnectionData {
    type Error = ConversionError;

    fn try_from(value: ProtoWireguardConnectionData) -> Result<Self, Self::Error> {
        Ok(Self {
            entry: WireguardNode::try_from(
                value
                    .entry
                    .ok_or(ConversionError::NoValueSet("WireguardConnectionData.entry"))?,
            )?,
            exit: WireguardNode::try_from(
                value
                    .exit
                    .ok_or(ConversionError::NoValueSet("WireguardConnectionData.exit"))?,
            )?,
        })
    }
}

impl TryFrom<ProtoWireguardNode> for WireguardNode {
    type Error = ConversionError;

    fn try_from(value: ProtoWireguardNode) -> Result<Self, Self::Error> {
        Ok(Self {
            endpoint: SocketAddr::from_str(&value.endpoint)
                .map_err(|e| ConversionError::ParseAddr("WireguardNode.endpoint", e))?,
            public_key: value.public_key,
            private_ipv4: Ipv4Addr::from_str(&value.private_ipv4)
                .map_err(|e| ConversionError::ParseAddr("WireguardNode.private_ipv4", e))?,
            private_ipv6: Ipv6Addr::from_str(&value.private_ipv6)
                .map_err(|e| ConversionError::ParseAddr("WireguardNode.private_ipv6", e))?,
        })
    }
}

impl From<ProtoGateway> for Gateway {
    fn from(value: ProtoGateway) -> Self {
        Self::new(value.id)
    }
}

impl From<ProtoAddress> for NymAddress {
    fn from(value: ProtoAddress) -> Self {
        Self::new(value.nym_address, value.gateway_id)
    }
}
