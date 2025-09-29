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
use nym_gateway_directory::GatewayType;
use nym_sdk::UserAgent;
use nym_vpn_network_config::{
    ApiUrl, NymNetwork, NymVpnNetwork, SystemMessage, SystemMessages, system_messages::Properties,
};
use nym_vpnd_types::{
    AccountCommandResponse, ListGatewaysOptions, StoreAccountRequest,
    gateway::{Performance, Score},
    log_path::LogPath,
    service::{ConnectArgs, ConnectOptions, VpnServiceInfo},
};

use crate::{conversions::ConversionError, proto};

impl TryFrom<proto::RichLocation> for nym_vpnd_types::gateway::Location {
    type Error = ConversionError;

    fn try_from(location: proto::RichLocation) -> Result<Self, Self::Error> {
        Ok(Self {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
            city: location.city,
            region: location.region,
            asn: location.asn.map(TryInto::try_into).transpose()?,
        })
    }
}

impl From<proto::Score> for Score {
    fn from(score: proto::Score) -> Self {
        match score {
            proto::Score::Offline => nym_vpnd_types::gateway::Score::Offline,
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
            Score::Offline => proto::Score::Offline,
        }
    }
}

impl TryFrom<proto::Performance> for nym_vpnd_types::gateway::Performance {
    type Error = ConversionError;

    fn try_from(value: proto::Performance) -> Result<Self, Self::Error> {
        Ok(Self {
            last_updated_utc: value.last_updated_utc,
            score: proto::Score::try_from(value.score)
                .map_err(|err| ConversionError::Decode("Performance.score", err))?
                .into(),
            load: proto::Score::try_from(value.load)
                .map_err(|err| ConversionError::Decode("Performance.load", err))?
                .into(),
            uptime_percentage_last_24_hours: value.uptime_percentage_last_24_hours,
        })
    }
}

