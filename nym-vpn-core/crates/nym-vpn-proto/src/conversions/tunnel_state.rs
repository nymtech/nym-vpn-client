// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
};

use nym_vpn_lib_types::{
    ActionAfterDisconnect, BridgeAddress, ConnectionData, ErrorStateReason,
    EstablishConnectionData, EstablishConnectionState, GatewayId, GatewayLightInfo,
    MixnetConnectionData, NymAddress, TunnelConnectionData, TunnelState, TunnelType,
    WireguardConnectionData, WireguardNode,
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

impl TryFrom<proto::tunnel_state::Error> for ErrorStateReason {
    type Error = ConversionError;

    fn try_from(value: proto::tunnel_state::Error) -> Result<Self, ConversionError> {
        use proto::tunnel_state::ErrorStateReason as Reason;
        let reason = proto::tunnel_state::ErrorStateReason::try_from(value.reason)
            .map_err(|_| ConversionError::NoValueSet("tunnel_state::Error::reason"))?;

        Ok(match reason {
            Reason::SetFirewallPolicy => Self::SetFirewallPolicy,
            Reason::SetRouting => Self::SetRouting,
            Reason::SetDns => Self::SetDns,
            Reason::TunDevice => Self::TunDevice,
            Reason::TunnelProvider => Self::TunnelProvider,
            Reason::Ipv6Unavailable => Self::Ipv6Unavailable,
            Reason::SameEntryAndExitGateway => Self::SameEntryAndExitGateway,
            Reason::PerformantEntryGatewayUnavailable => Self::PerformantEntryGatewayUnavailable,
            Reason::PerformantExitGatewayUnavailable => Self::PerformantExitGatewayUnavailable,
            Reason::InvalidEntryGatewayIdentity => Self::InvalidEntryGatewayIdentity,
            Reason::InvalidExitGatewayIdentity => Self::InvalidExitGatewayIdentity,
            Reason::InvalidEntryGatewayCountry => Self::InvalidEntryGatewayCountry,
            Reason::InvalidExitGatewayCountry => Self::InvalidExitGatewayCountry,
            Reason::CredentialWastedOnEntryGateway => Self::CredentialWastedOnEntryGateway,
            Reason::CredentialWastedOnExitGateway => Self::CredentialWastedOnExitGateway,
            Reason::BandwidthExceeded => Self::BandwidthExceeded,
            Reason::CredentialFetchingFailed => Self::CredentialFetchingFailed,
            Reason::NoCredentialAvailable => Self::NoCredentialAvailable,
            Reason::InactiveAccount => Self::InactiveAccount,
            Reason::InactiveSubscription => Self::InactiveSubscription,
            Reason::MaxDevicesReached => Self::MaxDevicesReached,
            Reason::DeviceTimeOutOfSync => Self::DeviceTimeOutOfSync,
            Reason::DeviceLoggedOut => Self::DeviceLoggedOut,
            Reason::NeedFullDiskPermissions => Self::NeedFullDiskPermissions,
            Reason::SplitTunnel => Self::SplitTunnel,
            Reason::NeedsRelaxedIndependenceCriteria => Self::NeedsRelaxedIndependenceCriteria,
            Reason::NeedsDeviceLocation => Self::NeedsDeviceLocation,
            Reason::Internal => Self::Internal(value.message.unwrap_or_default()),
        })
    }
}

impl From<ErrorStateReason> for proto::tunnel_state::Error {
    fn from(value: ErrorStateReason) -> Self {
        use proto::tunnel_state::ErrorStateReason as Reason;

        match value {
            ErrorStateReason::SetFirewallPolicy => Self {
                reason: Reason::SetFirewallPolicy.into(),
                message: None,
            },
            ErrorStateReason::SetRouting => Self {
                reason: Reason::SetRouting.into(),
                message: None,
            },
            ErrorStateReason::SetDns => Self {
                reason: Reason::SetDns.into(),
                message: None,
            },
            ErrorStateReason::TunDevice => Self {
                reason: Reason::TunDevice.into(),
                message: None,
            },
            ErrorStateReason::TunnelProvider => Self {
                reason: Reason::TunnelProvider.into(),
                message: None,
            },
            ErrorStateReason::Ipv6Unavailable => Self {
                reason: Reason::Ipv6Unavailable.into(),
                message: None,
            },
            ErrorStateReason::SameEntryAndExitGateway => Self {
                reason: Reason::SameEntryAndExitGateway.into(),
                message: None,
            },
            ErrorStateReason::PerformantEntryGatewayUnavailable => Self {
                reason: Reason::PerformantEntryGatewayUnavailable.into(),
                message: None,
            },
            ErrorStateReason::PerformantExitGatewayUnavailable => Self {
                reason: Reason::PerformantExitGatewayUnavailable.into(),
                message: None,
            },
            ErrorStateReason::InvalidEntryGatewayIdentity => Self {
                reason: Reason::InvalidEntryGatewayIdentity.into(),
                message: None,
            },
            ErrorStateReason::InvalidExitGatewayIdentity => Self {
                reason: Reason::InvalidExitGatewayIdentity.into(),
                message: None,
            },
            ErrorStateReason::InvalidEntryGatewayCountry => Self {
                reason: Reason::InvalidEntryGatewayCountry.into(),
                message: None,
            },
            ErrorStateReason::InvalidExitGatewayCountry => Self {
                reason: Reason::InvalidExitGatewayCountry.into(),
                message: None,
            },
            ErrorStateReason::CredentialWastedOnEntryGateway => Self {
                reason: Reason::CredentialWastedOnEntryGateway.into(),
                message: None,
            },
            ErrorStateReason::CredentialWastedOnExitGateway => Self {
                reason: Reason::CredentialWastedOnExitGateway.into(),
                message: None,
            },
            ErrorStateReason::BandwidthExceeded => Self {
                reason: Reason::BandwidthExceeded.into(),
                message: None,
            },
            ErrorStateReason::CredentialFetchingFailed => Self {
                reason: Reason::CredentialFetchingFailed.into(),
                message: None,
            },
            ErrorStateReason::NoCredentialAvailable => Self {
                reason: Reason::NoCredentialAvailable.into(),
                message: None,
            },
            ErrorStateReason::InactiveAccount => Self {
                reason: Reason::InactiveAccount.into(),
                message: None,
            },
            ErrorStateReason::InactiveSubscription => Self {
                reason: Reason::InactiveSubscription.into(),
                message: None,
            },
            ErrorStateReason::MaxDevicesReached => Self {
                reason: Reason::MaxDevicesReached.into(),
                message: None,
            },
            ErrorStateReason::DeviceTimeOutOfSync => Self {
                reason: Reason::DeviceTimeOutOfSync.into(),
                message: None,
            },
            ErrorStateReason::DeviceLoggedOut => Self {
                reason: Reason::DeviceLoggedOut.into(),
                message: None,
            },
            ErrorStateReason::NeedFullDiskPermissions => Self {
                reason: Reason::NeedFullDiskPermissions.into(),
                message: None,
            },
            ErrorStateReason::SplitTunnel => Self {
                reason: Reason::SplitTunnel.into(),
                message: None,
            },
            ErrorStateReason::NeedsRelaxedIndependenceCriteria => Self {
                reason: Reason::NeedsRelaxedIndependenceCriteria.into(),
                message: None,
            },
            ErrorStateReason::NeedsDeviceLocation => Self {
                reason: Reason::NeedsDeviceLocation.into(),
                message: None,
            },
            ErrorStateReason::Internal(message) => Self {
                reason: Reason::Internal.into(),
                message: Some(message),
            },
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
                state,
                tunnel_type,
                connection_data,
            }) => {
                let connection_data = connection_data
                    .map(EstablishConnectionData::try_from)
                    .transpose()?;
                let state = proto::EstablishConnectionState::try_from(state)
                    .map_err(|e| ConversionError::Decode("EstablishConnectionState", e))
                    .map(EstablishConnectionState::from)?;
                let tunnel_type = proto::TunnelType::try_from(tunnel_type)
                    .map_err(|e| ConversionError::Decode("TunnelType", e))
                    .map(TunnelType::from)?;

                Self::Connecting {
                    retry_attempt,
                    state,
                    tunnel_type,
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
            proto::tunnel_state::State::Error(err) => Self::Error(ErrorStateReason::try_from(err)?),
            proto::tunnel_state::State::Offline(proto::tunnel_state::Offline { reconnect }) => {
                Self::Offline { reconnect }
            }
        })
    }
}

