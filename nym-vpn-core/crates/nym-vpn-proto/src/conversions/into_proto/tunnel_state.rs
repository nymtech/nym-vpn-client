// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    ActionAfterDisconnect, ConnectionData, ErrorStateReason, MixnetConnectionData,
    TunnelConnectionData, TunnelState, WireguardConnectionData, WireguardNode,
};

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
    Address as ProtoAddress, ConnectionData as ProtoConnectionData, Gateway as ProtoGateway,
    MixnetConnectionData as ProtoMixnetConnectionData,
    TunnelConnectionData as ProtoTunnelConnectionData, TunnelState as ProtoTunnelState,
    WireguardConnectionData as ProtoWireguardConnectionData, WireguardNode as ProtoWireguardNode,
};

impl From<ActionAfterDisconnect> for ProtoActionAfterDisconnect {
    fn from(value: ActionAfterDisconnect) -> Self {
        match value {
            ActionAfterDisconnect::Error => Self::Error,
            ActionAfterDisconnect::Nothing => Self::Nothing,
            ActionAfterDisconnect::Offline => Self::Offline,
            ActionAfterDisconnect::Reconnect => Self::Reconnect,
        }
    }
}

impl From<ErrorStateReason> for ProtoErrorStateReason {
    fn from(value: ErrorStateReason) -> Self {
        match value {
            ErrorStateReason::Firewall => Self::Firewall,
            ErrorStateReason::Routing => Self::Routing,
            ErrorStateReason::Dns => Self::Dns,
            ErrorStateReason::TunDevice => Self::TunDevice,
            ErrorStateReason::TunnelProvider => Self::TunnelProvider,
            ErrorStateReason::SameEntryAndExitGateway => Self::SameEntryAndExitGateway,
            ErrorStateReason::InvalidEntryGatewayCountry => Self::InvalidEntryGatewayCountry,
            ErrorStateReason::InvalidExitGatewayCountry => Self::InvalidExitGatewayCountry,
            ErrorStateReason::BadBandwidthIncrease => Self::BadBandwidthIncrease,
            ErrorStateReason::DuplicateTunFd => Self::DuplicateTunFd,
            ErrorStateReason::Internal => Self::Internal,
        }
    }
}

impl From<TunnelState> for ProtoTunnelState {
    fn from(value: TunnelState) -> ProtoTunnelState {
        let proto_state: ProtoState = match value {
            TunnelState::Disconnected => ProtoState::Disconnected(ProtoDisconnected {}),
            TunnelState::Connecting { connection_data } => {
                ProtoState::Connecting(ProtoConnecting {
                    connection_data: connection_data.map(ProtoConnectionData::from),
                })
            }
            TunnelState::Connected { connection_data } => ProtoState::Connected(ProtoConnected {
                connection_data: Some(ProtoConnectionData::from(connection_data)),
            }),
            TunnelState::Disconnecting { after_disconnect } => {
                ProtoState::Disconnecting(ProtoDisconnecting {
                    after_disconnect: ProtoActionAfterDisconnect::from(after_disconnect) as i32,
                })
            }
            TunnelState::Offline { reconnect } => ProtoState::Offline(ProtoOffline { reconnect }),
            TunnelState::Error(reason) => ProtoState::Error(ProtoError {
                reason: ProtoErrorStateReason::from(reason) as i32,
            }),
        };

        ProtoTunnelState {
            state: Some(proto_state),
        }
    }
}

impl From<WireguardNode> for ProtoWireguardNode {
    fn from(value: WireguardNode) -> ProtoWireguardNode {
        ProtoWireguardNode {
            public_key: value.public_key.to_base64(),
            endpoint: value.endpoint.to_string(),
            private_ipv4: value.private_ipv4.to_string(),
            private_ipv6: value.private_ipv6.to_string(),
        }
    }
}

impl From<ConnectionData> for ProtoConnectionData {
    fn from(value: ConnectionData) -> ProtoConnectionData {
        ProtoConnectionData {
            entry_gateway: Some(ProtoGateway {
                id: value.entry_gateway.to_string(),
            }),
            exit_gateway: Some(ProtoGateway {
                id: value.exit_gateway.to_string(),
            }),
            connected_at: value
                .connected_at
                .map(crate::conversions::prost::offset_datetime_into_proto_timestamp),
            tunnel: Some(ProtoTunnelConnectionData::from(value.tunnel)),
        }
    }
}

impl From<MixnetConnectionData> for ProtoMixnetConnectionData {
    fn from(value: MixnetConnectionData) -> ProtoMixnetConnectionData {
        ProtoMixnetConnectionData {
            nym_address: Some(ProtoAddress {
                nym_address: value.nym_address.to_string(),
            }),
            exit_ipr: Some(ProtoAddress {
                nym_address: value.exit_ipr.to_string(),
            }),
            ipv4: value.ipv4.to_string(),
            ipv6: value.ipv6.to_string(),
        }
    }
}

impl From<WireguardConnectionData> for ProtoWireguardConnectionData {
    fn from(value: WireguardConnectionData) -> ProtoWireguardConnectionData {
        ProtoWireguardConnectionData {
            entry: Some(ProtoWireguardNode::from(value.entry)),
            exit: Some(ProtoWireguardNode::from(value.exit)),
        }
    }
}

impl From<TunnelConnectionData> for ProtoTunnelConnectionData {
    fn from(value: TunnelConnectionData) -> ProtoTunnelConnectionData {
        let state = match value {
            TunnelConnectionData::Mixnet(data) => {
                ProtoTunnelConnectionDataState::Mixnet(ProtoMixnetConnectionDataVariant {
                    data: Some(ProtoMixnetConnectionData::from(data)),
                })
            }
            TunnelConnectionData::Wireguard(data) => {
                ProtoTunnelConnectionDataState::Wireguard(ProtoWireguardConnectionDataVariant {
                    data: Some(ProtoWireguardConnectionData::from(data)),
                })
            }
        };

        ProtoTunnelConnectionData { state: Some(state) }
    }
}
