// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use prost_types::Timestamp;
use time::OffsetDateTime;

use nym_vpn_lib::tunnel_state_machine::{
    ActionAfterDisconnect, ConnectionData, ErrorStateReason, MixnetConnectionData,
    TunnelConnectionData, TunnelState, WireguardConnectionData, WireguardNode,
};
use nym_vpn_proto::{
    tunnel_connection_data::{
        Mixnet as ProtoMixnetConnectionDataVariant, State as ProtoTunnelConnectionDataState,
        Wireguard as ProtoWireguardConnectionDataVariant,
    },
    tunnel_state::{
        Connected as ProtoConnected, Connecting as ProtoConnecting,
        Disconnected as ProtoDisconnected, Disconnecting as ProtoDisconnecting,
        Error as ProtoError, Offline as ProtoOffline, State as ProtoState,
    },
    ActionAfterDisconnect as ProtoActionAfterDisconnect, Address,
    ConnectionData as ProtoConnectionData, ErrorStateReason as ProtoErrorStateReason, Gateway,
    MixnetConnectionData as ProtoMixnetConnectionData,
    TunnelConnectionData as ProtoTunnelConnectionData, TunnelState as ProtoTunnelState,
    WireguardConnectionData as ProtoWireguardConnectionData, WireguardNode as ProtoWireguardNode,
};

use super::IntoProtobuf;

impl IntoProtobuf for OffsetDateTime {
    type ProtobufType = Timestamp;

    fn to_protobuf(self) -> Timestamp {
        Timestamp {
            seconds: self.unix_timestamp(),
            nanos: self.nanosecond() as i32,
        }
    }
}

impl IntoProtobuf for ActionAfterDisconnect {
    type ProtobufType = ProtoActionAfterDisconnect;

    fn to_protobuf(self) -> ProtoActionAfterDisconnect {
        match self {
            Self::Error => ProtoActionAfterDisconnect::Error,
            Self::Nothing => ProtoActionAfterDisconnect::Nothing,
            Self::Offline => ProtoActionAfterDisconnect::Offline,
            Self::Reconnect => ProtoActionAfterDisconnect::Reconnect,
        }
    }
}

impl IntoProtobuf for ErrorStateReason {
    type ProtobufType = ProtoErrorStateReason;

    fn to_protobuf(self) -> ProtoErrorStateReason {
        match self {
            Self::Firewall => ProtoErrorStateReason::Firewall,
            Self::Routing => ProtoErrorStateReason::Routing,
            Self::Dns => ProtoErrorStateReason::Dns,
            Self::TunDevice => ProtoErrorStateReason::TunDevice,
            Self::TunnelProvider => ProtoErrorStateReason::TunnelProvider,
            Self::SameEntryAndExitGateway => ProtoErrorStateReason::SameEntryAndExitGateway,
            Self::InvalidEntryGatewayCountry => ProtoErrorStateReason::InvalidEntryGatewayCountry,
            Self::InvalidExitGatewayCountry => ProtoErrorStateReason::InvalidExitGatewayCountry,
            Self::BadBandwidthIncrease => ProtoErrorStateReason::BadBandwidthIncrease,
            Self::DuplicateTunFd => ProtoErrorStateReason::DuplicateTunFd,
            Self::Internal => ProtoErrorStateReason::Internal,
        }
    }
}

impl IntoProtobuf for WireguardNode {
    type ProtobufType = ProtoWireguardNode;

    fn to_protobuf(self) -> ProtoWireguardNode {
        ProtoWireguardNode {
            public_key: self.public_key.to_base64(),
            endpoint: self.endpoint.to_string(),
            private_ipv4: self.private_ipv4.to_string(),
            private_ipv6: self.private_ipv6.to_string(),
        }
    }
}

impl IntoProtobuf for ConnectionData {
    type ProtobufType = ProtoConnectionData;

    fn to_protobuf(self) -> ProtoConnectionData {
        ProtoConnectionData {
            entry_gateway: Some(Gateway {
                id: self.entry_gateway.to_string(),
            }),
            exit_gateway: Some(Gateway {
                id: self.exit_gateway.to_string(),
            }),
            connected_at: self.connected_at.map(|x| x.to_protobuf()),
            tunnel: Some(self.tunnel.to_protobuf()),
        }
    }
}

impl IntoProtobuf for MixnetConnectionData {
    type ProtobufType = ProtoMixnetConnectionData;

    fn to_protobuf(self) -> ProtoMixnetConnectionData {
        ProtoMixnetConnectionData {
            nym_address: Some(Address {
                nym_address: self.nym_address.to_string(),
            }),
            exit_ipr: Some(Address {
                nym_address: self.exit_ipr.to_string(),
            }),
            ipv4: self.ipv4.to_string(),
            ipv6: self.ipv6.to_string(),
        }
    }
}

impl IntoProtobuf for WireguardConnectionData {
    type ProtobufType = ProtoWireguardConnectionData;

    fn to_protobuf(self) -> ProtoWireguardConnectionData {
        ProtoWireguardConnectionData {
            entry: Some(self.entry.to_protobuf()),
            exit: Some(self.exit.to_protobuf()),
        }
    }
}

impl IntoProtobuf for TunnelConnectionData {
    type ProtobufType = ProtoTunnelConnectionData;

    fn to_protobuf(self) -> ProtoTunnelConnectionData {
        let state = match self {
            TunnelConnectionData::Mixnet(data) => {
                ProtoTunnelConnectionDataState::Mixnet(ProtoMixnetConnectionDataVariant {
                    data: Some(data.to_protobuf()),
                })
            }
            TunnelConnectionData::Wireguard(data) => {
                ProtoTunnelConnectionDataState::Wireguard(ProtoWireguardConnectionDataVariant {
                    data: Some(data.to_protobuf()),
                })
            }
        };

        ProtoTunnelConnectionData { state: Some(state) }
    }
}

impl IntoProtobuf for TunnelState {
    type ProtobufType = ProtoTunnelState;

    fn to_protobuf(self) -> Self::ProtobufType {
        let proto_state: ProtoState = match self {
            TunnelState::Disconnected => ProtoState::Disconnected(ProtoDisconnected {}),
            TunnelState::Connecting { connection_data } => {
                ProtoState::Connecting(ProtoConnecting {
                    connection_data: connection_data.map(|x| x.to_protobuf()),
                })
            }
            TunnelState::Connected { connection_data } => ProtoState::Connected(ProtoConnected {
                connection_data: Some(connection_data.to_protobuf()),
            }),
            TunnelState::Disconnecting { after_disconnect } => {
                ProtoState::Disconnecting(ProtoDisconnecting {
                    after_disconnect: after_disconnect.to_protobuf() as i32,
                })
            }
            TunnelState::Offline { reconnect } => ProtoState::Offline(ProtoOffline { reconnect }),
            TunnelState::Error(reason) => ProtoState::Error(ProtoError {
                reason: reason.to_protobuf() as i32,
            }),
        };

        ProtoTunnelState {
            state: Some(proto_state),
        }
    }
}
