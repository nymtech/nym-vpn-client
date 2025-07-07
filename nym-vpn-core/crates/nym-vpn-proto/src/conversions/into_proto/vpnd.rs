// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpnd_types::{
    ConnectArgs, ForgetAccountResponse, ListCountriesOptions, ListGatewaysOptions,
    StoreAccountRequest, StoreAccountResponse, gateway::Score, log_path::LogPath,
    service::VpnServiceInfo,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{conversions::error::ConversionError, proto};

impl From<nym_vpnd_types::gateway::Location> for proto::Location {
    fn from(location: nym_vpnd_types::gateway::Location) -> Self {
        proto::Location {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<Score> for proto::Score {
    fn from(score: Score) -> Self {
        match score {
            Score::High => proto::Score::High,
            Score::Medium => proto::Score::Medium,
            Score::Low => proto::Score::Low,
            Score::None => proto::Score::None,
        }
    }
}

impl From<nym_vpnd_types::gateway::Entry> for proto::AsEntry {
    fn from(entry: nym_vpnd_types::gateway::Entry) -> Self {
        proto::AsEntry {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<nym_vpnd_types::gateway::Exit> for proto::AsExit {
    fn from(exit: nym_vpnd_types::gateway::Exit) -> Self {
        proto::AsExit {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

impl From<nym_vpnd_types::gateway::ProbeOutcome> for proto::ProbeOutcome {
    fn from(outcome: nym_vpnd_types::gateway::ProbeOutcome) -> Self {
        let as_entry = Some(proto::AsEntry::from(outcome.as_entry));
        let as_exit = outcome.as_exit.map(proto::AsExit::from);
        let wg = None;
        proto::ProbeOutcome {
            as_entry,
            as_exit,
            wg,
        }
    }
}

impl From<nym_vpnd_types::gateway::Probe> for proto::Probe {
    fn from(probe: nym_vpnd_types::gateway::Probe) -> Self {
        let last_updated = OffsetDateTime::parse(&probe.last_updated_utc, &Rfc3339).ok();
        let last_updated_utc = last_updated.map(|timestamp| prost_types::Timestamp {
            seconds: timestamp.unix_timestamp(),
            nanos: timestamp.nanosecond() as i32,
        });
        let outcome = Some(proto::ProbeOutcome::from(probe.outcome));
        proto::Probe {
            last_updated_utc,
            outcome,
        }
    }
}

impl From<nym_vpnd_types::gateway::Gateway> for proto::GatewayResponse {
    fn from(gateway: nym_vpnd_types::gateway::Gateway) -> Self {
        let id = Some(proto::Gateway {
            id: gateway.identity_key.to_string(),
        });
        let location = gateway.location.map(proto::Location::from);
        let last_probe = gateway.last_probe.map(proto::Probe::from);
        let moniker = gateway.moniker;
        proto::GatewayResponse {
            id,
            location,
            last_probe,
            wg_score: gateway
                .wg_score
                .map(|score| proto::Score::from(score) as i32),
            mixnet_score: gateway
                .mixnet_score
                .map(|score| proto::Score::from(score) as i32),
            moniker,
        }
    }
}

impl From<nym_vpnd_types::gateway::Country> for proto::Location {
    fn from(country: nym_vpnd_types::gateway::Country) -> Self {
        proto::Location {
            two_letter_iso_country_code: country.iso_code().to_string(),
            latitude: None,
            longitude: None,
        }
    }
}

impl From<VpnServiceInfo> for proto::InfoResponse {
    fn from(info: VpnServiceInfo) -> Self {
        let build_timestamp = info
            .build_timestamp
            .map(crate::conversions::prost::offset_datetime_into_proto_timestamp);

        let nym_network = Some(proto::NymNetworkDetails::from(info.nym_network.clone()));
        let nym_vpn_network = Some(proto::NymVpnNetworkDetails::from(info.nym_vpn_network));

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

impl From<LogPath> for proto::GetLogPathResponse {
    fn from(value: nym_vpnd_types::log_path::LogPath) -> Self {
        Self {
            // todo: consider TryFrom instead to raise encoding issues
            path: value.dir.to_string_lossy().into_owned(),
            filename: value.filename,
        }
    }
}

impl TryFrom<ConnectArgs> for proto::ConnectRequest {
    type Error = ConversionError;

    fn try_from(value: ConnectArgs) -> Result<Self, Self::Error> {
        let entry = value.entry.map(proto::EntryNode::try_from).transpose()?;
        let exit = value.exit.map(proto::ExitNode::try_from).transpose()?;
        Ok(Self {
            dns: value
                .options
                .dns
                .map(|ip| proto::Dns { ip: ip.to_string() }),
            enable_two_hop: value.options.enable_two_hop,
            netstack: value.options.netstack,
            disable_poisson_rate: value.options.disable_poisson_rate,
            disable_background_cover_traffic: value.options.disable_background_cover_traffic,
            enable_credentials_mode: value.options.enable_credentials_mode,
            user_agent: value.options.user_agent.map(|s| proto::UserAgent::from(s)),
            entry,
            exit,
        })
    }
}

impl TryFrom<nym_gateway_directory::ExitPoint> for proto::ExitNode {
    type Error = ConversionError;

    fn try_from(value: nym_gateway_directory::ExitPoint) -> Result<Self, Self::Error> {
        let exit_node_enum = match value {
            nym_gateway_directory::ExitPoint::Address { address } => {
                proto::exit_node::ExitNodeEnum::Address(proto::Address {
                    nym_address: address.to_string(),
                    gateway_id: address.gateway().to_base58_string(),
                })
            }
            nym_gateway_directory::ExitPoint::Gateway { identity } => {
                proto::exit_node::ExitNodeEnum::Gateway(proto::Gateway {
                    id: identity.to_base58_string(),
                })
            }
            nym_gateway_directory::ExitPoint::Location { location } => {
                proto::exit_node::ExitNodeEnum::Location(proto::Location {
                    two_letter_iso_country_code: location,
                    latitude: None,
                    longitude: None,
                })
            }
            nym_gateway_directory::ExitPoint::Random => proto::exit_node::ExitNodeEnum::Random(()),
        };
        Ok(proto::ExitNode {
            exit_node_enum: Some(exit_node_enum),
        })
    }
}

impl TryFrom<nym_gateway_directory::EntryPoint> for proto::EntryNode {
    type Error = ConversionError;
    fn try_from(value: nym_gateway_directory::EntryPoint) -> Result<Self, Self::Error> {
        match value {
            nym_gateway_directory::EntryPoint::Gateway { identity } => Ok(proto::EntryNode {
                entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Gateway(proto::Gateway {
                    id: identity.to_base58_string(),
                })),
            }),
            nym_gateway_directory::EntryPoint::Location { location } => Ok(proto::EntryNode {
                entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Location(
                    proto::Location {
                        two_letter_iso_country_code: location,
                        latitude: None,
                        longitude: None,
                    },
                )),
            }),
            nym_gateway_directory::EntryPoint::Random => Ok(proto::EntryNode {
                entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Random(())),
            }),
        }
    }
}

impl TryFrom<ListGatewaysOptions> for proto::ListGatewaysRequest {
    type Error = ConversionError;

    fn try_from(value: ListGatewaysOptions) -> Result<Self, Self::Error> {
        let proto_gw_type = proto::GatewayType::from(value.gw_type);
        let user_agent = value.user_agent.map(proto::UserAgent::from);

        Ok(Self {
            kind: proto_gw_type as i32,
            user_agent,
        })
    }
}

impl TryFrom<ListCountriesOptions> for proto::ListCountriesRequest {
    type Error = ConversionError;

    fn try_from(value: ListCountriesOptions) -> Result<Self, Self::Error> {
        let proto_gw_type = proto::GatewayType::from(value.gw_type);
        let user_agent = value.user_agent.map(proto::UserAgent::from);

        Ok(Self {
            kind: proto_gw_type as i32,
            user_agent,
        })
    }
}

impl From<StoreAccountRequest> for proto::StoreAccountRequest {
    fn from(value: StoreAccountRequest) -> Self {
        Self {
            mnemonic: value.mnemonic,
        }
    }
}

impl TryFrom<StoreAccountResponse> for proto::StoreAccountResponse {
    type Error = ConversionError;

    fn try_from(value: StoreAccountResponse) -> Result<Self, Self::Error> {
        let error = value
            .error
            .map(proto::StoreAccountError::try_from)
            .transpose()
            .map_err(|e| {
                ConversionError::Generic(format!("failed to parse StoreAccountError: {e}"))
            })?;

        Ok(Self { error })
    }
}

impl TryFrom<ForgetAccountResponse> for proto::ForgetAccountResponse {
    type Error = ConversionError;

    fn try_from(value: ForgetAccountResponse) -> Result<Self, Self::Error> {
        let error = value.error.map(proto::ForgetAccountError::from);

        Ok(Self { error })
    }
}
