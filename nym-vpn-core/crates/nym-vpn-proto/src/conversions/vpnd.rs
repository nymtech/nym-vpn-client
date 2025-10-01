// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    str::FromStr,
};

use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use nym_vpn_lib_types::{
    AccountCommandResponse, ApiUrl, ConnectArgs, ConnectOptions, GatewayType, ListGatewaysOptions,
    LogPath, NymNetworkDetails, NymVpnNetwork, Performance, SystemMessage, UserAgent,
    VpnServiceInfo,
};

use crate::{conversions::ConversionError, proto};

impl TryFrom<proto::Location> for nym_vpn_lib_types::Location {
    type Error = ConversionError;

    fn try_from(location: proto::Location) -> Result<Self, Self::Error> {
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

impl TryFrom<proto::Performance> for nym_vpn_lib_types::Performance {
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

impl From<nym_vpn_lib_types::Performance> for proto::Performance {
    fn from(value: nym_vpn_lib_types::Performance) -> Self {
        Self {
            last_updated_utc: value.last_updated_utc,
            score: proto::Score::from(value.score).into(),
            load: proto::Score::from(value.load).into(),
            uptime_percentage_last_24_hours: value.uptime_percentage_last_24_hours,
        }
    }
}

impl From<proto::AsEntry> for nym_vpn_lib_types::Entry {
    fn from(entry: proto::AsEntry) -> Self {
        Self {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<nym_vpn_lib_types::Entry> for proto::AsEntry {
    fn from(entry: nym_vpn_lib_types::Entry) -> Self {
        proto::AsEntry {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<proto::AsExit> for nym_vpn_lib_types::Exit {
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

impl From<nym_vpn_lib_types::Exit> for proto::AsExit {
    fn from(exit: nym_vpn_lib_types::Exit) -> Self {
        proto::AsExit {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

impl From<nym_vpn_lib_types::ProbeOutcome> for proto::ProbeOutcome {
    fn from(outcome: nym_vpn_lib_types::ProbeOutcome) -> Self {
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

impl TryFrom<proto::ProbeOutcome> for nym_vpn_lib_types::ProbeOutcome {
    type Error = ConversionError;

    fn try_from(outcome: proto::ProbeOutcome) -> Result<Self, Self::Error> {
        let as_entry = outcome
            .as_entry
            .map(nym_vpn_lib_types::Entry::from)
            .ok_or(ConversionError::generic("missing as entry"))?;
        let as_exit = outcome.as_exit.map(nym_vpn_lib_types::Exit::from);
        Ok(Self { as_entry, as_exit })
    }
}

impl TryFrom<proto::Probe> for nym_vpn_lib_types::Probe {
    type Error = ConversionError;

    fn try_from(probe: proto::Probe) -> Result<Self, Self::Error> {
        let last_updated_utc = probe
            .last_updated_utc
            .ok_or(ConversionError::generic("missing last updated timestamp"))
            .map(|timestamp| timestamp.to_string())?;
        let outcome = probe
            .outcome
            .ok_or(ConversionError::generic("missing probe outcome"))
            .and_then(nym_vpn_lib_types::ProbeOutcome::try_from)?;
        Ok(Self {
            last_updated_utc,
            outcome,
        })
    }
}

impl From<nym_vpn_lib_types::Probe> for proto::Probe {
    fn from(probe: nym_vpn_lib_types::Probe) -> Self {
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

impl TryFrom<proto::GatewayResponse> for nym_vpn_lib_types::Gateway {
    type Error = ConversionError;

    fn try_from(gateway: proto::GatewayResponse) -> Result<Self, Self::Error> {
        let identity_key = gateway
            .id
            .map(|id| id.id)
            .ok_or_else(|| ConversionError::generic("missing gateway id"))?;
        let moniker = gateway.moniker;
        let location = gateway
            .location
            .map(nym_vpn_lib_types::Location::try_from)
            .transpose()?;
        let last_probe = gateway
            .last_probe
            .map(nym_vpn_lib_types::Probe::try_from)
            .transpose()?;
        let mixnet_score = gateway
            .mixnet_score
            .map(proto::Score::try_from)
            .transpose()
            .map_err(|err| ConversionError::Decode("GatewayResponse.mixnet_score", err))?
            .map(nym_vpn_lib_types::Score::from);
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

impl From<nym_vpn_lib_types::Gateway> for proto::GatewayResponse {
    fn from(gateway: nym_vpn_lib_types::Gateway) -> Self {
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

impl From<proto::Country> for nym_vpn_lib_types::Country {
    fn from(location: proto::Country) -> Self {
        Self {
            iso_code: location.two_letter_iso_country_code,
        }
    }
}

impl TryFrom<proto::InfoResponse> for nym_vpn_lib_types::VpnServiceInfo {
    type Error = ConversionError;

    fn try_from(info: proto::InfoResponse) -> Result<Self, Self::Error> {
        let build_timestamp = info
            .build_timestamp
            .map(crate::conversions::prost::prost_timestamp_into_offset_datetime)
            .transpose()
            .map_err(|e| ConversionError::ConvertTime("build_timestamp", e))?;

        let nym_network = info
            .nym_network
            .ok_or(ConversionError::NoValueSet("nym_network"))
            .and_then(NymNetworkDetails::try_from)?;

        let nym_vpn_network = info
            .nym_vpn_network
            .ok_or(ConversionError::NoValueSet("nym_vpn_network"))
            .map(|s| {
                let nym_vpn_api_url = s.nym_vpn_api_url;
                let nym_vpn_api_urls = s
                    .nym_vpn_api_urls
                    .into_iter()
                    .map(ApiUrl::from)
                    .collect::<Vec<_>>();
                NymVpnNetwork {
                    nym_vpn_api_url,
                    nym_vpn_api_urls,
                }
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

impl From<proto::GetLogPathResponse> for nym_vpn_lib_types::LogPath {
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

impl From<proto::SystemMessage> for SystemMessage {
    fn from(value: proto::SystemMessage) -> Self {
        Self {
            // todo: why is this not present in protobuf?
            display_from: None,
            display_until: None,
            name: value.name,
            message: value.message,
            properties: Some(value.properties),
        }
    }
}

impl TryFrom<proto::ConnectRequest> for ConnectArgs {
    type Error = ConversionError;

    fn try_from(value: proto::ConnectRequest) -> Result<Self, Self::Error> {
        let entry = value
            .entry
            .clone() // todo: prevent clone()
            .map(nym_vpn_lib_types::EntryPoint::try_from)
            .transpose()?;
        let exit = value
            .exit
            .clone() // todo: prevent clone()
            .map(nym_vpn_lib_types::ExitPoint::try_from)
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

impl TryFrom<proto::StoreAccountRequest> for nym_vpn_lib_types::StoreAccountRequest {
    type Error = ConversionError;

    fn try_from(value: proto::StoreAccountRequest) -> Result<Self, Self::Error> {
        let request = value
            .request
            .ok_or(ConversionError::NoValueSet("StoreAccountRequest.request"))?;

        Ok(match request {
            proto::store_account_request::Request::VpnAccountStore(account) => {
                nym_vpn_lib_types::StoreAccountRequest::Vpn {
                    mnemonic: account.mnemonic,
                }
            }
            proto::store_account_request::Request::DecentralisedAccountStore(account) => {
                nym_vpn_lib_types::StoreAccountRequest::Decentralised {
                    mnemonic: account.mnemonic,
                }
            }
        })
    }
}

impl From<nym_vpn_lib_types::StoreAccountRequest> for proto::StoreAccountRequest {
    fn from(value: nym_vpn_lib_types::StoreAccountRequest) -> Self {
        let request = match value {
            nym_vpn_lib_types::StoreAccountRequest::Vpn { mnemonic } => {
                proto::store_account_request::Request::VpnAccountStore(
                    proto::VpnAccountStoreRequest { mnemonic },
                )
            }
            nym_vpn_lib_types::StoreAccountRequest::Decentralised { mnemonic } => {
                proto::store_account_request::Request::DecentralisedAccountStore(
                    proto::DecentralisedAccountStoreRequest { mnemonic },
                )
            }
        };

        proto::StoreAccountRequest {
            request: Some(request),
        }
    }
}

impl From<proto::DecentralisedObtainTicketbooksRequest>
    for nym_vpn_lib_types::DecentralisedObtainTicketbooksRequest
{
    fn from(value: proto::DecentralisedObtainTicketbooksRequest) -> Self {
        Self {
            amount: value.amount,
        }
    }
}

impl From<nym_vpn_lib_types::DecentralisedObtainTicketbooksRequest>
    for proto::DecentralisedObtainTicketbooksRequest
{
    fn from(value: nym_vpn_lib_types::DecentralisedObtainTicketbooksRequest) -> Self {
        Self {
            amount: value.amount,
        }
    }
}

impl TryFrom<proto::AccountCommandResponse> for nym_vpn_lib_types::AccountCommandResponse {
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

impl TryFrom<proto::AccountBalanceResponse> for nym_vpn_lib_types::AccountBalanceResponse {
    type Error = ConversionError;
    fn try_from(value: proto::AccountBalanceResponse) -> Result<Self, Self::Error> {
        let result = match value.account_balance.ok_or(ConversionError::NoValueSet(
            "AccountBalanceResponse.account_balance",
        ))? {
            proto::account_balance_response::AccountBalance::Error(err) => Err(err.try_into()?),
            proto::account_balance_response::AccountBalance::Balance(balance) => Ok(balance
                .coins
                .into_iter()
                .map(|c| nym_vpn_lib_types::Coin::new(c.amount as u128, c.denom))
                .collect()),
        };

        Ok(nym_vpn_lib_types::AccountBalanceResponse { result })
    }
}

impl From<nym_vpn_lib_types::AccountBalanceResponse> for proto::AccountBalanceResponse {
    fn from(value: nym_vpn_lib_types::AccountBalanceResponse) -> Self {
        let account_balance = match value.result {
            Err(err) => proto::account_balance_response::AccountBalance::Error(err.into()),
            Ok(coins) => {
                proto::account_balance_response::AccountBalance::Balance(proto::BalanceList {
                    coins: coins
                        .into_iter()
                        .map(|c| proto::Coin {
                            denom: c.denom,
                            amount: c.amount as u64,
                        })
                        .collect(),
                })
            }
        };

        proto::AccountBalanceResponse {
            account_balance: Some(account_balance),
        }
    }
}

impl From<nym_vpn_lib_types::AsnKind> for proto::AsnKind {
    fn from(value: nym_vpn_lib_types::AsnKind) -> Self {
        match value {
            nym_vpn_lib_types::AsnKind::Residential => proto::AsnKind::Residential,
            nym_vpn_lib_types::AsnKind::Other => proto::AsnKind::Other,
        }
    }
}

impl From<proto::AsnKind> for nym_vpn_lib_types::AsnKind {
    fn from(value: proto::AsnKind) -> Self {
        match value {
            proto::AsnKind::Residential => nym_vpn_lib_types::AsnKind::Residential,
            proto::AsnKind::Other => nym_vpn_lib_types::AsnKind::Other,
        }
    }
}

impl From<nym_vpn_lib_types::Asn> for proto::Asn {
    fn from(value: nym_vpn_lib_types::Asn) -> Self {
        proto::Asn {
            asn: value.asn,
            name: value.name,
            kind: proto::AsnKind::from(value.kind).into(),
        }
    }
}

impl TryFrom<proto::Asn> for nym_vpn_lib_types::Asn {
    type Error = ConversionError;

    fn try_from(value: proto::Asn) -> Result<Self, Self::Error> {
        Ok(nym_vpn_lib_types::Asn {
            asn: value.asn,
            name: value.name,
            kind: proto::AsnKind::try_from(value.kind)
                .map_err(|err| ConversionError::Decode("AsnKind", err))?
                .into(),
        })
    }
}

impl From<nym_vpn_lib_types::Location> for proto::Location {
    fn from(location: nym_vpn_lib_types::Location) -> Self {
        Self {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
            city: location.city,
            region: location.region,
            asn: location.asn.map(Into::into),
        }
    }
}

impl From<nym_vpn_lib_types::Country> for proto::Country {
    fn from(country: nym_vpn_lib_types::Country) -> Self {
        proto::Country {
            two_letter_iso_country_code: country.iso_code().to_string(),
        }
    }
}
