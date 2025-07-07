// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, str::FromStr};

use nym_config::defaults::NymNetworkDetails;
use nym_gateway_directory::GatewayType;
use nym_sdk::UserAgent;
use nym_vpn_network_config::{
    NymNetwork, NymVpnNetwork, SystemMessage, SystemMessages, system_messages::Properties,
};
use nym_vpnd_types::{ConnectArgs, ConnectOptions, ListCountriesOptions, ListGatewaysOptions};
use url::Url;

use crate::{
    ConnectRequest, EntryNode, ExitNode, GatewayType as ProtoGatewayType, GetLogPathResponse,
    GetSystemMessagesResponse, ListCountriesRequest as ProtoListCountriesRequest,
    ListGatewaysRequest as ProtoListGatewaysRequest, Score, SystemMessage as ProtoSystemMessage,
    conversions::ConversionError,
};

impl From<crate::Location> for nym_vpnd_types::gateway::Location {
    fn from(location: crate::Location) -> Self {
        Self {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<crate::Score> for nym_vpnd_types::gateway::Score {
    fn from(score: Score) -> Self {
        match score {
            Score::None => nym_vpnd_types::gateway::Score::None,
            Score::Low => nym_vpnd_types::gateway::Score::Low,
            Score::Medium => nym_vpnd_types::gateway::Score::Medium,
            Score::High => nym_vpnd_types::gateway::Score::High,
        }
    }
}

impl From<crate::AsEntry> for nym_vpnd_types::gateway::Entry {
    fn from(entry: crate::AsEntry) -> Self {
        Self {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<crate::AsExit> for nym_vpnd_types::gateway::Exit {
    fn from(exit: crate::AsExit) -> Self {
        Self {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

impl TryFrom<crate::ProbeOutcome> for nym_vpnd_types::gateway::ProbeOutcome {
    type Error = ConversionError;

    fn try_from(outcome: crate::ProbeOutcome) -> Result<Self, Self::Error> {
        let as_entry = outcome
            .as_entry
            .map(nym_vpnd_types::gateway::Entry::from)
            .ok_or(ConversionError::generic("missing as entry"))?;
        let as_exit = outcome.as_exit.map(nym_vpnd_types::gateway::Exit::from);
        Ok(Self { as_entry, as_exit })
    }
}

impl TryFrom<crate::Probe> for nym_vpnd_types::gateway::Probe {
    type Error = ConversionError;

    fn try_from(probe: crate::Probe) -> Result<Self, Self::Error> {
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

impl TryFrom<crate::GatewayResponse> for nym_vpnd_types::gateway::Gateway {
    type Error = ConversionError;
    fn try_from(gateway: crate::GatewayResponse) -> Result<Self, Self::Error> {
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
        Ok(Self {
            identity_key,
            moniker,
            location,
            last_probe,
            wg_score,
            mixnet_score,
        })
    }
}

impl From<crate::Location> for nym_vpnd_types::gateway::Country {
    fn from(location: crate::Location) -> Self {
        Self {
            iso_code: location.two_letter_iso_country_code,
        }
    }
}

impl TryFrom<crate::InfoResponse> for nym_vpnd_types::service::VpnServiceInfo {
    type Error = ConversionError;

    fn try_from(info: crate::InfoResponse) -> Result<Self, Self::Error> {
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
                Ok(NymVpnNetwork {
                    nym_vpn_api_url,
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

impl From<GetLogPathResponse> for nym_vpnd_types::log_path::LogPath {
    fn from(value: GetLogPathResponse) -> Self {
        Self {
            dir: PathBuf::from(value.path),
            filename: value.filename,
        }
    }
}

impl From<GetSystemMessagesResponse> for SystemMessages {
    fn from(value: GetSystemMessagesResponse) -> Self {
        Self {
            messages: value
                .messages
                .into_iter()
                .map(SystemMessage::from)
                .collect(),
        }
    }
}

impl From<ProtoSystemMessage> for SystemMessage {
    fn from(value: ProtoSystemMessage) -> Self {
        Self {
            // todo: why is this not present in protobuf?
            display_from: None,
            display_until: None,
            name: value.name,
            message: value.message,
            properties: Properties::from(value.properties),
        }
    }
}

impl TryFrom<ConnectRequest> for ConnectArgs {
    type Error = ConversionError;

    fn try_from(value: ConnectRequest) -> Result<Self, Self::Error> {
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

        Ok(Self {
            entry,
            exit,
            options,
        })
    }
}

impl TryFrom<ConnectRequest> for ConnectOptions {
    type Error = ConversionError;

    fn try_from(value: ConnectRequest) -> Result<Self, Self::Error> {
        let dns = value
            .dns
            .map(|dns| {
                dns.ip
                    .parse()
                    .map_err(|err| ConversionError::ParseAddr("ConnectRequest.dns", err))
            })
            .transpose()?;

        Ok(Self {
            dns,
            enable_two_hop: value.enable_two_hop,
            netstack: value.netstack,
            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,
            enable_credentials_mode: value.enable_credentials_mode,
            // todo: perf options are missing from connect request?
            min_mixnode_performance: None,
            min_gateway_mixnet_performance: None,
            min_gateway_vpn_performance: None,
            user_agent: value.user_agent.map(nym_sdk::UserAgent::from),
        })
    }
}

impl TryFrom<EntryNode> for nym_gateway_directory::EntryPoint {
    type Error = ConversionError;

    fn try_from(value: EntryNode) -> Result<Self, Self::Error> {
        let entry_enum_value = value
            .entry_node_enum
            .ok_or(ConversionError::NoValueSet("EntryNode.entry_node_enum"))?;

        Ok(match entry_enum_value {
            crate::entry_node::EntryNodeEnum::Location(location) => {
                nym_gateway_directory::EntryPoint::Location {
                    location: location.two_letter_iso_country_code.to_string(),
                }
            }
            crate::entry_node::EntryNodeEnum::Gateway(gateway) => {
                let identity = nym_gateway_directory::NodeIdentity::from_base58_string(&gateway.id)
                    .map_err(|err| {
                        ConversionError::Generic(format!("failed to parse gateway id: {err}"))
                    })?;
                nym_gateway_directory::EntryPoint::Gateway { identity }
            }
            crate::entry_node::EntryNodeEnum::Random(_) => {
                nym_gateway_directory::EntryPoint::Random
            }
        })
    }
}

impl TryFrom<ExitNode> for nym_gateway_directory::ExitPoint {
    type Error = ConversionError;

    fn try_from(value: ExitNode) -> Result<Self, Self::Error> {
        let exit_enum_value = value
            .exit_node_enum
            .ok_or(ConversionError::NoValueSet("ExitNode.exit_node_enum"))?;

        Ok(match exit_enum_value {
            crate::exit_node::ExitNodeEnum::Address(address) => {
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
            crate::exit_node::ExitNodeEnum::Gateway(gateway) => {
                let identity = nym_gateway_directory::NodeIdentity::from_base58_string(&gateway.id)
                    .map_err(|err| {
                        ConversionError::Generic(format!("failed to parse gateway id: {err}"))
                    })?;
                nym_gateway_directory::ExitPoint::Gateway { identity }
            }
            crate::exit_node::ExitNodeEnum::Location(location) => {
                nym_gateway_directory::ExitPoint::Location {
                    location: location.two_letter_iso_country_code.to_string(),
                }
            }
            crate::exit_node::ExitNodeEnum::Random(_) => nym_gateway_directory::ExitPoint::Random,
        })
    }
}

impl TryFrom<ProtoListGatewaysRequest> for ListGatewaysOptions {
    type Error = ConversionError;

    fn try_from(value: ProtoListGatewaysRequest) -> Result<Self, Self::Error> {
        let proto_gw_type = ProtoGatewayType::try_from(value.kind)
            .map_err(|err| ConversionError::Decode("ListGatewaysRequest.kind", err))?;

        Ok(Self {
            gw_type: GatewayType::from(proto_gw_type),
            user_agent: value.user_agent.map(UserAgent::from),
        })
    }
}

impl TryFrom<ProtoListCountriesRequest> for ListCountriesOptions {
    type Error = ConversionError;

    fn try_from(value: ProtoListCountriesRequest) -> Result<Self, Self::Error> {
        let gw_type = ProtoGatewayType::try_from(value.kind)
            .map_err(|err| ConversionError::Decode("ListCountriesRequest.kind", err))
            .map(nym_gateway_directory::GatewayType::from)?;

        let user_agent = value.user_agent.map(nym_sdk::UserAgent::from);

        Ok(Self {
            gw_type,
            user_agent,
        })
    }
}