impl From<proto::TunnelType> for TunnelType {
    fn from(value: proto::TunnelType) -> Self {
        match value {
            proto::TunnelType::Mixnet => Self::Mixnet,
            proto::TunnelType::Wireguard => Self::Wireguard,
        }
    }
}

impl From<TunnelType> for proto::TunnelType {
    fn from(value: TunnelType) -> Self {
        match value {
            TunnelType::Mixnet => Self::Mixnet,
            TunnelType::Wireguard => Self::Wireguard,
        }
    }
}

impl From<proto::EstablishConnectionState> for EstablishConnectionState {
    fn from(value: proto::EstablishConnectionState) -> Self {
        match value {
            proto::EstablishConnectionState::AwaitingAccountReadiness => {
                Self::AwaitingAccountReadiness
            }
            proto::EstablishConnectionState::AwaitingCredentialsAvailability => {
                Self::AwaitingCredentialsAvailability
            }
            proto::EstablishConnectionState::ResolvingApiAddresses => Self::ResolvingApiAddresses,
            proto::EstablishConnectionState::RefreshingGateways => Self::RefreshingGateways,
            proto::EstablishConnectionState::SelectingGateways => Self::SelectingGateways,
            proto::EstablishConnectionState::RegisteringWithGateways => {
                Self::RegisteringWithGateways
            }
            proto::EstablishConnectionState::ConnectingTunnel => Self::ConnectingTunnel,
        }
    }
}

