// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(test)]
mod tests;

use ipnetwork::IpNetwork;
use itertools::Itertools;
use nym_sdk::mixnet::NodeIdentity;
use nym_topology::{NodeId, RoutingNode};
use nym_validator_client::models::{
    KeyRotationId,
    described::{type_translation::LewesProtocolDetailsV1, v2::NymNodeDescriptionV2},
};
use nym_vpn_api_client::{
    response::{BridgeInformation, BridgeParameters, NodeFamily, NodeStaking},
    types::Percent,
};
use rand::seq::IteratorRandom;
use std::{
    cmp::Ordering,
    collections::HashSet,
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    str::FromStr,
};
use typed_builder::TypedBuilder;

use crate::{
    AuthAddress, BlacklistedGateways, Country, EntryPoint, Error, ExitPoint, IpPacketRouterAddress,
    error::Result, helpers,
};

pub type NymNode = Gateway;

pub const COUNTRY_WITH_REGION_SELECTOR: &str = "US";

#[derive(Clone, Debug, TypedBuilder)]
pub struct Gateway {
    pub identity: NodeIdentity,
    #[builder(default="".to_owned())]
    pub name: String,
    #[builder(default)]
    pub description: Option<String>,
    #[builder(default, setter(strip_option))]
    pub location: Option<Location>,
    #[builder(default, setter(strip_option))]
    pub ipr_address: Option<IpPacketRouterAddress>,
    #[builder(default, setter(strip_option))]
    pub authenticator_address: Option<AuthAddress>,
    #[builder(default)]
    pub nr_address: Option<String>,
    #[builder(default)]
    pub bridge_params: Option<BridgeInformation>,
    #[builder(default)]
    pub last_probe: Option<Probe>,
    #[builder(default=vec![])]
    pub ips: Vec<IpAddr>,
    #[builder(default)]
    pub host: Option<String>,
    #[builder(default)]
    pub clients_ws_port: Option<u16>,
    #[builder(default)]
    pub clients_wss_port: Option<u16>,
    // todo: remove since it's unused?
    #[builder(default)]
    pub mixnet_performance: Option<Percent>,
    #[builder(default, setter(strip_option))]
    pub performance: Option<Performance>,
    #[builder(default)]
    pub version: Option<String>,
    #[builder(default)]
    pub lewes_protocol_details: Option<LewesProtocolDetailsV1>,
    #[builder(default)]
    pub staking_data: Option<NodeStaking>,
    #[builder(default)]
    pub family_data: Option<NodeFamily>,
}

impl Gateway {
    pub fn try_from_node_description(
        node_description: NymNodeDescriptionV2,
        current_key_rotation: KeyRotationId,
    ) -> Result<Self> {
        let identity = node_description.description.host_information.keys.ed25519;
        let location = node_description
            .description
            .auxiliary_details
            .location
            .map(|l| Location {
                two_letter_iso_country_code: l.alpha2.to_string(),
                ..Default::default()
            });
        let ipr_address = node_description
            .description
            .ip_packet_router
            .as_ref()
            .and_then(|ipr| {
                IpPacketRouterAddress::try_from_base58_string(&ipr.address)
                    .inspect_err(|err| tracing::error!("Failed to parse IPR address: {err}"))
                    .ok()
            });
        let authenticator_address = node_description
            .description
            .authenticator
            .as_ref()
            .and_then(|a| {
                AuthAddress::try_from_base58_string(&a.address)
                    .inspect_err(|err| {
                        tracing::error!("Failed to parse authenticator address: {err}")
                    })
                    .ok()
            });
        let nr_address = node_description
            .description
            .network_requester
            .as_ref()
            .map(|nr| nr.address.clone());

        let lewes_protocol_details = node_description.description.lewes_protocol.clone();

        let version = Some(node_description.version().to_string());
        let role = if node_description.description.declared_role.entry {
            nym_validator_client::nym_nodes::NodeRole::EntryGateway
        } else if node_description.description.declared_role.exit_ipr
            || node_description.description.declared_role.exit_nr
        {
            nym_validator_client::nym_nodes::NodeRole::ExitGateway
        } else {
            nym_validator_client::nym_nodes::NodeRole::Inactive
        };

        let gateway = RoutingNode::try_from(&node_description.to_skimmed_node(
            current_key_rotation,
            role,
            Default::default(),
        ))
        .map_err(|_| Error::MalformedGateway)?;

        let host = gateway.ws_entry_address(false);
        let entry_info = &gateway.entry;
        let clients_ws_port = entry_info.as_ref().map(|g| g.clients_ws_port);
        let clients_wss_port = entry_info.as_ref().and_then(|g| g.clients_wss_port);
        let ips = node_description.description.host_information.ip_address;
        Ok(Gateway {
            identity,
            name: "".to_owned(),
            description: None,
            location,
            ipr_address,
            authenticator_address,
            nr_address,
            bridge_params: None,
            last_probe: None,
            ips,
            host,
            clients_ws_port,
            clients_wss_port,
            mixnet_performance: None,
            performance: None,
            version,
            lewes_protocol_details,
            staking_data: None,
            family_data: None,
        })
    }