impl From<nym_vpnd_types::gateway::Performance> for proto::Performance {
    fn from(value: nym_vpnd_types::gateway::Performance) -> Self {
        Self {
            last_updated_utc: value.last_updated_utc,
            score: proto::Score::from(value.score).into(),
            load: proto::Score::from(value.load).into(),
            uptime_percentage_last_24_hours: value.uptime_percentage_last_24_hours,
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
            .map(nym_vpnd_types::gateway::Location::try_from)
            .transpose()?;
        let last_probe = gateway
            .last_probe
            .map(nym_vpnd_types::gateway::Probe::try_from)
            .transpose()?;
        let mixnet_score = gateway
            .mixnet_score
            .map(proto::Score::try_from)
            .transpose()
            .map_err(|err| ConversionError::Decode("GatewayResponse.mixnet_score", err))?
            .map(Score::from);
        let mixnet_performance = gateway.mixnet_performance.map(|x| x as u8);
        let wg_performance = gateway
            .wg_performance
            .map(Performance::try_from)
            .transpose()?;

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
            wg_performance,
            mixnet_performance,
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
        let location = gateway.location.map(proto::RichLocation::from);
        let last_probe = gateway.last_probe.map(proto::Probe::from);
        let moniker = gateway.moniker;
        let exit_ipv4s = gateway.exit_ipv4s.iter().map(|ip| ip.to_string()).collect();
        let exit_ipv6s = gateway.exit_ipv6s.iter().map(|ip| ip.to_string()).collect();
        let build_version = gateway.build_version;
        proto::GatewayResponse {
            id,
            location,
            last_probe,
            mixnet_performance: gateway.mixnet_performance.map(u32::from),
            mixnet_score: gateway
                .mixnet_score
                .map(|score| proto::Score::from(score) as i32),
            wg_performance: gateway.wg_performance.map(Into::into),
            moniker,
            exit_ipv4s,
            exit_ipv6s,
            build_version,
        }
    }
}

impl From<proto::Country> for nym_vpnd_types::gateway::Country {
    fn from(location: proto::Country) -> Self {
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
        let entry = value.entry.map(proto::EntryNode::from);
        let exit = value.exit.map(proto::ExitNode::from);
        Ok(Self {
            dns: value.options.dns.map(|ip| proto::Dns {
                ip: Some(ip.to_string()),
            }),
            disable_ipv6: value.options.disable_ipv6,
            enable_two_hop: value.options.enable_two_hop,
            enable_bridges: value.options.enable_bridges,
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
                    .map_err(|e| ConversionError::ParseAddr("ConnectRequest.dns", e))?,
            ),
            None => None,
        };

        Ok(Self {
            dns,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            netstack: value.netstack,
            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,
            enable_credentials_mode: value.enable_credentials_mode,
            user_agent: value.user_agent.map(UserAgent::from),
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

impl From<nym_vpnd_types::gateway::AsnKind> for proto::AsnKind {
    fn from(value: nym_vpnd_types::gateway::AsnKind) -> Self {
        match value {
            nym_vpnd_types::gateway::AsnKind::Residential => proto::AsnKind::Residential,
            nym_vpnd_types::gateway::AsnKind::Other => proto::AsnKind::Other,
        }
    }
}

impl From<proto::AsnKind> for nym_vpnd_types::gateway::AsnKind {
    fn from(value: proto::AsnKind) -> Self {
        match value {
            proto::AsnKind::Residential => nym_vpnd_types::gateway::AsnKind::Residential,
            proto::AsnKind::Other => nym_vpnd_types::gateway::AsnKind::Other,
        }
    }
}

impl From<nym_vpnd_types::gateway::Asn> for proto::Asn {
    fn from(value: nym_vpnd_types::gateway::Asn) -> Self {
        proto::Asn {
            asn: value.asn,
            name: value.name,
            kind: proto::AsnKind::from(value.kind).into(),
        }
    }
}

impl TryFrom<proto::Asn> for nym_vpnd_types::gateway::Asn {
    type Error = ConversionError;

    fn try_from(value: proto::Asn) -> Result<Self, Self::Error> {
        Ok(nym_vpnd_types::gateway::Asn {
            asn: value.asn,
            name: value.name,
            kind: proto::AsnKind::try_from(value.kind)
                .map_err(|err| ConversionError::Decode("AsnKind", err))?
                .into(),
        })
    }
}

impl From<nym_vpnd_types::gateway::Location> for proto::RichLocation {
    fn from(location: nym_vpnd_types::gateway::Location) -> Self {
        proto::RichLocation {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
            city: location.city,
            region: location.region,
            asn: location.asn.map(Into::into),
        }
    }
}

impl From<nym_vpnd_types::gateway::Country> for proto::Country {
    fn from(country: nym_vpnd_types::gateway::Country) -> Self {
        proto::Country {
            two_letter_iso_country_code: country.iso_code().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::proto;
    use nym_gateway_directory::{EntryPoint, ExitPoint, NodeIdentity};
    use nym_vpn_lib_types::service::VpnServiceConfig;
    use std::str::FromStr;

    #[test]
    fn test_vpn_service_config_conversion() {
        let internal = VpnServiceConfig {
            entry_point: EntryPoint::Country {
                two_letter_iso_country_code: "US".to_string(),
            },
            exit_point: ExitPoint::Gateway {
                identity: NodeIdentity::from_str("7fp3cmzCvgeRgbB1ycTnK6RokjHNqPmCCSBG23gyxshj")
                    .unwrap(),
            },
            dns: Some(std::net::IpAddr::from_str("192.168.50.1").unwrap()),
            allow_lan: false,
            disable_ipv6: false,
            enable_two_hop: false,
            enable_bridges: false,
            netstack: false,
            disable_poisson_rate: false,
            disable_background_cover_traffic: false,
            min_mixnode_performance: Some(55u8),
            min_gateway_mixnet_performance: Some(64u8),
            min_gateway_vpn_performance: Some(1u8),
        };

        let external: proto::VpnServiceConfig = internal.clone().into();
        let internal_converted: VpnServiceConfig = external.try_into().unwrap();
        assert_eq!(internal, internal_converted);
    }
}
