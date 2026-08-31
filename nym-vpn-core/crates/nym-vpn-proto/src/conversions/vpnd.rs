// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use nym_vpn_lib_types::{
    AccountCommandResponse, ApiUrl, BridgeInformation, BridgeParameters, GatewayType,
    LewesProtocolDetails, LewesProtocolDetailsData, ListGatewaysOptions, LogPath,
    NymNetworkDetails, NymVpnNetwork, Performance, QuicClientOptions, StoreAccountRequest,
    SystemMessage, TlsClientOptions, UserAgent, VpnServiceInfo,
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
            mixnet_score: proto::Score::try_from(value.score)
                .map_err(|err| ConversionError::Decode("Performance.mixnet_score", err))?
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
            mixnet_score: proto::Score::from(value.mixnet_score).into(),
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
        let socks5 = outcome.socks5.map(proto::Socks5ProbeResult::from);
        let lp = outcome.lp.map(proto::LpProbeResult::from);
        proto::ProbeOutcome {
            as_entry,
            as_exit,
            wg,
            socks5,
            lp,
        }
    }
}

impl From<nym_vpn_lib_types::Socks5> for proto::Socks5ProbeResult {
    fn from(value: nym_vpn_lib_types::Socks5) -> Self {
        Self {
            can_proxy_https: value.can_proxy_https,
            score: value
                .score
                .map(<proto::Score as From<nym_vpn_lib_types::Score>>::from)
                .map(i32::from),
            errors: value.errors.map(From::from),
        }
    }
}

impl From<Vec<String>> for proto::ErrorList {
    fn from(error: Vec<String>) -> Self {
        Self { error }
    }
}

impl From<nym_vpn_lib_types::Lp> for proto::LpProbeResult {
    fn from(value: nym_vpn_lib_types::Lp) -> Self {
        Self {
            can_connect: value.can_connect,
            can_handshake: value.can_handshake,
            can_register: value.can_register,
            error: value.error,
        }
    }
}

impl From<proto::Socks5ProbeResult> for nym_vpn_lib_types::Socks5 {
    fn from(value: proto::Socks5ProbeResult) -> Self {
        Self {
            can_proxy_https: value.can_proxy_https,
            score: value
                .score
                .and_then(|s| proto::Score::try_from(s).ok())
                .map(nym_vpn_lib_types::Score::from),
            errors: value.errors.map(|e| e.error),
        }
    }
}

impl From<proto::LpProbeResult> for nym_vpn_lib_types::Lp {
    fn from(value: proto::LpProbeResult) -> Self {
        Self {
            can_connect: value.can_connect,
            can_handshake: value.can_handshake,
            can_register: value.can_register,
            error: value.error,
        }
    }
}

impl TryFrom<proto::ProbeOutcome> for nym_vpn_lib_types::ProbeOutcome {
    type Error = ConversionError;