impl From<EstablishConnectionState> for proto::EstablishConnectionState {
    fn from(value: EstablishConnectionState) -> Self {
        match value {
            EstablishConnectionState::AwaitingAccountReadiness => Self::AwaitingAccountReadiness,
            EstablishConnectionState::AwaitingCredentialsAvailability => {
                Self::AwaitingCredentialsAvailability
            }
            EstablishConnectionState::ResolvingApiAddresses => Self::ResolvingApiAddresses,
            EstablishConnectionState::RefreshingGateways => Self::RefreshingGateways,
            EstablishConnectionState::SelectingGateways => Self::SelectingGateways,
            EstablishConnectionState::RegisteringWithGateways => Self::RegisteringWithGateways,
            EstablishConnectionState::ConnectingTunnel => Self::ConnectingTunnel,
        }
    }
}

impl TryFrom<proto::EstablishConnectionData> for EstablishConnectionData {
    type Error = ConversionError;

    fn try_from(value: proto::EstablishConnectionData) -> Result<Self, Self::Error> {
        let entry_gateway =
            value
                .entry_gateway
                .map(GatewayLightInfo::from)
                .ok_or(ConversionError::NoValueSet(
                    "EstablishingConnectionData.entry_gateway",
                ))?;

        let exit_gateway =
            value
                .exit_gateway
                .map(GatewayLightInfo::from)
                .ok_or(ConversionError::NoValueSet(
                    "EstablishingConnectionData.exit_gateway",
                ))?;

        let tunnel = value
            .tunnel
            .map(TunnelConnectionData::try_from)
            .transpose()?;

        Ok(Self {
            entry_gateway,
            exit_gateway,
            tunnel,
        })
    }
}

impl TryFrom<proto::ConnectionData> for ConnectionData {
    type Error = ConversionError;

    fn try_from(value: proto::ConnectionData) -> Result<Self, Self::Error> {
        let connected_at = value
            .connected_at
            .ok_or(ConversionError::NoValueSet("ConnectionData.connected_at"))?;

        let connected_at =
            crate::conversions::prost::prost_timestamp_into_offset_datetime(connected_at)
                .map_err(|e| ConversionError::ConvertTime("ConnectionData.connected_at", e))?;

        let tunnel_connection_data = value
            .tunnel
            .ok_or(ConversionError::NoValueSet("ConnectionData.tunnel"))?;

        Ok(Self {
            connected_at,
            entry_gateway: value
                .entry_gateway
                .map(GatewayLightInfo::from)
                .ok_or(ConversionError::NoValueSet("ConnectionData.entry_gateway"))?,
            exit_gateway: value
                .exit_gateway
                .map(GatewayLightInfo::from)
                .ok_or(ConversionError::NoValueSet("ConnectionData.exit_gateway"))?,
            tunnel: TunnelConnectionData::try_from(tunnel_connection_data)?,
        })
    }
}

