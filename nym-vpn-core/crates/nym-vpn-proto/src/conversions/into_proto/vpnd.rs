// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::GatewayType;
use nym_sdk::UserAgent;
use nym_vpnd_types::{
    ConnectArgs, ListCountriesOptions, ListGatewaysOptions, gateway::Score, log_path::LogPath,
    service::VpnServiceInfo,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    Address as ProtoAddress, ConnectRequest, Dns as ProtoDns, EntryNode as ProtoEntryNode,
    ExitNode as ProtoExitNode, GatewayType as ProtoGatewayType,
    GetLogPathResponse as ProtoGetLogPathResponse, InfoResponse as ProtoInfoResponse,
    ListCountriesRequest as ProtoListCountriesRequest,
    ListGatewaysRequest as ProtoListGatewaysRequest, NymNetworkDetails as ProtoNymNetworkDetails,
    NymVpnNetworkDetails as ProtoNymVpnNetworkDetails, UserAgent as ProtoUserAgent,
    conversions::ConversionError, exit_node::ExitNodeEnum as ProtoExitNodeEnum,
};

impl From<nym_vpnd_types::gateway::Location> for crate::Location {
    fn from(location: nym_vpnd_types::gateway::Location) -> Self {
        crate::Location {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<Score> for crate::Score {
    fn from(score: Score) -> Self {
        match score {
            Score::High => crate::Score::High,
            Score::Medium => crate::Score::Medium,
            Score::Low => crate::Score::Low,
            Score::None => crate::Score::None,
        }
    }
}

impl From<nym_vpnd_types::gateway::Entry> for crate::AsEntry {
    fn from(entry: nym_vpnd_types::gateway::Entry) -> Self {
        crate::AsEntry {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<nym_vpnd_types::gateway::Exit> for crate::AsExit {
    fn from(exit: nym_vpnd_types::gateway::Exit) -> Self {
        crate::AsExit {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

impl From<nym_vpnd_types::gateway::ProbeOutcome> for crate::ProbeOutcome {
    fn from(outcome: nym_vpnd_types::gateway::ProbeOutcome) -> Self {
        let as_entry = Some(crate::AsEntry::from(outcome.as_entry));
        let as_exit = outcome.as_exit.map(crate::AsExit::from);
        let wg = None;
        crate::ProbeOutcome {
            as_entry,
            as_exit,
            wg,
        }
    }
}

impl From<nym_vpnd_types::gateway::Probe> for crate::Probe {
    fn from(probe: nym_vpnd_types::gateway::Probe) -> Self {
        let last_updated = OffsetDateTime::parse(&probe.last_updated_utc, &Rfc3339).ok();
        let last_updated_utc = last_updated.map(|timestamp| prost_types::Timestamp {
            seconds: timestamp.unix_timestamp(),
            nanos: timestamp.nanosecond() as i32,
        });
        let outcome = Some(crate::ProbeOutcome::from(probe.outcome));
        crate::Probe {
            last_updated_utc,
            outcome,
        }
    }
}

impl From<nym_vpnd_types::gateway::Gateway> for crate::GatewayResponse {
    fn from(gateway: nym_vpnd_types::gateway::Gateway) -> Self {
        let id = Some(crate::Gateway {
            id: gateway.identity_key.to_string(),
        });
        let location = gateway.location.map(crate::Location::from);
        let last_probe = gateway.last_probe.map(crate::Probe::from);
        let moniker = gateway.moniker;
        crate::GatewayResponse {
            id,
            location,
            last_probe,
            wg_score: gateway
                .wg_score
                .map(|score| crate::Score::from(score) as i32),
            mixnet_score: gateway
                .mixnet_score
                .map(|score| crate::Score::from(score) as i32),
            moniker,
        }
    }
}

impl From<nym_vpnd_types::gateway::Country> for crate::Location {
    fn from(country: nym_vpnd_types::gateway::Country) -> Self {
        crate::Location {
            two_letter_iso_country_code: country.iso_code().to_string(),
            latitude: None,
            longitude: None,
        }
    }
}

impl From<VpnServiceInfo> for ProtoInfoResponse {
    fn from(info: VpnServiceInfo) -> Self {
        let build_timestamp = info
            .build_timestamp
            .map(crate::conversions::prost::offset_datetime_into_proto_timestamp);

        let nym_network = Some(ProtoNymNetworkDetails::from(info.nym_network.clone()));
        let nym_vpn_network = Some(ProtoNymVpnNetworkDetails::from(info.nym_vpn_network));

        Self {
            version: info.version,
            build_timestamp,
            triple: info.triple,
            platform: info.platform,
            git_commit: info.git_commit,
            nym_network,
            nym_vpn_network,
        }
    }
}

impl From<LogPath> for ProtoGetLogPathResponse {
    fn from(value: nym_vpnd_types::log_path::LogPath) -> Self {
        Self {
            // todo: consider TryFrom instead to raise encoding issues
            path: value.dir.to_string_lossy().into_owned(),
            filename: value.filename,
        }
    }
}

impl TryFrom<ConnectArgs> for ConnectRequest {
    type Error = ConversionError;

    fn try_from(value: ConnectArgs) -> Result<Self, Self::Error> {
        let entry = value.entry.map(ProtoEntryNode::try_from).transpose()?;
        let exit = value.exit.map(ProtoExitNode::try_from).transpose()?;
        Ok(Self {
            dns: value.options.dns.map(|ip| ProtoDns { ip: ip.to_string() }),
            enable_two_hop: value.options.enable_two_hop,
            netstack: value.options.netstack,
            disable_poisson_rate: value.options.disable_poisson_rate,
            disable_background_cover_traffic: value.options.disable_background_cover_traffic,
            enable_credentials_mode: value.options.enable_credentials_mode,
            user_agent: value.options.user_agent.map(|s| ProtoUserAgent::from(s)),
            entry,
            exit,
        })
    }
}

impl TryFrom<nym_gateway_directory::ExitPoint> for ProtoExitNode {
    type Error = ConversionError;

    fn try_from(value: nym_gateway_directory::ExitPoint) -> Result<Self, Self::Error> {
        let exit_node_enum = match value {
            nym_gateway_directory::ExitPoint::Address { address } => {
                ProtoExitNodeEnum::Address(ProtoAddress {
                    nym_address: address.to_string(),
                    gateway_id: address.gateway().to_base58_string(),
                })
            }
            nym_gateway_directory::ExitPoint::Gateway { identity } => {
                ProtoExitNodeEnum::Gateway(crate::Gateway {
                    id: identity.to_base58_string(),
                })
            }
            nym_gateway_directory::ExitPoint::Location { location } => {
                ProtoExitNodeEnum::Location(crate::Location {
                    two_letter_iso_country_code: location,
                    latitude: None,
                    longitude: None,
                })
            }
            nym_gateway_directory::ExitPoint::Random => ProtoExitNodeEnum::Random(()),
        };
        Ok(ProtoExitNode {
            exit_node_enum: Some(exit_node_enum),
        })
    }
}

impl TryFrom<nym_gateway_directory::EntryPoint> for ProtoEntryNode {
    type Error = ConversionError;
    fn try_from(value: nym_gateway_directory::EntryPoint) -> Result<Self, Self::Error> {
        match value {
            nym_gateway_directory::EntryPoint::Gateway { identity } => Ok(ProtoEntryNode {
                entry_node_enum: Some(crate::entry_node::EntryNodeEnum::Gateway(crate::Gateway {
                    id: identity.to_base58_string(),
                })),
            }),
            nym_gateway_directory::EntryPoint::Location { location } => Ok(ProtoEntryNode {
                entry_node_enum: Some(crate::entry_node::EntryNodeEnum::Location(
                    crate::Location {
                        two_letter_iso_country_code: location,
                        latitude: None,
                        longitude: None,
                    },
                )),
            }),
            nym_gateway_directory::EntryPoint::Random => Ok(ProtoEntryNode {
                entry_node_enum: Some(crate::entry_node::EntryNodeEnum::Random(())),
            }),
        }
    }
}

impl TryFrom<ListGatewaysOptions> for ProtoListGatewaysRequest {
    type Error = ConversionError;

    fn try_from(value: ListGatewaysOptions) -> Result<Self, Self::Error> {
        let proto_gw_type = ProtoGatewayType::from(value.gw_type);
        let user_agent = value.user_agent.map(ProtoUserAgent::from);

        Ok(Self {
            kind: proto_gw_type as i32,
            user_agent,
        })
    }
}

impl TryFrom<ListCountriesOptions> for ProtoListCountriesRequest {
    type Error = ConversionError;

    fn try_from(value: ListCountriesOptions) -> Result<Self, Self::Error> {
        let proto_gw_type = ProtoGatewayType::from(value.gw_type);
        let user_agent = value.user_agent.map(ProtoUserAgent::from);

        Ok(Self {
            kind: proto_gw_type as i32,
            user_agent,
        })
    }
}