    pub fn identity(&self) -> NodeIdentity {
        self.identity
    }

    pub fn two_letter_iso_country_code(&self) -> Option<&str> {
        self.location
            .as_ref()
            .map(|l| l.two_letter_iso_country_code.as_str())
    }

    pub fn is_in_country(&self, two_letter_iso_country_code: &str) -> bool {
        self.location
            .as_ref()
            .is_some_and(|v| v.two_letter_iso_country_code == two_letter_iso_country_code)
    }

    pub fn region(&self) -> Option<&str> {
        self.location.as_ref().map(|l| l.region.as_str())
    }

    pub fn is_in_region(&self, region: &str) -> bool {
        self.location.as_ref().is_some_and(|v| v.region == region)
    }

    pub fn is_residential_asn(&self) -> bool {
        self.location.as_ref().is_some_and(|v| {
            v.asn
                .as_ref()
                .is_some_and(|v| v.kind == AsnKind::Residential)
        })
    }

    pub fn is_quic_enabled(&self) -> bool {
        self.get_bridge_params()
            .map(|bp| matches!(bp, BridgeParameters::QuicPlain(_)))
            .unwrap_or(false)
    }

    pub fn is_exit_node(&self) -> bool {
        self.ipr_address.is_some()
    }

    pub fn is_vpn_node(&self) -> bool {
        self.authenticator_address.is_some()
    }

    pub fn is_whitelisted(&self, blacklisted_gateways: &BlacklistedGateways) -> bool {
        match blacklisted_gateways.exists(&self.identity) {
            Ok(exists) => !exists,
            Err(e) => {
                tracing::error!("Error testing gateway whitelisting: {e}");
                false
            }
        }
    }

    pub fn host(&self) -> Option<&String> {
        self.host.as_ref()
    }

    pub fn lookup_ip(&self) -> Option<IpAddr> {
        self.ips.first().copied()
    }

    pub fn split_ips(&self) -> (Vec<Ipv4Addr>, Vec<Ipv6Addr>) {
        helpers::split_ips(self.ips.clone())
    }

    pub fn clients_address_no_tls(&self) -> Option<String> {
        match (&self.host, &self.clients_ws_port) {
            (Some(host), Some(port)) => Some(format!("ws://{host}:{port}")),
            _ => None,
        }
    }

    pub fn clients_address_tls(&self) -> Option<String> {
        match (&self.host, &self.clients_wss_port) {
            (Some(host), Some(port)) => Some(format!("wss://{host}:{port}")),
            _ => None,
        }
    }

    pub fn meets_score(&self, gw_type: Option<GatewayType>, min_score: ScoreValue) -> bool {
        match gw_type {
            Some(GatewayType::MixnetEntry) | Some(GatewayType::MixnetExit) => self
                .performance
                .as_ref()
                .is_some_and(|p| p.mixnet_score >= min_score),
            Some(GatewayType::Wg) => self
                .performance
                .as_ref()
                .is_some_and(|p| p.score >= min_score),
            None => false,
        }
    }

    pub fn meets_socks5_score(&self, min_socks5_score: ScoreValue) -> bool {
        self.last_probe
            .as_ref()
            .and_then(|probe| {
                probe
                    .outcome
                    .socks5
                    .as_ref()
                    .and_then(|socks5| socks5.score.as_ref())
            })
            .is_some_and(|score| *score >= min_socks5_score)
    }

    pub fn not_mixnet_blacklisted(&self) -> bool {
        // Currently the mixnet blacklisting threshold is 50%, so let's take a slightly bigger number
        // in case of caching differences between VPN API and mixnet API
        self.mixnet_performance
            .as_ref()
            .is_some_and(|p| p.round_to_integer() > 55)
    }

    /// Tests whether the gateway matches a specific filter.
    pub fn matches_filter(&self, gw_type: Option<GatewayType>, filter: &GatewayFilter) -> bool {
        match filter {
            GatewayFilter::MinScore(score) => self.meets_score(gw_type, *score),
            GatewayFilter::MinSocks5Score(score) => self.meets_socks5_score(*score),
            GatewayFilter::Country(code) => self.is_in_country(code),
            GatewayFilter::Region(region) => self.is_in_region(region),
            GatewayFilter::Residential => self.is_residential_asn(),
            GatewayFilter::QuicEnabled => self.is_quic_enabled(),
            GatewayFilter::Exit => self.is_exit_node(),
            GatewayFilter::Vpn => self.is_vpn_node(),
            GatewayFilter::NotBlacklisted(blacklisted_gateways) => {
                self.is_whitelisted(blacklisted_gateways)
            }
        }
    }

