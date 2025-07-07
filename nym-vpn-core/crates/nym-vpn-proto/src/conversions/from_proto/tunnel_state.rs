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

use crate::{conversions::ConversionError, proto};

impl From<proto::tunnel_state::ActionAfterDisconnect> for ActionAfterDisconnect {
    fn from(value: proto::tunnel_state::ActionAfterDisconnect) -> Self {
        match value {
            proto::tunnel_state::ActionAfterDisconnect::Error => Self::Error,
            proto::tunnel_state::ActionAfterDisconnect::Nothing => Self::Nothing,
            proto::tunnel_state::ActionAfterDisconnect::Offline => Self::Offline,
            proto::tunnel_state::ActionAfterDisconnect::Reconnect => Self::Reconnect,
        }
    }
}

impl From<proto::tunnel_state::Error> for ClientErrorReason {
    fn from(value: proto::tunnel_state::Error) -> Self {
        match value.reason() {
            proto::tunnel_state::ErrorStateReason::Firewall => ClientErrorReason::Firewall,
            proto::tunnel_state::ErrorStateReason::Routing => ClientErrorReason::Routing,
            proto::tunnel_state::ErrorStateReason::SameEntryAndExitGateway => {
                ClientErrorReason::SameEntryAndExitGateway
            }
            proto::tunnel_state::ErrorStateReason::InvalidEntryGatewayCountry => {
                ClientErrorReason::InvalidEntryGatewayCountry
            }
            proto::tunnel_state::ErrorStateReason::InvalidExitGatewayCountry => {
                ClientErrorReason::InvalidExitGatewayCountry
            }
            proto::tunnel_state::ErrorStateReason::MaxDevicesReached => {
                ClientErrorReason::MaxDevicesReached
            }
            proto::tunnel_state::ErrorStateReason::BandwidthExceeded => {
                ClientErrorReason::BandwidthExceeded
            }
            proto::tunnel_state::ErrorStateReason::SubscriptionExpired => {
                ClientErrorReason::SubscriptionExpired
            }
            proto::tunnel_state::ErrorStateReason::Dns => ClientErrorReason::Dns(value.detail),
            proto::tunnel_state::ErrorStateReason::Api => ClientErrorReason::Api(value.detail),
            proto::tunnel_state::ErrorStateReason::DeviceTimeOutOfSync => {
                ClientErrorReason::DeviceTimeOutOfSync
            }
            proto::tunnel_state::ErrorStateReason::CreateMixnetStorage => {
                ClientErrorReason::CreateMixnetStorage
            }
            proto::tunnel_state::ErrorStateReason::Internal => {
                ClientErrorReason::Internal(value.detail)
            }
        }
    }
}

impl TryFrom<proto::TunnelState> for TunnelState {
    type Error = ConversionError;

    fn try_from(value: proto::TunnelState) -> Result<Self, ConversionError> {
        let state = value
            .state
            .ok_or(ConversionError::NoValueSet("TunnelState.state"))?;

        Ok(match state {
            proto::tunnel_state::State::Disconnected(proto::tunnel_state::Disconnected {}) => {
                Self::Disconnected
            }
            proto::tunnel_state::State::Disconnecting(proto::tunnel_state::Disconnecting {
                after_disconnect,
            }) => {
                let proto_after_disconnect =
                    proto::tunnel_state::ActionAfterDisconnect::try_from(after_disconnect)
                        .map_err(|e| ConversionError::Decode("TunnelState.after_disconnect", e))?;

                Self::Disconnecting {
                    after_disconnect: ActionAfterDisconnect::from(proto_after_disconnect),
                }
            }
            proto::tunnel_state::State::Connecting(proto::tunnel_state::Connecting {
                retry_attempt,
                connection_data,
            }) => {
                let connection_data = connection_data.map(ConnectionData::try_from).transpose()?;

                Self::Connecting {
                    retry_attempt,
                    connection_data,
                }
            }
            proto::tunnel_state::State::Connected(proto::tunnel_state::Connected {
                connection_data,
            }) => {
                let connection_data = connection_data
                    .ok_or(ConversionError::NoValueSet("TunnelState.connection_data"))
                    .and_then(ConnectionData::try_from)?;

                Self::Connected { connection_data }
            }
            proto::tunnel_state::State::Error(error_state_reason) => {
                Self::Error(error_state_reason.into())
            }
            proto::tunnel_state::State::Offline(proto::tunnel_state::Offline { reconnect }) => {
                Self::Offline { reconnect }
            }
        })
    }
}

impl TryFrom<proto::ConnectionData> for ConnectionData {
    type Error = ConversionError;

    fn try_from(value: proto::ConnectionData) -> Result<Self, Self::Error> {
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

impl TryFrom<proto::TunnelConnectionData> for TunnelConnectionData {
    type Error = ConversionError;

    fn try_from(value: proto::TunnelConnectionData) -> Result<Self, Self::Error> {
        let state = value
            .state
            .ok_or(ConversionError::NoValueSet("TunnelConnectionData.state"))?;

        Ok(match state {
            proto::tunnel_connection_data::State::Mixnet(
                proto::tunnel_connection_data::Mixnet { data },
            ) => Self::Mixnet(MixnetConnectionData::try_from(data.ok_or(
                ConversionError::NoValueSet("TunnelConnectionData::Mixnet.data"),
            )?)?),
            proto::tunnel_connection_data::State::Wireguard(
                proto::tunnel_connection_data::Wireguard { data },
            ) => Self::Wireguard(WireguardConnectionData::try_from(data.ok_or(
                ConversionError::NoValueSet("TunnelConnectionData::Wireguard.data"),
            )?)?),
        })
    }
}

impl TryFrom<proto::MixnetConnectionData> for MixnetConnectionData {
    type Error = ConversionError;

    fn try_from(value: proto::MixnetConnectionData) -> Result<Self, Self::Error> {
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

impl TryFrom<proto::WireguardConnectionData> for WireguardConnectionData {
    type Error = ConversionError;

    fn try_from(value: proto::WireguardConnectionData) -> Result<Self, Self::Error> {
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

impl TryFrom<proto::WireguardNode> for WireguardNode {
    type Error = ConversionError;

    fn try_from(value: proto::WireguardNode) -> Result<Self, Self::Error> {
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

impl From<proto::Gateway> for Gateway {
    fn from(value: proto::Gateway) -> Self {
        Self::new(value.id)
    }
}

impl From<proto::Address> for NymAddress {
    fn from(value: proto::Address) -> Self {
        Self::new(value.nym_address, value.gateway_id)
    }
}