impl From<EstablishConnectionData> for proto::EstablishConnectionData {
    fn from(value: EstablishConnectionData) -> Self {
        proto::EstablishConnectionData {
            entry_gateway: Some(proto::GatewayLightInfo::from(value.entry_gateway)),
            exit_gateway: Some(proto::GatewayLightInfo::from(value.exit_gateway)),
            tunnel: value.tunnel.map(proto::TunnelConnectionData::from),
        }
    }
}

impl From<ConnectionData> for proto::ConnectionData {
    fn from(value: ConnectionData) -> proto::ConnectionData {
        proto::ConnectionData {
            entry_gateway: Some(proto::GatewayLightInfo::from(value.entry_gateway)),
            exit_gateway: Some(proto::GatewayLightInfo::from(value.exit_gateway)),
            connected_at: Some(
                crate::conversions::prost::offset_datetime_into_proto_timestamp(value.connected_at),
            ),
            tunnel: Some(proto::TunnelConnectionData::from(value.tunnel)),
        }
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

impl From<TunnelConnectionData> for proto::TunnelConnectionData {
    fn from(value: TunnelConnectionData) -> proto::TunnelConnectionData {
        let state = match value {
            TunnelConnectionData::Mixnet(data) => proto::tunnel_connection_data::State::Mixnet(
                proto::tunnel_connection_data::Mixnet {
                    data: Some(proto::MixnetConnectionData::from(data)),
                },
            ),
            TunnelConnectionData::Wireguard(data) => {
                proto::tunnel_connection_data::State::Wireguard(
                    proto::tunnel_connection_data::Wireguard {
                        data: Some(proto::WireguardConnectionData::from(data)),
                    },
                )
            }
        };

        proto::TunnelConnectionData { state: Some(state) }
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
            ipv6: value
                .ipv6
                .as_deref()
                .map(|ipv6| {
                    Ipv6Addr::from_str(ipv6)
                        .map_err(|e| ConversionError::ParseAddr("MixnetConnectionData.ipv6", e))
                })
                .transpose()?,
        })
    }
}

impl From<MixnetConnectionData> for proto::MixnetConnectionData {
    fn from(value: MixnetConnectionData) -> proto::MixnetConnectionData {
        proto::MixnetConnectionData {
            nym_address: Some(proto::Address {
                nym_address: value.nym_address.to_string(),
                gateway_id: value.nym_address.gateway_id,
            }),
            exit_ipr: Some(proto::Address {
                nym_address: value.exit_ipr.to_string(),
                gateway_id: value.exit_ipr.gateway_id,
            }),
            entry_ip: value.entry_ip.to_string(),
            exit_ip: value.exit_ip.to_string(),
            ipv4: value.ipv4.to_string(),
            ipv6: value.ipv6.map(|ipv6| ipv6.to_string()),
        }
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
            entry_bridge_addr: value
                .entry_bridge_addr
                .map(BridgeAddress::try_from)
                .transpose()?,
        })
    }
}

impl TryFrom<proto::BridgeAddress> for BridgeAddress {
    type Error = ConversionError;

    fn try_from(value: proto::BridgeAddress) -> Result<Self, Self::Error> {
        let listen_addr = value
            .listen_addr
            .ok_or(ConversionError::NoValueSet("BridgeAddress.local_addr"))?;
        let remote_addr = value
            .remote_addr
            .ok_or(ConversionError::NoValueSet("BridgeAddress.remote_addr"))?;
        Ok(Self {
            listen_addr: SocketAddr::try_from(listen_addr)?,
            remote_addr: SocketAddr::try_from(remote_addr)?,
        })
    }
}

impl From<BridgeAddress> for proto::BridgeAddress {
    fn from(value: BridgeAddress) -> Self {
        Self {
            listen_addr: Some(proto::SocketAddr::from(value.listen_addr)),
            remote_addr: Some(proto::SocketAddr::from(value.remote_addr)),
        }
    }
}