    /// Tests whether the gateway matches all the filters.
    pub fn matches_all_filters(
        &self,
        gw_type: Option<GatewayType>,
        filters: &GatewayFilters,
    ) -> bool {
        filters
            .iter()
            .all(|filter| self.matches_filter(gw_type, filter))
    }

    pub fn get_bridge_params(&self) -> Option<BridgeParameters> {
        if let Some(all_params) = &self.bridge_params {
            all_params.transports.first().cloned()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AsnKind {
    Residential,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Asn {
    pub asn: String,
    pub name: String,
    pub route: IpNetwork,
    pub kind: AsnKind,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Location {
    pub two_letter_iso_country_code: String,
    pub latitude: f64,
    pub longitude: f64,

    pub city: String,
    pub region: String,

    pub asn: Option<Asn>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScoreValue {
    Offline,
    Low,
    Medium,
    High,
}

impl ScoreValue {
    fn priority(&self) -> u8 {
        match self {
            ScoreValue::Offline => 0,
            ScoreValue::Low => 1,
            ScoreValue::Medium => 2,
            ScoreValue::High => 3,
        }
    }
}

impl PartialOrd for ScoreValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.priority().cmp(&other.priority()))
    }
}

impl FromStr for ScoreValue {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "offline" => Ok(ScoreValue::Offline),
            "low" => Ok(ScoreValue::Low),
            "medium" => Ok(ScoreValue::Medium),
            "high" => Ok(ScoreValue::High),
            _ => Err(Error::InvalidScoreValue(s.to_string())),
        }
    }
}

impl fmt::Display for ScoreValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ScoreValue::Offline => "Offline",
            ScoreValue::Low => "Low",
            ScoreValue::Medium => "Medium",
            ScoreValue::High => "High",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Performance {
    pub last_updated_utc: String,
    /// WireGuard performance score
    pub score: ScoreValue,
    /// Mixnet performance score
    pub mixnet_score: ScoreValue,
    pub load: ScoreValue,
    pub uptime_percentage_last_24_hours: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Probe {
    pub last_updated_utc: String,
    pub outcome: ProbeOutcome,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeOutcome {
    pub as_entry: Entry,
    pub as_exit: Option<Exit>,
    pub wg: Option<WgProbeResults>,
    pub socks5: Option<Socks5>,
    pub lp: Option<Lp>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Socks5 {
    pub can_proxy_https: bool,
    pub score: Option<ScoreValue>,
    pub errors: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lp {
    pub can_connect: bool,
    pub can_handshake: bool,
    pub can_register: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub can_connect: bool,
    pub can_route: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Exit {
    pub can_connect: bool,
    pub can_route_ip_v4: bool,
    pub can_route_ip_external_v4: bool,
    pub can_route_ip_v6: bool,
    pub can_route_ip_external_v6: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WgProbeResults {
    pub can_register: bool,
    pub can_handshake: bool,
    pub can_resolve_dns: bool,
    pub can_query_metadata_v4: bool,
    pub ping_hosts_performance: f32,
    pub ping_ips_performance: f32,
}

impl From<nym_vpn_api_client::response::AsnKind> for AsnKind {
    fn from(value: nym_vpn_api_client::response::AsnKind) -> Self {
        match value {
            nym_vpn_api_client::response::AsnKind::Residential => AsnKind::Residential,
            nym_vpn_api_client::response::AsnKind::Other => AsnKind::Other,
        }
    }
}

impl TryFrom<nym_vpn_api_client::response::Asn> for Asn {
    type Error = Error;

    fn try_from(
        location: nym_vpn_api_client::response::Asn,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Asn {
            asn: location.asn,
            name: location.name,
            route: location
                .route
                .parse()
                .map_err(|source| Error::AsnRouteFormattingError {
                    route: location.route,
                    source,
                })?,
            kind: location.kind.into(),
        })
    }
}

impl TryFrom<nym_vpn_api_client::response::Location> for Location {
    type Error = Error;

    fn try_from(
        location: nym_vpn_api_client::response::Location,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Location {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
            city: location.city,
            region: location.region,
            asn: location.asn.map(TryInto::try_into).transpose()?,
        })
    }
}

impl From<nym_vpn_api_client::response::ScoreValue> for ScoreValue {
    fn from(value: nym_vpn_api_client::response::ScoreValue) -> Self {
        match value {
            nym_vpn_api_client::response::ScoreValue::Offline => ScoreValue::Offline,
            nym_vpn_api_client::response::ScoreValue::Low => ScoreValue::Low,
            nym_vpn_api_client::response::ScoreValue::Medium => ScoreValue::Medium,
            nym_vpn_api_client::response::ScoreValue::High => ScoreValue::High,
        }
    }
}

impl From<nym_vpn_api_client::response::DVpnGatewayPerformance> for Performance {
    fn from(value: nym_vpn_api_client::response::DVpnGatewayPerformance) -> Self {
        Performance {
            last_updated_utc: value.last_updated_utc,
            score: value.score.into(),
            mixnet_score: value.mixnet_score.into(),
            load: value.load.into(),
            uptime_percentage_last_24_hours: value.uptime_percentage_last_24_hours,
        }
    }
}

impl From<nym_vpn_api_client::response::Probe> for Probe {
    fn from(probe: nym_vpn_api_client::response::Probe) -> Self {
        Probe {
            last_updated_utc: probe.last_updated_utc,
            outcome: ProbeOutcome::from(probe.outcome),
        }
    }
}

impl From<nym_vpn_api_client::response::ProbeOutcome> for ProbeOutcome {
    fn from(outcome: nym_vpn_api_client::response::ProbeOutcome) -> Self {
        ProbeOutcome {
            as_entry: Entry::from(outcome.as_entry),
            as_exit: outcome.as_exit.map(Exit::from),
            wg: outcome.wg.map(WgProbeResults::from),
            socks5: outcome.socks5.map(From::from),
            lp: outcome.lp.map(From::from),
        }
    }
}

impl From<nym_vpn_api_client::response::Socks5> for Socks5 {
    fn from(exit: nym_vpn_api_client::response::Socks5) -> Self {
        Self {
            can_proxy_https: exit.can_proxy_https,
            score: exit.score.map(ScoreValue::from),
            errors: exit.errors,
        }
    }
}

impl From<nym_vpn_api_client::response::Lp> for Lp {
    fn from(lp: nym_vpn_api_client::response::Lp) -> Self {
        Self {
            can_connect: lp.can_connect,
            can_handshake: lp.can_handshake,
            can_register: lp.can_register,
            error: lp.error,
        }
    }
}

impl From<nym_vpn_api_client::response::Entry> for Entry {
    fn from(entry: nym_vpn_api_client::response::Entry) -> Self {
        Entry {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<nym_vpn_api_client::response::Exit> for Exit {
    fn from(exit: nym_vpn_api_client::response::Exit) -> Self {
        Exit {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

impl From<nym_vpn_api_client::response::WgProbeResults> for WgProbeResults {
    fn from(results: nym_vpn_api_client::response::WgProbeResults) -> Self {
        WgProbeResults {
            can_register: results.can_register,
            can_handshake: results.can_handshake,
            can_resolve_dns: results.can_resolve_dns,
            can_query_metadata_v4: results.can_query_metadata_v4.unwrap_or(false),
            ping_hosts_performance: results.ping_hosts_performance,
            ping_ips_performance: results.ping_ips_performance,
        }
    }
}

impl TryFrom<nym_vpn_api_client::response::NymDirectoryGateway> for Gateway {
    type Error = Error;

    fn try_from(gateway: nym_vpn_api_client::response::NymDirectoryGateway) -> Result<Self> {
        let identity =
            NodeIdentity::from_base58_string(&gateway.identity_key).map_err(|source| {
                Error::NodeIdentityFormattingError {
                    identity: gateway.identity_key,
                    source,
                }
            })?;

        let ipr_address = gateway
            .ip_packet_router
            .and_then(|ipr| IpPacketRouterAddress::try_from_base58_string(&ipr.address).ok());

        let authenticator_address = gateway
            .authenticator
            .and_then(|auth| AuthAddress::try_from_base58_string(&auth.address).ok());

        let hostname = gateway.entry.hostname;
        let first_ip_address = gateway
            .ip_addresses
            .first()
            .cloned()
            .map(|ip| ip.to_string());
        let host = hostname.or(first_ip_address);

        let mut last_probe = gateway.last_probe.map(Probe::from);

        let performance = gateway.performance_v2.map(Performance::from);

        // If the SOCKS5 score is not available then take it from the mixnet score of the last probe.
        let socks5_score_from_mixnet =
            |socks5: Option<Socks5>, performance: Option<&Performance>| -> Option<Socks5> {
                if let Some(ref s) = socks5
                    && s.score.is_some()
                {
                    // Already have a score, nothing to do here
                    return socks5;
                }

                let performance = performance?;
                let mixnet_score = Some(performance.mixnet_score);

                match socks5 {
                    Some(mut s) => {
                        s.score = mixnet_score;
                        Some(s)
                    }
                    None => Some(Socks5 {
                        can_proxy_https: false,
                        score: mixnet_score,
                        errors: None,
                    }),
                }
            };

        if let Some(probe) = last_probe.as_mut() {
            probe.outcome.socks5 =
                socks5_score_from_mixnet(probe.outcome.socks5.clone(), performance.as_ref());
        }

        Ok(Gateway {
            identity,
            name: gateway.name,
            description: gateway.description,
            location: Some(gateway.location.try_into()?),
            ipr_address,
            authenticator_address,
            nr_address: None,
            bridge_params: gateway.bridges,
            last_probe,
            ips: gateway.ip_addresses,
            host,
            clients_ws_port: Some(gateway.entry.ws_port),
            clients_wss_port: gateway.entry.wss_port,
            mixnet_performance: Some(gateway.performance),
            performance,
            version: gateway.build_information.map(|info| info.build_version),
            lewes_protocol_details: gateway.lewes_protocol_details,
            staking_data: gateway.staking_data,
            family_data: gateway.family_data,
        })
    }
}

/// Convert a raw nym-vpn-api gateway list into a filtered [`GatewayList`], applying the same
/// per-gateway conversion and mixnet-blacklist filtering regardless of whether the raw data came
/// from a live fetch, the builtin snapshot, or the on-disk gateway cache.
pub(crate) fn gateways_from_raw(
    raw: Vec<nym_vpn_api_client::response::NymDirectoryGateway>,
    gw_type: GatewayType,
) -> GatewayList {
    let gateways: Vec<_> = raw
        .into_iter()
        .filter_map(|gw| {
            Gateway::try_from(gw)
                .inspect_err(|err| tracing::error!("Failed to parse gateway: {err}"))
                .ok()
        })
        .filter(Gateway::not_mixnet_blacklisted)
        .collect();

    GatewayList::new(Some(gw_type), gateways)
}

pub type NymNodeList = GatewayList;

#[derive(Debug, Clone)]
pub struct GatewayList {
    /// If None, then the list contains mixed types.
    gw_type: Option<GatewayType>,
    gateways: Vec<Gateway>,
}

impl GatewayList {
    pub fn new(gw_type: Option<GatewayType>, gateways: Vec<Gateway>) -> Self {
        GatewayList { gw_type, gateways }
    }

    // Returns a list of all locations of the gateways, including duplicates
    fn all_locations(&self) -> impl Iterator<Item = &Location> {
        self.gateways
            .iter()
            .filter_map(|gateway| gateway.location.as_ref())
    }

    pub fn all_countries(&self) -> Vec<Country> {
        self.all_locations()
            .cloned()
            .map(Country::from)
            .unique()
            .collect()
    }

    pub fn all_iso_codes(&self) -> Vec<String> {
        self.all_countries()
            .into_iter()
            .map(|country| country.iso_code().to_string())
            .collect()
    }

    pub fn filter(&self, filters: &GatewayFilters) -> Vec<Gateway> {
        self.gateways
            .iter()
            .filter(|gateway| gateway.matches_all_filters(self.gw_type, filters))
            .cloned()
            .collect()
    }

    pub fn node_with_identity(&self, identity: &NodeIdentity) -> Option<&NymNode> {
        // Not using self.filter() here as find() will stop at the first match
        self.gateways
            .iter()
            .find(|node| &node.identity() == identity)
    }

    pub fn gateway_with_identity(&self, identity: &NodeIdentity) -> Option<&Gateway> {
        self.node_with_identity(identity)
    }

    pub fn gateway_with_identity_filtered(
        &self,
        identity: &NodeIdentity,
        filters: &GatewayFilters,
    ) -> Option<Gateway> {
        self.filter(filters)
            .iter()
            .find(|node| &node.identity() == identity)
            .cloned()
    }

    pub fn choose_random(&self, filters: &GatewayFilters) -> Option<Gateway> {
        self.filter(filters)
            .into_iter()
            .choose(&mut rand::thread_rng())
    }

    pub fn filtered_min_by<F>(&self, filters: &GatewayFilters, cmp: F) -> Option<Gateway>
    where
        F: FnMut(&Gateway, &Gateway) -> Ordering,
    {
        self.filter(filters).into_iter().min_by(cmp)
    }

    pub fn retain_gateways_by<F>(&mut self, pred: F)
    where
        F: FnMut(&Gateway) -> bool,
    {
        self.gateways.retain(pred);
    }

    pub fn gw_type(&self) -> Option<GatewayType> {
        self.gw_type
    }

    pub fn len(&self) -> usize {
        self.gateways.len()
    }

    pub fn is_empty(&self) -> bool {
        self.gateways.is_empty()
    }

    pub fn into_exit_gateways(self) -> GatewayList {
        Self::new(
            self.gw_type,
            self.filter(&GatewayFilters::from(&[GatewayFilter::Exit])),
        )
    }

    pub fn into_vpn_gateways(self) -> GatewayList {
        Self::new(
            self.gw_type,
            self.filter(&GatewayFilters::from(&[GatewayFilter::Vpn])),
        )
    }

    pub fn into_inner(self) -> Vec<Gateway> {
        self.gateways
    }

    /// Find an entry gateway that matches `entry_point`.
    pub fn find_entry_gateway(
        &self,
        entry_point: &EntryPoint,
        base_filters: &GatewayFilters,
        optional_filters: &GatewayFilters,
    ) -> Result<Gateway> {
        match &entry_point {
            EntryPoint::Gateway { identity } => {
                tracing::debug!("Selecting gateway by identity: {identity}");
                self.gateway_with_identity(identity)
                    .ok_or_else(|| Error::NoMatchingGateway {
                        requested_identity: identity.to_string(),
                    })
                    .cloned()
            }
            EntryPoint::Country {
                two_letter_iso_country_code,
            } => {
                tracing::debug!(
                    "Selecting entry gateway by country: {two_letter_iso_country_code}"
                );

                let filters = base_filters
                    .with(optional_filters.iter())
                    .with(&[GatewayFilter::Country(two_letter_iso_country_code.clone())]);

                self.choose_random(&filters).ok_or_else(|| {
                    Error::NoMatchingEntryGatewayForLocation {
                        requested_location: two_letter_iso_country_code.clone(),
                        available_countries: self.all_iso_codes(),
                    }
                })
            }
            EntryPoint::Region { region } => {
                tracing::debug!("Selecting entry gateway by region/state: {region}");

                // Currently only supported in the US
                let filters = base_filters.with(optional_filters.iter()).with(&[
                    GatewayFilter::Country(COUNTRY_WITH_REGION_SELECTOR.to_string()),
                    GatewayFilter::Region(region.to_string()),
                ]);

                self.choose_random(&filters).ok_or_else(|| {
                    Error::NoMatchingEntryGatewayForLocation {
                        requested_location: region.clone(),
                        available_countries: self.all_iso_codes(),
                    }
                })
            }
            EntryPoint::Random => {
                tracing::debug!("Selecting a random entry gateway");

                let filters = base_filters.with(optional_filters.iter());

                self.choose_random(&filters)
                    .ok_or_else(|| Error::FailedToSelectGatewayRandomly)
            }
        }
    }

    /// Find the "best" entry gateway that matches `entry_point` using a descending score system.
    /// If no gateway matches the highest score, it will try the next lower score, and so on.
    pub fn find_best_entry_point_gateway(
        &self,
        entry_point: &EntryPoint,
        base_filters: &GatewayFilters,
    ) -> Result<Gateway> {
        for score in [ScoreValue::High, ScoreValue::Medium, ScoreValue::Low] {
            tracing::debug!("Looking for entry gateway with minimum score: {score}");

            let optional_filters = GatewayFilters::from(&[GatewayFilter::MinScore(score)]);

            match self.find_entry_gateway(entry_point, base_filters, &optional_filters) {
                Ok(gateway) => {
                    return Ok(gateway);
                }
                Err(err) => {
                    if !err.is_unmatched_non_specific_gateway() {
                        return Err(err);
                    }
                    // continue
                }
            }
        }
        match entry_point {
            EntryPoint::Gateway { identity } => Err(Error::NoMatchingGateway {
                requested_identity: identity.to_string(),
            }),
            EntryPoint::Country {
                two_letter_iso_country_code,
            } => Err(Error::NoMatchingEntryGatewayForLocation {
                requested_location: two_letter_iso_country_code.clone(),
                available_countries: self.all_iso_codes(),
            }),
            EntryPoint::Region { region } => Err(Error::NoMatchingEntryGatewayForLocation {
                requested_location: region.clone(),
                available_countries: self.all_iso_codes(),
            }),
            EntryPoint::Random => Err(Error::FailedToSelectGatewayRandomly),
        }
    }

    /// Find the "best" gateway with a given ordering criteria, using a descending score system.
    /// If no gateway matches the highest score, it will try the next lower score, and so on.
    pub fn find_best_ordering_criteria_gateway<F>(
        &self,
        mut ordering_criteria: F,
        base_filters: &GatewayFilters,
    ) -> Result<Gateway>
    where
        F: FnMut(&Gateway, &Gateway) -> Ordering,
    {
        for score in [ScoreValue::High, ScoreValue::Medium, ScoreValue::Low] {
            tracing::debug!("Looking for gateway with minimum score: {score}");

            let filters = base_filters.with(&[GatewayFilter::MinScore(score)]);

            if let Some(gateway) = self.filtered_min_by(&filters, &mut ordering_criteria) {
                return Ok(gateway);
            }
        }
        Err(Error::FailedToSelectGatewayWithCriteria)
    }

    /// Find an exit gateway that matches `exit_point`.
    pub fn find_exit_gateway(
        &self,
        exit_point: &ExitPoint,
        base_filters: &GatewayFilters,
    ) -> Result<Gateway> {
        match &exit_point {
            ExitPoint::Address { address } => {
                tracing::debug!("Selecting gateway by address: {address}");
                // There is no validation done when a ip packet router is specified by address
                // since it might be private and not available in any directory.
                let ipr_address = IpPacketRouterAddress::from(**address);
                let gateway_address = ipr_address.gateway();

                // Now fetch the gateway that the IPR is connected to, and override its IPR address
                let mut gateway = self
                    .gateway_with_identity(&gateway_address)
                    .ok_or_else(|| Error::NoMatchingGateway {
                        requested_identity: gateway_address.to_string(),
                    })
                    .cloned()?;
                gateway.ipr_address = Some(ipr_address);
                Ok(gateway)
            }
            ExitPoint::Gateway { identity } => {
                tracing::debug!("Selecting exit gateway by identity: {identity}");
                self.gateway_with_identity(identity)
                    .ok_or_else(|| Error::NoMatchingGateway {
                        requested_identity: identity.to_string(),
                    })
                    .cloned()
            }
            ExitPoint::Country {
                two_letter_iso_country_code,
            } => {
                tracing::debug!("Selecting exit gateway by country: {two_letter_iso_country_code}");

                let filters = base_filters
                    .with(&[GatewayFilter::Country(two_letter_iso_country_code.clone())]);

                self.choose_random(&filters).ok_or_else(|| {
                    Error::NoMatchingExitGatewayForLocation {
                        requested_location: two_letter_iso_country_code.clone(),
                        available_countries: self.all_iso_codes(),
                    }
                })
            }
            ExitPoint::Region { region } => {
                tracing::debug!("Selecting exit gateway by region/state: {region}");

                // Currently only supported in the US
                let filters = base_filters.with(&[
                    GatewayFilter::Country(COUNTRY_WITH_REGION_SELECTOR.to_string()),
                    GatewayFilter::Region(region.to_string()),
                ]);

                self.choose_random(&filters).ok_or_else(|| {
                    Error::NoMatchingExitGatewayForLocation {
                        requested_location: region.clone(),
                        available_countries: self.all_iso_codes(),
                    }
                })
            }
            ExitPoint::Random => {
                tracing::debug!("Selecting a random exit gateway");

                self.choose_random(base_filters)
                    .ok_or_else(|| Error::FailedToSelectGatewayRandomly)
            }
        }
    }

    /// Find the "best" exit gateway that matches `exit_point` using a descending score system.
    /// If no gateway matches the highest score, it will try the next lower score, and so on.
    pub fn find_best_exit_point_gateway(
        &self,
        exit_point: &ExitPoint,
        base_filters: &GatewayFilters,
    ) -> Result<Gateway> {
        for score in [ScoreValue::High, ScoreValue::Medium, ScoreValue::Low] {
            tracing::debug!("Looking for entry gateway with minimum score: {score}");

            let filters = base_filters.with(&[GatewayFilter::MinScore(score)]);
            match self.find_exit_gateway(exit_point, &filters) {
                Ok(gateway) => {
                    return Ok(gateway);
                }
                Err(err) => {
                    if !err.is_unmatched_non_specific_gateway() {
                        return Err(err);
                    }
                    // continue
                }
            }
        }
        match exit_point {
            ExitPoint::Address { address } => Err(Error::NoMatchingGateway {
                requested_identity: address.to_string(),
            }),
            ExitPoint::Gateway { identity } => Err(Error::NoMatchingGateway {
                requested_identity: identity.to_string(),
            }),
            ExitPoint::Country {
                two_letter_iso_country_code,
            } => Err(Error::NoMatchingExitGatewayForLocation {
                requested_location: two_letter_iso_country_code.clone(),
                available_countries: self.all_iso_codes(),
            }),
            ExitPoint::Region { region } => Err(Error::NoMatchingExitGatewayForLocation {
                requested_location: region.clone(),
                available_countries: self.all_iso_codes(),
            }),
            ExitPoint::Random => Err(Error::FailedToSelectGatewayRandomly),
        }
    }

    /// Find the "best" SOCKS5 gateway that matches `exit_point` using a descending score system.
    /// If no gateway matches the highest score, it will try the next lower score, and so on.
    pub fn find_best_socks5_gateway(
        &self,
        exit_point: &ExitPoint,
        base_filters: &GatewayFilters,
    ) -> Result<Gateway> {
        for score in [ScoreValue::High, ScoreValue::Medium, ScoreValue::Low] {
            tracing::debug!("Looking for exit gateway with minimum SOCKS5 score: {score}");

            let filters = base_filters.with(&[GatewayFilter::MinSocks5Score(score)]);
            match self.find_exit_gateway(exit_point, &filters) {
                Ok(gateway) => {
                    return Ok(gateway);
                }
                Err(err) => {
                    if !err.is_unmatched_non_specific_gateway() {
                        return Err(err);
                    }
                    // continue
                }
            }
        }
        match exit_point {
            ExitPoint::Address { address } => Err(Error::NoMatchingGateway {
                requested_identity: address.to_string(),
            }),
            ExitPoint::Gateway { identity } => Err(Error::NoMatchingGateway {
                requested_identity: identity.to_string(),
            }),
            ExitPoint::Country {
                two_letter_iso_country_code,
            } => Err(Error::NoMatchingExitGatewayForLocation {
                requested_location: two_letter_iso_country_code.clone(),
                available_countries: self.all_iso_codes(),
            }),
            ExitPoint::Region { region } => Err(Error::NoMatchingExitGatewayForLocation {
                requested_location: region.clone(),
                available_countries: self.all_iso_codes(),
            }),
            ExitPoint::Random => Err(Error::FailedToSelectGatewayRandomly),
        }
    }

    pub fn build_entry_filters(
        min_score: Option<ScoreValue>,
        blacklisted_gateways: &BlacklistedGateways,
    ) -> Vec<GatewayFilter> {
        let mut filters = Vec::new();
        if let Some(min_score) = min_score {
            filters.push(GatewayFilter::MinScore(min_score));
        }
        if blacklisted_gateways.is_empty().unwrap_or(true) {
            tracing::warn!("Error checking blacklisted gateways is empty");
        } else {
            filters.push(GatewayFilter::NotBlacklisted(blacklisted_gateways.clone()));
        }
        filters
    }

    pub fn build_exit_filters(
        min_score: Option<ScoreValue>,
        residential_exit: bool,
    ) -> Vec<GatewayFilter> {
        let mut filters = Vec::new();
        if let Some(min_score) = min_score {
            filters.push(GatewayFilter::MinScore(min_score));
        }
        if residential_exit {
            filters.push(GatewayFilter::Residential);
            filters.push(GatewayFilter::Exit);
        }
        filters
    }
}

impl IntoIterator for GatewayList {
    type Item = Gateway;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.gateways.into_iter()
    }
}

impl nym_client_core::init::helpers::ConnectableGateway for Gateway {
    #[allow(unconditional_recursion)]
    fn node_id(&self) -> NodeId {
        self.node_id()
    }

    fn identity(&self) -> NodeIdentity {
        self.identity()
    }

    fn clients_address(&self, _prefer_ipv6: bool) -> Option<String> {
        // This is a bit of a sharp edge, but temporary until we can remove Option from host
        // and tls port when we add these to the vpn API endpoints.
        Some(
            self.clients_address_tls()
                .or(self.clients_address_no_tls())
                .unwrap_or("ws://".to_string()),
        )
    }

    fn is_wss(&self) -> bool {
        self.clients_address_tls().is_some()
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, strum::EnumIter)]
pub enum GatewayType {
    MixnetEntry,
    MixnetExit,
    Wg,
}

impl fmt::Display for GatewayType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GatewayType::MixnetEntry => write!(f, "mixnet entry"),
            GatewayType::MixnetExit => write!(f, "mixnet exit"),
            GatewayType::Wg => write!(f, "vpn"),
        }
    }
}

impl From<nym_vpn_api_client::types::GatewayType> for GatewayType {
    fn from(gateway_type: nym_vpn_api_client::types::GatewayType) -> Self {
        match gateway_type {
            nym_vpn_api_client::types::GatewayType::MixnetEntry => GatewayType::MixnetEntry,
            nym_vpn_api_client::types::GatewayType::MixnetExit => GatewayType::MixnetExit,
            nym_vpn_api_client::types::GatewayType::Wg => GatewayType::Wg,
        }
    }
}

impl From<GatewayType> for nym_vpn_api_client::types::GatewayType {
    fn from(gateway_type: GatewayType) -> Self {
        match gateway_type {
            GatewayType::MixnetEntry => nym_vpn_api_client::types::GatewayType::MixnetEntry,
            GatewayType::MixnetExit => nym_vpn_api_client::types::GatewayType::MixnetExit,
            GatewayType::Wg => nym_vpn_api_client::types::GatewayType::Wg,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GatewayFilter {
    MinScore(ScoreValue),                // Mixnet or Wg score
    MinSocks5Score(ScoreValue),          // SOCKS5 score
    Country(String),                     // Two-letter ISO country code
    Region(String),                      // Region name
    Residential,                         // Has a residential ASN
    QuicEnabled,                         // Has QUIC enabled
    Exit,                                // Has an IPR address
    Vpn,                                 // Has an authenticator address
    NotBlacklisted(BlacklistedGateways), // Is not in the blacklist
}

#[derive(Debug, Clone, Default)]
pub struct GatewayFilters(HashSet<GatewayFilter>);

impl GatewayFilters {
    pub fn from<'a>(filters: impl IntoIterator<Item = &'a GatewayFilter>) -> Self {
        GatewayFilters(filters.into_iter().cloned().collect())
    }

    pub fn with<'a>(&self, other: impl IntoIterator<Item = &'a GatewayFilter>) -> Self {
        let mut new_self = self.clone();
        for filter in other.into_iter() {
            new_self.0.insert(filter.clone());
        }
        new_self
    }

    pub fn add(&mut self, filter: GatewayFilter) {
        self.0.insert(filter);
    }

    pub fn remove(&mut self, filter: &GatewayFilter) {
        self.0.remove(filter);
    }

    pub fn contains(&self, filter: &GatewayFilter) -> bool {
        self.0.contains(filter)
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, GatewayFilter> {
        self.0.iter()
    }
}

#[derive(Debug, Clone)]
pub struct LookupGatewayFilters {
    pub gw_type: GatewayType,
    pub filters: GatewayFilters,
}
