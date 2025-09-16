// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    str::FromStr,
};

use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use url::Url;

use nym_config::defaults::NymNetworkDetails;
use nym_gateway_directory::{GatewayType, NaiveFloat, Percent};
use nym_sdk::UserAgent;
use nym_vpn_network_config::{
    ApiUrl, NymNetwork, NymVpnNetwork, SystemMessage, SystemMessages, system_messages::Properties,
};
use nym_vpnd_types::{
    AccountCommandResponse, ListCountriesOptions, ListGatewaysOptions, StoreAccountRequest,
    gateway::Score,
    log_path::LogPath,
    service::{ConnectArgs, ConnectOptions, VpnServiceConfig, VpnServiceInfo},
};

use crate::{conversions::ConversionError, proto};

// Used for serializing the Percent type to 3 decimal places
macro_rules! round_f64 {
    ($float:expr) => {{ ($float as f64 * 1000.0).round() / 1000.0 }};
}

macro_rules! round_f32 {
    ($float:expr) => {{ (($float * 1000.0).round() / 1000.0) as f32 }};
}

impl From<proto::Location> for nym_vpnd_types::gateway::Location {
    fn from(location: proto::Location) -> Self {
        Self {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<proto::Score> for Score {
    fn from(score: proto::Score) -> Self {
        match score {
            proto::Score::None => nym_vpnd_types::gateway::Score::None,
            proto::Score::Low => nym_vpnd_types::gateway::Score::Low,
            proto::Score::Medium => nym_vpnd_types::gateway::Score::Medium,
            proto::Score::High => nym_vpnd_types::gateway::Score::High,
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

impl From<proto::AsEntry> for nym_vpnd_types::gateway::Entry {
    fn from(entry: proto::AsEntry) -> Self {
        Self {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
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

impl From<proto::AsExit> for nym_vpnd_types::gateway::Exit {
    fn from(exit: proto::AsExit) -> Self {
        Self {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
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

impl TryFrom<proto::ProbeOutcome> for nym_vpnd_types::gateway::ProbeOutcome {
    type Error = ConversionError;

    fn try_from(outcome: proto::ProbeOutcome) -> Result<Self, Self::Error> {
        let as_entry = outcome
            .as_entry
            .map(nym_vpnd_types::gateway::Entry::from)
            .ok_or(ConversionError::generic("missing as entry"))?;
        let as_exit = outcome.as_exit.map(nym_vpnd_types::gateway::Exit::from);
        Ok(Self { as_entry, as_exit })
    }
}

impl TryFrom<proto::Probe> for nym_vpnd_types::gateway::Probe {
    type Error = ConversionError;

    fn try_from(probe: proto::Probe) -> Result<Self, Self::Error> {
        let last_updated_utc = probe
            .last_updated_utc
            .ok_or(ConversionError::generic("missing last updated timestamp"))
            .map(|timestamp| timestamp.to_string())?;
        let outcome = probe
            .outcome
            .ok_or(ConversionError::generic("missing probe outcome"))
            .and_then(nym_vpnd_types::gateway::ProbeOutcome::try_from)?;
        Ok(Self {
            last_updated_utc,
            outcome,
        })
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

impl TryFrom<proto::GatewayResponse> for nym_vpnd_types::gateway::Gateway {
    type Error = ConversionError;
    fn try_from(gateway: proto::GatewayResponse) -> Result<Self, Self::Error> {
        let identity_key = gateway
            .id
            .map(|id| id.id)
            .ok_or_else(|| ConversionError::generic("missing gateway id"))?;
        let moniker = gateway.moniker;
        let location = gateway
            .location
            .map(nym_vpnd_types::gateway::Location::from);
        let last_probe = gateway
            .last_probe
            .map(nym_vpnd_types::gateway::Probe::try_from)
            .transpose()?;
        let mixnet_score = gateway
            .mixnet_score
            .map(nym_vpnd_types::gateway::Score::from_i32);
        let wg_score = gateway
            .wg_score
            .map(nym_vpnd_types::gateway::Score::from_i32);

        let exit_ipv4s = gateway
            .exit_ipv4s
            .iter()
            .map(|ip| Ipv4Addr::from_str(ip))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ConversionError::ParseAddr("GatewayResponse.exit_ipv4s", e))?;
        let exit_ipv6s = gateway
            .exit_ipv6s
            .iter()
            .map(|ip| Ipv6Addr::from_str(ip))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ConversionError::ParseAddr("GatewayResponse.exit_ipv6s", e))?;
        let build_version = gateway.build_version;
        Ok(Self {
            identity_key,
            moniker,
            location,
            last_probe,
            wg_score,
            mixnet_score,
            exit_ipv4s,
            exit_ipv6s,
            build_version,
        })
    }
}

impl From<nym_vpnd_types::gateway::Gateway> for proto::GatewayResponse {
    fn from(gateway: nym_vpnd_types::gateway::Gateway) -> Self {
        let id = Some(proto::GatewayId {
            id: gateway.identity_key.to_string(),
        });
        let location = gateway.location.map(proto::Location::from);
        let last_probe = gateway.last_probe.map(proto::Probe::from);
        let moniker = gateway.moniker;
        let exit_ipv4s = gateway.exit_ipv4s.iter().map(|ip| ip.to_string()).collect();
        let exit_ipv6s = gateway.exit_ipv6s.iter().map(|ip| ip.to_string()).collect();
        let build_version = gateway.build_version;
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
            exit_ipv4s,
            exit_ipv6s,
            build_version,
        }
    }
}

impl From<proto::Location> for nym_vpnd_types::gateway::Country {
    fn from(location: proto::Location) -> Self {
        Self {
            iso_code: location.two_letter_iso_country_code,
        }
    }
}

impl TryFrom<proto::InfoResponse> for nym_vpnd_types::service::VpnServiceInfo {
    type Error = ConversionError;

    fn try_from(info: proto::InfoResponse) -> Result<Self, Self::Error> {
        let build_timestamp = info
            .build_timestamp
            .map(crate::conversions::prost::prost_timestamp_into_offset_datetime)
            .transpose()
            .map_err(|e| ConversionError::ConvertTime("build_timestamp", e))?;

        // todo: why is it not passed as `NymNetwork` instead?
        let nym_network = info
            .nym_network
            .ok_or(ConversionError::NoValueSet("nym_network"))
            .and_then(NymNetworkDetails::try_from)
            .map(NymNetwork::new)?;

        // todo: why is it not passed as `NymVpnNetwork` instead?
        let nym_vpn_network = info
            .nym_vpn_network
            .ok_or(ConversionError::NoValueSet("nym_vpn_network"))
            .and_then(|s| {
                // todo: rework this later
                let nym_vpn_api_url = s
                    .nym_vpn_api_url
                    .ok_or(ConversionError::NoValueSet(
                        "NymVpnNetworkDetails.nym_vpn_api_url",
                    ))
                    .and_then(|s| {
                        Url::from_str(&s.url).map_err(|e| {
                            ConversionError::Generic(format!("failed to parse Url: {e}"))
                        })
                    })?;
                let nym_vpn_api_urls = s
                    .nym_vpn_api_urls
                    .into_iter()
                    .map(ApiUrl::from)
                    .collect::<Vec<_>>();
                Ok(NymVpnNetwork {
                    nym_vpn_api_url,
                    nym_vpn_api_urls,
                    account_management: Default::default(),
                    system_messages: Default::default(),
                })
            })?;

        Ok(Self {
            version: info.version,
            build_timestamp,
            triple: info.triple,
            platform: info.platform,
            git_commit: info.git_commit,
            nym_network,
            nym_vpn_network,
        })
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

impl From<proto::GetLogPathResponse> for nym_vpnd_types::log_path::LogPath {
    fn from(value: proto::GetLogPathResponse) -> Self {
        Self {
            dir: PathBuf::from(value.dir),
            filename: value.filename,
        }
    }
}

impl TryFrom<LogPath> for proto::GetLogPathResponse {
    type Error = ConversionError;

    fn try_from(value: LogPath) -> Result<Self, Self::Error> {
        Ok(Self {
            // todo: consider TryFrom instead to raise encoding issues
            dir: value
                .dir
                .into_os_string()
                .into_string()
                .map_err(ConversionError::Utf8Encoding)?,
            filename: value.filename,
        })
    }
}

impl From<proto::GetSystemMessagesResponse> for SystemMessages {
    fn from(value: proto::GetSystemMessagesResponse) -> Self {
        Self {
            messages: value
                .messages
                .into_iter()
                .map(SystemMessage::from)
                .collect(),
        }
    }
}

impl From<proto::SystemMessage> for SystemMessage {
    fn from(value: proto::SystemMessage) -> Self {
        Self {
            // todo: why is this not present in protobuf?
            display_from: None,
            display_until: None,
            name: value.name,
            message: value.message,
            properties: Some(Properties::from(value.properties)),
        }
    }
}

impl TryFrom<proto::ConnectRequest> for ConnectArgs {
    type Error = ConversionError;

    fn try_from(value: proto::ConnectRequest) -> Result<Self, Self::Error> {
        let entry = value
            .entry
            .clone() // todo: prevent clone()
            .map(nym_gateway_directory::EntryPoint::try_from)
            .transpose()?;
        let exit = value
            .exit
            .clone() // todo: prevent clone()
            .map(nym_gateway_directory::ExitPoint::try_from)
            .transpose()?;
        let options = ConnectOptions::try_from(value)?;
        Ok(ConnectArgs {
            entry,
            exit,
            options,
        })
    }
}

impl TryFrom<ConnectArgs> for proto::ConnectRequest {
    type Error = ConversionError;

    fn try_from(value: ConnectArgs) -> Result<Self, Self::Error> {
        let entry = value.entry.map(proto::EntryNode::try_from).transpose()?;
        let exit = value.exit.map(proto::ExitNode::try_from).transpose()?;
        Ok(Self {
            dns: value.options.dns.map(|ip| proto::Dns {
                ip: Some(ip.to_string()),
            }),
            disable_ipv6: value.options.disable_ipv6,
            enable_two_hop: value.options.enable_two_hop,
            netstack: value.options.netstack,
            disable_poisson_rate: value.options.disable_poisson_rate,
            disable_background_cover_traffic: value.options.disable_background_cover_traffic,
            enable_credentials_mode: value.options.enable_credentials_mode,
            user_agent: value.options.user_agent.map(proto::UserAgent::from),
            entry,
            exit,
        })
    }
}

impl TryFrom<proto::ConnectRequest> for ConnectOptions {
    type Error = ConversionError;

    fn try_from(value: proto::ConnectRequest) -> Result<Self, Self::Error> {
        let dns = match value.dns.and_then(|dns| dns.ip) {
            Some(ip) => Some(
                ip.parse::<std::net::IpAddr>()
                    .map_err(|e| ConversionError::ParseAddr("VpnServiceConfig.dns", e))?,
            ),
            None => None,
        };

        Ok(Self {
            dns,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            netstack: value.netstack,
            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,
            enable_credentials_mode: value.enable_credentials_mode,
            min_mixnode_performance: None,
            min_gateway_mixnet_performance: None,
            min_gateway_vpn_performance: None,
            user_agent: value.user_agent.map(UserAgent::from),
        })
    }
}

impl TryFrom<proto::VpnServiceConfig> for VpnServiceConfig {
    type Error = ConversionError;

    fn try_from(value: proto::VpnServiceConfig) -> Result<Self, Self::Error> {
        let config = VpnServiceConfig {
            entry_point: value
                .entry_point
                .map(nym_gateway_directory::EntryPoint::try_from)
                .transpose()?
                .ok_or(ConversionError::NoValueSet("VpnServiceConfig.entry_point"))?,

            exit_point: value
                .exit_point
                .map(nym_gateway_directory::ExitPoint::try_from)
                .transpose()?
                .ok_or(ConversionError::NoValueSet("VpnServiceConfig.exit_point"))?,

            dns: match value.dns.and_then(|dns| dns.ip) {
                Some(ip) => Some(
                    ip.parse::<std::net::IpAddr>()
                        .map_err(|e| ConversionError::ParseAddr("VpnServiceConfig.dns", e))?,
                ),
                None => None,
            },

            disable_ipv6: value.disable_ipv6,

            enable_two_hop: value.enable_two_hop,

            netstack: value.netstack,

            disable_poisson_rate: value.disable_poisson_rate,

            disable_background_cover_traffic: value.disable_background_cover_traffic,

            min_mixnode_performance: match value.min_mixnode_performance.and_then(|p| p.percent) {
                Some(f) => Some(
                    Percent::naive_try_from_f64(round_f64!(f))
                        .map_err(|e| ConversionError::ParsePercent(f, e))?,
                ),
                None => None,
            },

            min_gateway_mixnet_performance: match value
                .min_gateway_mixnet_performance
                .and_then(|p| p.percent)
            {
                Some(f) => Some(
                    Percent::naive_try_from_f64(round_f64!(f))
                        .map_err(|e| ConversionError::ParsePercent(f, e))?,
                ),
                None => None,
            },

            min_gateway_vpn_performance: match value
                .min_gateway_vpn_performance
                .and_then(|p| p.percent)
            {
                Some(f) => Some(
                    Percent::naive_try_from_f64(round_f64!(f))
                        .map_err(|e| ConversionError::ParsePercent(f, e))?,
                ),
                None => None,
            },
        };
        Ok(config)
    }
}

impl TryFrom<VpnServiceConfig> for proto::VpnServiceConfig {
    type Error = ConversionError;

    fn try_from(value: VpnServiceConfig) -> Result<Self, Self::Error> {
        let config = proto::VpnServiceConfig {
            entry_point: Some(proto::EntryNode::try_from(value.entry_point)?),
            exit_point: Some(proto::ExitNode::try_from(value.exit_point)?),
            dns: value.dns.map(|ip| proto::Dns {
                ip: Some(ip.to_string()),
            }),
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            netstack: value.netstack,
            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,
            min_mixnode_performance: value.min_mixnode_performance.map(|p| proto::Percent {
                percent: Some(round_f32!(p.naive_to_f64())),
            }),
            min_gateway_mixnet_performance: value.min_gateway_mixnet_performance.map(|p| {
                proto::Percent {
                    percent: Some(round_f32!(p.naive_to_f64())),
                }
            }),
            min_gateway_vpn_performance: value.min_gateway_vpn_performance.map(|p| {
                proto::Percent {
                    percent: Some(round_f32!(p.naive_to_f64())),
                }
            }),
        };

        Ok(config)
    }
}

impl TryFrom<proto::EntryNode> for nym_gateway_directory::EntryPoint {
    type Error = ConversionError;

    fn try_from(value: proto::EntryNode) -> Result<Self, Self::Error> {
        let entry_enum_value = value
            .entry_node_enum
            .ok_or(ConversionError::NoValueSet("EntryNode.entry_node_enum"))?;

        Ok(match entry_enum_value {
            proto::entry_node::EntryNodeEnum::Location(location) => {
                nym_gateway_directory::EntryPoint::Location {
                    location: location.two_letter_iso_country_code.to_string(),
                }
            }
            proto::entry_node::EntryNodeEnum::Gateway(gateway) => {
                let identity = nym_gateway_directory::NodeIdentity::from_base58_string(&gateway.id)
                    .map_err(|err| {
                        ConversionError::Generic(format!("failed to parse gateway id: {err}"))
                    })?;
                nym_gateway_directory::EntryPoint::Gateway { identity }
            }
            proto::entry_node::EntryNodeEnum::Random(_) => {
                nym_gateway_directory::EntryPoint::Random
            }
        })
    }
}

impl TryFrom<nym_gateway_directory::EntryPoint> for proto::EntryNode {
    type Error = ConversionError;
    fn try_from(value: nym_gateway_directory::EntryPoint) -> Result<Self, Self::Error> {
        match value {
            nym_gateway_directory::EntryPoint::Gateway { identity } => Ok(proto::EntryNode {
                entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Gateway(
                    proto::GatewayId {
                        id: identity.to_base58_string(),
                    },
                )),
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

impl TryFrom<proto::ExitNode> for nym_gateway_directory::ExitPoint {
    type Error = ConversionError;

    fn try_from(value: proto::ExitNode) -> Result<Self, Self::Error> {
        let exit_enum_value = value
            .exit_node_enum
            .ok_or(ConversionError::NoValueSet("ExitNode.exit_node_enum"))?;

        Ok(match exit_enum_value {
            proto::exit_node::ExitNodeEnum::Address(address) => {
                let address = nym_gateway_directory::Recipient::try_from_base58_string(
                    address.nym_address.clone(),
                )
                .map_err(|err| {
                    ConversionError::Generic(format!("failed to parse exit node address: {err}"))
                })?;
                nym_gateway_directory::ExitPoint::Address {
                    address: Box::new(address),
                }
            }
            proto::exit_node::ExitNodeEnum::Gateway(gateway) => {
                let identity = nym_gateway_directory::NodeIdentity::from_base58_string(&gateway.id)
                    .map_err(|err| {
                        ConversionError::Generic(format!("failed to parse gateway id: {err}"))
                    })?;
                nym_gateway_directory::ExitPoint::Gateway { identity }
            }
            proto::exit_node::ExitNodeEnum::Location(location) => {
                nym_gateway_directory::ExitPoint::Location {
                    location: location.two_letter_iso_country_code.to_string(),
                }
            }
            proto::exit_node::ExitNodeEnum::Random(_) => nym_gateway_directory::ExitPoint::Random,
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
                proto::exit_node::ExitNodeEnum::Gateway(proto::GatewayId {
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

impl TryFrom<proto::ListGatewaysRequest> for ListGatewaysOptions {
    type Error = ConversionError;

    fn try_from(value: proto::ListGatewaysRequest) -> Result<Self, Self::Error> {
        let proto_gw_type = proto::GatewayType::try_from(value.kind)
            .map_err(|err| ConversionError::Decode("ListGatewaysRequest.kind", err))?;

        Ok(Self {
            gw_type: GatewayType::from(proto_gw_type),
            user_agent: value.user_agent.map(UserAgent::from),
        })
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

impl TryFrom<proto::ListCountriesRequest> for ListCountriesOptions {
    type Error = ConversionError;

    fn try_from(value: proto::ListCountriesRequest) -> Result<Self, Self::Error> {
        let gw_type = proto::GatewayType::try_from(value.kind)
            .map_err(|err| ConversionError::Decode("ListCountriesRequest.kind", err))
            .map(nym_gateway_directory::GatewayType::from)?;

        let user_agent = value.user_agent.map(nym_sdk::UserAgent::from);

        Ok(Self {
            gw_type,
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

impl From<proto::StoreAccountRequest> for nym_vpnd_types::StoreAccountRequest {
    fn from(value: proto::StoreAccountRequest) -> Self {
        Self {
            mnemonic: value.mnemonic,
        }
    }
}

impl From<StoreAccountRequest> for proto::StoreAccountRequest {
    fn from(value: StoreAccountRequest) -> Self {
        Self {
            mnemonic: value.mnemonic,
        }
    }
}

impl TryFrom<proto::AccountCommandResponse> for nym_vpnd_types::AccountCommandResponse {
    type Error = ConversionError;
    fn try_from(value: proto::AccountCommandResponse) -> Result<Self, Self::Error> {
        let error = value
            .error
            .map(nym_vpn_lib_types::AccountCommandError::try_from)
            .transpose()
            .map_err(|e| {
                ConversionError::Generic(format!("failed to parse AccountCommandError: {e}"))
            })?;

        Ok(Self { error })
    }
}

impl TryFrom<AccountCommandResponse> for proto::AccountCommandResponse {
    type Error = ConversionError;

    fn try_from(value: AccountCommandResponse) -> Result<Self, Self::Error> {
        let error = value
            .error
            .map(proto::AccountCommandError::try_from)
            .transpose()
            .map_err(|e| {
                ConversionError::Generic(format!("failed to parse AccountCommandError: {e}"))
            })?;

        Ok(Self { error })
    }
}

impl From<nym_vpnd_types::gateway::Location> for proto::Location {
    fn from(location: nym_vpnd_types::gateway::Location) -> Self {
        proto::Location {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto;
    use nym_gateway_directory::{EntryPoint, ExitPoint, NaiveFloat, NodeIdentity, Percent};
    use std::str::FromStr;

    #[test]
    fn test_vpn_service_config_conversion() {
        let internal = VpnServiceConfig {
            entry_point: EntryPoint::Location {
                location: "US".to_string(),
            },
            exit_point: ExitPoint::Gateway {
                identity: NodeIdentity::from_str("7fp3cmzCvgeRgbB1ycTnK6RokjHNqPmCCSBG23gyxshj")
                    .unwrap(),
            },
            dns: Some(std::net::IpAddr::from_str("192.168.50.1").unwrap()),
            disable_ipv6: true,
            enable_two_hop: true,
            netstack: true,
            disable_poisson_rate: true,
            disable_background_cover_traffic: true,
            min_mixnode_performance: Some(Percent::naive_try_from_f64(0.552).unwrap()),
            min_gateway_mixnet_performance: Some(Percent::naive_try_from_f64(0.643).unwrap()),
            min_gateway_vpn_performance: Some(Percent::naive_try_from_f64(0.001).unwrap()),
        };

        let external: proto::VpnServiceConfig = internal.clone().try_into().unwrap();
        let internal_converted: VpnServiceConfig = external.try_into().unwrap();
        assert_eq!(internal, internal_converted);
    }
}