impl From<WireguardConnectionData> for proto::WireguardConnectionData {
    fn from(value: WireguardConnectionData) -> proto::WireguardConnectionData {
        proto::WireguardConnectionData {
            entry: Some(proto::WireguardNode::from(value.entry)),
            exit: Some(proto::WireguardNode::from(value.exit)),
            entry_bridge_addr: value.entry_bridge_addr.map(proto::BridgeAddress::from),
        }
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
            private_ipv6: value
                .private_ipv6
                .as_deref()
                .map(|private_ipv6| {
                    Ipv6Addr::from_str(private_ipv6)
                        .map_err(|e| ConversionError::ParseAddr("WireguardNode.private_ipv6", e))
                })
                .transpose()?,
        })
    }
}

impl From<WireguardNode> for proto::WireguardNode {
    fn from(value: WireguardNode) -> proto::WireguardNode {
        proto::WireguardNode {
            public_key: value.public_key,
            endpoint: value.endpoint.to_string(),
            private_ipv4: value.private_ipv4.to_string(),
            private_ipv6: value
                .private_ipv6
                .as_ref()
                .map(|private_ipv6| private_ipv6.to_string()),
        }
    }
}

impl From<proto::GatewayId> for GatewayId {
    fn from(value: proto::GatewayId) -> Self {
        Self::new(value.id)
    }
}

impl From<proto::GatewayLightInfo> for GatewayLightInfo {
    fn from(value: proto::GatewayLightInfo) -> Self {
        Self::new(value.id, value.country_code)
    }
}

impl From<GatewayLightInfo> for proto::GatewayLightInfo {
    fn from(value: GatewayLightInfo) -> Self {
        Self {
            id: value.id,
            country_code: value.country_code,
        }
    }
}

impl From<proto::Address> for NymAddress {
    fn from(value: proto::Address) -> Self {
        Self::new(value.nym_address, value.gateway_id)
    }
}

impl From<ActionAfterDisconnect> for proto::tunnel_state::ActionAfterDisconnect {
    fn from(value: ActionAfterDisconnect) -> Self {
        match value {
            ActionAfterDisconnect::Error => Self::Error,
            ActionAfterDisconnect::Nothing => Self::Nothing,
            ActionAfterDisconnect::Offline => Self::Offline,
            ActionAfterDisconnect::Reconnect => Self::Reconnect,
        }
    }
}

impl From<TunnelState> for proto::TunnelState {
    fn from(value: TunnelState) -> proto::TunnelState {
        let proto_state: proto::tunnel_state::State = match value {
            TunnelState::Disconnected => {
                proto::tunnel_state::State::Disconnected(proto::tunnel_state::Disconnected {})
            }
            TunnelState::Connecting {
                retry_attempt,
                state,
                tunnel_type,
                connection_data,
            } => proto::tunnel_state::State::Connecting(proto::tunnel_state::Connecting {
                retry_attempt,
                state: proto::EstablishConnectionState::from(state) as i32,
                tunnel_type: proto::TunnelType::from(tunnel_type) as i32,
                connection_data: connection_data.map(proto::EstablishConnectionData::from),
            }),
            TunnelState::Connected { connection_data } => {
                proto::tunnel_state::State::Connected(proto::tunnel_state::Connected {
                    connection_data: Some(proto::ConnectionData::from(connection_data)),
                })
            }
            TunnelState::Disconnecting { after_disconnect } => {
                proto::tunnel_state::State::Disconnecting(proto::tunnel_state::Disconnecting {
                    after_disconnect: proto::tunnel_state::ActionAfterDisconnect::from(
                        after_disconnect,
                    ) as i32,
                })
            }
            TunnelState::Offline { reconnect } => {
                proto::tunnel_state::State::Offline(proto::tunnel_state::Offline { reconnect })
            }
            TunnelState::Error(reason) => {
                proto::tunnel_state::State::Error(proto::tunnel_state::Error::from(reason))
            }
        };

        proto::TunnelState {
            state: Some(proto_state),
        }
    }
}

impl From<GatewayId> for proto::GatewayId {
    fn from(value: GatewayId) -> Self {
        Self { id: value.id }
    }
}