    fn try_from(outcome: proto::ProbeOutcome) -> Result<Self, Self::Error> {
        let as_entry = outcome
            .as_entry
            .map(nym_vpn_lib_types::Entry::from)
            .ok_or(ConversionError::generic("missing as_entry"))?;
        let as_exit = outcome.as_exit.map(nym_vpn_lib_types::Exit::from);
        Ok(Self {
            as_entry,
            as_exit,
            socks5: outcome.socks5.map(From::from),
            lp: outcome.lp.map(From::from),
        })
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

impl From<LewesProtocolDetails> for proto::LewesProtocolDetails {
    fn from(value: LewesProtocolDetails) -> Self {
        Self {
            content: Some(proto::LewesProtocolDetailsData::from(value.content)),
            signature: value.signature,
        }
    }
}

impl From<LewesProtocolDetailsData> for proto::LewesProtocolDetailsData {
    fn from(value: LewesProtocolDetailsData) -> Self {
        Self {
            enabled: value.enabled,
            control_port: value.control_port as u32,
            data_port: value.data_port as u32,
            x25519: value.x25519,
            kem_keys: value
                .kem_keys
                .into_iter()
                .map(|(k, v)| (k, proto::KemKeyDigests { digests: v }))
                .collect(),
        }
    }
}

impl TryFrom<proto::LewesProtocolDetails> for LewesProtocolDetails {
    type Error = ConversionError;

    fn try_from(value: proto::LewesProtocolDetails) -> Result<Self, Self::Error> {
        let content = value
            .content
            .ok_or_else(|| ConversionError::generic("missing LewesProtocolDetails.content"))?;
        Ok(Self {
            content: LewesProtocolDetailsData::from(content),
            signature: value.signature,
        })
    }
}

impl From<proto::LewesProtocolDetailsData> for LewesProtocolDetailsData {
    fn from(value: proto::LewesProtocolDetailsData) -> Self {
        Self {
            enabled: value.enabled,
            control_port: value.control_port as u16,
            data_port: value.data_port as u16,
            x25519: value.x25519,
            kem_keys: value
                .kem_keys
                .into_iter()
                .map(|(k, v)| (k, v.digests))
                .collect(),
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
        let location = gateway
            .location
            .map(nym_vpn_lib_types::Location::try_from)
            .transpose()?;
        let last_probe = gateway
            .last_probe
            .map(nym_vpn_lib_types::Probe::try_from)
            .transpose()?;
        let mixnet_performance = gateway.mixnet_performance.map(|x| x as u8);
        let performance = gateway.performance.map(Performance::try_from).transpose()?;
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
        let bridge_params = gateway
            .bridge_params
            .map(BridgeInformation::try_from)
            .transpose()?;
        let lp_details = gateway
            .lp_details
            .map(LewesProtocolDetails::try_from)
            .transpose()?;
        Ok(Self {
            name: gateway.name,
            description: gateway.description,
            identity_key,
            location,
            last_probe,
            performance,
            mixnet_performance,
            exit_ipv4s,
            exit_ipv6s,
            build_version,
            bridge_params,
            lewes_protocol_details: lp_details,
            node_family_name: gateway.node_family_name,
        })
    }
}

impl From<nym_vpn_lib_types::Gateway> for proto::GatewayResponse {
    fn from(gateway: nym_vpn_lib_types::Gateway) -> Self {
        let id = Some(proto::GatewayId {
            id: gateway.identity_key.to_string(),
        });
        let exit_ipv4s = gateway.exit_ipv4s.iter().map(|ip| ip.to_string()).collect();
        let exit_ipv6s = gateway.exit_ipv6s.iter().map(|ip| ip.to_string()).collect();

        proto::GatewayResponse {
            id,
            location: gateway.location.map(proto::Location::from),
            last_probe: gateway.last_probe.map(proto::Probe::from),
            mixnet_performance: gateway.mixnet_performance.map(u32::from),
            performance: gateway.performance.map(proto::Performance::from),
            name: gateway.name,
            description: gateway.description,
            exit_ipv4s,
            exit_ipv6s,
            build_version: gateway.build_version,
            bridge_params: gateway.bridge_params.map(proto::BridgeInformation::from),
            lp_details: gateway
                .lewes_protocol_details
                .map(proto::LewesProtocolDetails::from),
            node_family_name: gateway.node_family_name,
        }
    }
}

impl TryFrom<proto::TentativeGateways> for nym_vpn_lib_types::TentativeGateways {
    type Error = ConversionError;

    fn try_from(value: proto::TentativeGateways) -> Result<Self, Self::Error> {
        match value
            .tentative_gateways
            .ok_or_else(|| ConversionError::generic("missing tentative gateways"))?
        {
            proto::tentative_gateways::TentativeGateways::Selected(
                proto::tentative_gateways::Selected { entry, exit },
            ) => {
                let entry = entry
                    .ok_or_else(|| ConversionError::generic("missing selected entry gateway"))?;
                let exit =
                    exit.ok_or_else(|| ConversionError::generic("missing selected exit gateway"))?;
                Ok(nym_vpn_lib_types::TentativeGateways::Selected {
                    entry: Box::new(entry.try_into()?),
                    exit: Box::new(exit.try_into()?),
                })
            }
            proto::tentative_gateways::TentativeGateways::NeedsRelaxedIndependenceCriteria(_) => {
                Ok(nym_vpn_lib_types::TentativeGateways::NeedsRelaxedIndependenceCriteria)
            }
            proto::tentative_gateways::TentativeGateways::NoGatewaysAvailable(_) => {
                Ok(nym_vpn_lib_types::TentativeGateways::NoGatewaysAvailable)
            }
        }
    }
}

impl From<nym_vpn_lib_types::TentativeGateways> for proto::TentativeGateways {
    fn from(value: nym_vpn_lib_types::TentativeGateways) -> Self {
        match value {
            nym_vpn_lib_types::TentativeGateways::Selected { entry, exit } => {
                proto::TentativeGateways {
                    tentative_gateways: Some(
                        proto::tentative_gateways::TentativeGateways::Selected(
                            proto::tentative_gateways::Selected {
                                entry: Some(proto::GatewayResponse::from(*entry)),
                                exit: Some(proto::GatewayResponse::from(*exit)),
                            },
                        ),
                    ),
                }
            }
            nym_vpn_lib_types::TentativeGateways::NeedsRelaxedIndependenceCriteria => proto::TentativeGateways {
                tentative_gateways: Some(
                    proto::tentative_gateways::TentativeGateways::NeedsRelaxedIndependenceCriteria(
                        proto::tentative_gateways::NeedsRelaxedIndependenceCriteria {}
                    ),
                ),
            },
            nym_vpn_lib_types::TentativeGateways::NoGatewaysAvailable => proto::TentativeGateways {
                tentative_gateways: Some(
                    proto::tentative_gateways::TentativeGateways::NoGatewaysAvailable(
                        proto::tentative_gateways::NoGatewaysAvailable {},
                    ),
                ),
            },
        }
    }
}

impl From<nym_vpn_lib_types::RecentGateways> for proto::RecentGateways {
    fn from(value: nym_vpn_lib_types::RecentGateways) -> Self {
        Self {
            entry: value.entry.into_iter().map(Into::into).collect(),
            exit: value.exit.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<proto::RecentGateways> for nym_vpn_lib_types::RecentGateways {
    type Error = ConversionError;

    fn try_from(value: proto::RecentGateways) -> Result<Self, Self::Error> {
        Ok(Self {
            entry: value
                .entry
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, Self::Error>>()?,
            exit: value
                .exit
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, Self::Error>>()?,
        })
    }
}

impl From<BridgeInformation> for proto::BridgeInformation {
    fn from(value: BridgeInformation) -> Self {
        let transports = value
            .transports
            .into_iter()
            .map(proto::BridgeParameters::from)
            .collect::<Vec<_>>();

        Self {
            version: value.version,
            transports,
        }
    }
}

impl TryFrom<proto::BridgeInformation> for BridgeInformation {
    type Error = ConversionError;

    fn try_from(value: proto::BridgeInformation) -> Result<Self, Self::Error> {
        let transports = value
            .transports
            .into_iter()
            .map(BridgeParameters::try_from)
            .collect::<Result<Vec<BridgeParameters>, ConversionError>>()?;

        Ok(Self {
            version: value.version,
            transports,
        })
    }
}

impl From<QuicClientOptions> for proto::QuicClientOptions {
    fn from(value: QuicClientOptions) -> Self {
        Self {
            addresses: value
                .addresses
                .into_iter()
                .map(proto::SocketAddr::from)
                .collect(),
            host: value.host,
            id_pubkey: value.id_pubkey,
        }
    }
}

impl TryFrom<proto::QuicClientOptions> for QuicClientOptions {
    type Error = ConversionError;

    fn try_from(value: proto::QuicClientOptions) -> Result<Self, Self::Error> {
        let addresses = value
            .addresses
            .into_iter()
            .map(SocketAddr::try_from)
            .collect::<Result<Vec<SocketAddr>, ConversionError>>()?;
        Ok(Self {
            host: value.host,
            id_pubkey: value.id_pubkey,
            addresses,
        })
    }
}

impl From<TlsClientOptions> for proto::TlsClientOptions {
    fn from(value: TlsClientOptions) -> Self {
        Self {
            addresses: value
                .addresses
                .into_iter()
                .map(proto::SocketAddr::from)
                .collect(),
            host: value.host,
            id_pubkey: value.id_pubkey,
        }
    }
}

impl TryFrom<proto::TlsClientOptions> for TlsClientOptions {
    type Error = ConversionError;

    fn try_from(value: proto::TlsClientOptions) -> Result<Self, Self::Error> {
        let addresses = value
            .addresses
            .into_iter()
            .map(SocketAddr::try_from)
            .collect::<Result<Vec<SocketAddr>, ConversionError>>()?;
        Ok(Self {
            host: value.host,
            id_pubkey: value.id_pubkey,
            addresses,
        })
    }
}

impl From<BridgeParameters> for proto::BridgeParameters {
    fn from(value: BridgeParameters) -> Self {
        match value {
            BridgeParameters::QuicPlain(options) => proto::BridgeParameters {
                state: Some(proto::bridge_parameters::State::QuicPlain(
                    proto::QuicClientOptions::from(options),
                )),
            },
            BridgeParameters::TlsPlain(options) => proto::BridgeParameters {
                state: Some(proto::bridge_parameters::State::TlsPlain(
                    proto::TlsClientOptions::from(options),
                )),
            },
        }
    }
}

impl TryFrom<proto::BridgeParameters> for BridgeParameters {
    type Error = ConversionError;

    fn try_from(value: proto::BridgeParameters) -> Result<Self, Self::Error> {
        let state = value
            .state
            .ok_or(ConversionError::NoValueSet("BridgeParameters.state"))?;
        Ok(match state {
            proto::bridge_parameters::State::QuicPlain(options) => {
                BridgeParameters::QuicPlain(QuicClientOptions::try_from(options)?)
            }
            proto::bridge_parameters::State::TlsPlain(options) => {
                BridgeParameters::TlsPlain(TlsClientOptions::try_from(options)?)
            }
        })
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
                let nym_vpn_api_urls = s
                    .nym_vpn_api_urls
                    .into_iter()
                    .map(ApiUrl::from)
                    .collect::<Vec<_>>();
                NymVpnNetwork { nym_vpn_api_urls }
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

impl From<proto::LogPath> for nym_vpn_lib_types::LogPath {
    fn from(value: proto::LogPath) -> Self {
        Self {
            dir: PathBuf::from(value.dir),
            filename: value.filename,
        }
    }
}

impl TryFrom<LogPath> for proto::LogPath {
    type Error = ConversionError;

    fn try_from(value: LogPath) -> Result<Self, Self::Error> {
        Ok(Self {
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

impl TryFrom<proto::StoreAccountRequest> for StoreAccountRequest {
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
            proto::store_account_request::Request::PrivyAccountStore(account) => {
                nym_vpn_lib_types::StoreAccountRequest::Privy {
                    mnemonic: account.mnemonic,
                }
            }
        })
    }
}

impl From<StoreAccountRequest> for proto::StoreAccountRequest {
    fn from(value: StoreAccountRequest) -> Self {
        let request = match value {
            StoreAccountRequest::Vpn { mnemonic } => {
                proto::store_account_request::Request::VpnAccountStore(
                    proto::VpnAccountStoreRequest { mnemonic },
                )
            }
            StoreAccountRequest::Decentralised { mnemonic } => {
                proto::store_account_request::Request::DecentralisedAccountStore(
                    proto::DecentralisedAccountStoreRequest { mnemonic },
                )
            }
            StoreAccountRequest::Privy { mnemonic } => {
                proto::store_account_request::Request::PrivyAccountStore(
                    proto::PrivyAccountStoreRequest { mnemonic },
                )
            }
        };

        proto::StoreAccountRequest {
            request: Some(request),
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
        let value = value.account_balance.ok_or(ConversionError::NoValueSet(
            "AccountBalanceResponse.account_balance",
        ))?;

        match value {
            proto::account_balance_response::AccountBalance::Error(err) => {
                Ok(nym_vpn_lib_types::AccountBalanceResponse {
                    result: Err(err.try_into()?),
                })
            }
            proto::account_balance_response::AccountBalance::Balance(balance) => {
                let coins = balance
                    .coins
                    .into_iter()
                    .map(|c| {
                        let amount = u128::from_str(&c.amount)
                            .map_err(|e| ConversionError::ParseInteger("Coin.amount", e))?;
                        Ok(nym_vpn_lib_types::Coin::new(amount, c.denom))
                    })
                    .collect::<Result<Vec<_>, ConversionError>>()?;
                Ok(nym_vpn_lib_types::AccountBalanceResponse { result: Ok(coins) })
            }
        }
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
                            amount: c.amount.to_string(),
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
