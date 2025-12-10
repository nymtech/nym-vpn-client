// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use itertools::Itertools;
use nym_sdk::mixnet::NodeIdentity;
use nym_topology::{NodeId, RoutingNode};
use nym_validator_client::models::{KeyRotationId, NymNodeDescription};
use nym_vpn_api_client::{
    response::{BridgeInformation, BridgeParameters},
    types::Percent,
};
use rand::seq::IteratorRandom;
use std::{
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
}

impl Gateway {
    pub fn try_from_node_description(
        node_description: NymNodeDescription,
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

    /// Tests whether the gateway matches all of the filters.
    pub fn matches_all_filters(
        &self,
        gw_type: Option<GatewayType>,
        filters: &[GatewayFilter],
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl From<nym_vpn_api_client::response::Asn> for Asn {
    fn from(location: nym_vpn_api_client::response::Asn) -> Self {
        Asn {
            asn: location.asn,
            name: location.name,
            kind: location.kind.into(),
        }
    }
}

impl From<nym_vpn_api_client::response::Location> for Location {
    fn from(location: nym_vpn_api_client::response::Location) -> Self {
        Location {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
            city: location.city,
            region: location.region,
            asn: location.asn.map(Into::into),
        }
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
            can_query_metadata_v4: results.can_query_metadata_v4,
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

        Ok(Gateway {
            identity,
            name: gateway.name,
            description: gateway.description,
            location: Some(gateway.location.into()),
            ipr_address,
            authenticator_address,
            nr_address: None,
            bridge_params: gateway.bridges,
            last_probe: gateway.last_probe.map(Probe::from),
            ips: gateway.ip_addresses,
            host,
            clients_ws_port: Some(gateway.entry.ws_port),
            clients_wss_port: gateway.entry.wss_port,
            mixnet_performance: Some(gateway.performance),
            performance: gateway.performance_v2.map(Performance::from),
            version: gateway.build_information.map(|info| info.build_version),
        })
    }
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

    pub fn filter(&self, filters: &[GatewayFilter]) -> Vec<Gateway> {
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

    pub fn choose_random(&self, filters: &[GatewayFilter]) -> Option<Gateway> {
        self.filter(filters)
            .into_iter()
            .choose(&mut rand::thread_rng())
    }

    pub fn remove_gateway(&mut self, entry_gateway: &Gateway) {
        self.gateways
            .retain(|gateway| gateway.identity() != entry_gateway.identity());
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
        Self::new(self.gw_type, self.filter(&[GatewayFilter::Exit]))
    }

    pub fn into_vpn_gateways(self) -> GatewayList {
        Self::new(self.gw_type, self.filter(&[GatewayFilter::Vpn]))
    }

    pub fn into_inner(self) -> Vec<Gateway> {
        self.gateways
    }

    pub fn find_entry_gateway(
        &self,
        entry_point: &EntryPoint,
        base_filters: &[GatewayFilter],
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
                    .iter()
                    .chain(&vec![GatewayFilter::Country(
                        two_letter_iso_country_code.clone(),
                    )])
                    .cloned()
                    .collect::<Vec<_>>();

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
                let filters = base_filters
                    .iter()
                    .chain(&vec![
                        GatewayFilter::Country(COUNTRY_WITH_REGION_SELECTOR.to_string()),
                        GatewayFilter::Region(region.to_string()),
                    ])
                    .cloned()
                    .collect::<Vec<_>>();

                self.choose_random(&filters).ok_or_else(|| {
                    Error::NoMatchingEntryGatewayForLocation {
                        requested_location: region.clone(),
                        available_countries: self.all_iso_codes(),
                    }
                })
            }
            EntryPoint::Random => {
                tracing::debug!("Selecting a random entry gateway");

                self.choose_random(base_filters)
                    .ok_or_else(|| Error::FailedToSelectGatewayRandomly)
            }
        }
    }

    pub fn find_best_entry_gateway(
        &self,
        entry_point: &EntryPoint,
        base_filters: &[GatewayFilter],
    ) -> Result<Gateway> {
        for score in [ScoreValue::High, ScoreValue::Medium, ScoreValue::Low] {
            tracing::debug!("Looking for entry gateway with minimum score: {score}");

            let mut filters = base_filters.to_vec();
            filters.push(GatewayFilter::MinScore(score));

            match self.find_entry_gateway(entry_point, &filters) {
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

    pub fn find_exit_gateway(
        &self,
        exit_point: &ExitPoint,
        base_filters: &[GatewayFilter],
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
                    .iter()
                    .chain(&vec![GatewayFilter::Country(
                        two_letter_iso_country_code.clone(),
                    )])
                    .cloned()
                    .collect::<Vec<_>>();

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
                let filters = base_filters
                    .iter()
                    .chain(&vec![
                        GatewayFilter::Country(COUNTRY_WITH_REGION_SELECTOR.to_string()),
                        GatewayFilter::Region(region.to_string()),
                    ])
                    .cloned()
                    .collect::<Vec<_>>();

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

    pub fn find_best_exit_gateway(
        &self,
        exit_point: &ExitPoint,
        base_filters: &[GatewayFilter],
    ) -> Result<Gateway> {
        for score in [ScoreValue::High, ScoreValue::Medium, ScoreValue::Low] {
            tracing::debug!("Looking for entry gateway with minimum score: {score}");

            let mut filters = base_filters.to_vec();
            filters.push(GatewayFilter::MinScore(score));
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
            } => Err(Error::NoMatchingEntryGatewayForLocation {
                requested_location: two_letter_iso_country_code.clone(),
                available_countries: self.all_iso_codes(),
            }),
            ExitPoint::Region { region } => Err(Error::NoMatchingEntryGatewayForLocation {
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

#[derive(Debug, Clone, PartialEq)]
pub enum GatewayFilter {
    MinScore(ScoreValue),                // Mixnet or Wg score
    Country(String),                     // Two-letter ISO country code
    Region(String),                      // Region name
    Residential,                         // Has a residential ASN
    QuicEnabled,                         // Has QUIC enabled
    Exit,                                // Has an IPR address
    Vpn,                                 // Has an authenticator address
    NotBlacklisted(BlacklistedGateways), // Is not in the blacklist
}

#[derive(Debug, Clone, PartialEq)]
pub struct GatewayFilters {
    pub gw_type: GatewayType,
    pub filters: Vec<GatewayFilter>,
}

#[cfg(test)]
mod tests {
    use nym_vpn_api_client::response::QuicClientOptions;

    use super::*;

    #[test]
    fn test_matching_mixnet_score() {
        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .performance(Performance {
                last_updated_utc: "".to_owned(),
                score: ScoreValue::Offline,
                mixnet_score: ScoreValue::High,
                load: ScoreValue::Medium,
                uptime_percentage_last_24_hours: 1f32,
            })
            .build();

        for gw_type in [GatewayType::MixnetEntry, GatewayType::MixnetExit] {
            assert!(
                gateway.matches_filter(Some(gw_type), &GatewayFilter::MinScore(ScoreValue::Low))
            );
        }

        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .performance(Performance {
                last_updated_utc: "".to_owned(),
                score: ScoreValue::Offline,
                mixnet_score: ScoreValue::Low,
                load: ScoreValue::Medium,
                uptime_percentage_last_24_hours: 1f32,
            })
            .build();

        for gw_type in [GatewayType::MixnetEntry, GatewayType::MixnetExit] {
            assert!(
                !gateway.matches_filter(Some(gw_type), &GatewayFilter::MinScore(ScoreValue::High))
            );
        }
    }

    #[test]
    fn test_matching_wg_score() {
        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .performance(Performance {
                last_updated_utc: "".to_owned(),
                score: ScoreValue::High,
                mixnet_score: ScoreValue::Offline,
                load: ScoreValue::Medium,
                uptime_percentage_last_24_hours: 1f32,
            })
            .build();

        assert!(gateway.matches_filter(
            Some(GatewayType::Wg),
            &GatewayFilter::MinScore(ScoreValue::Low)
        ));

        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .performance(Performance {
                last_updated_utc: "".to_owned(),
                score: ScoreValue::Low,
                mixnet_score: ScoreValue::Offline,
                load: ScoreValue::Medium,
                uptime_percentage_last_24_hours: 1f32,
            })
            .build();

        assert!(!gateway.matches_filter(
            Some(GatewayType::Wg),
            &GatewayFilter::MinScore(ScoreValue::High)
        ));
    }

    #[test]
    fn test_matching_exit_node() {
        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .ipr_address(IpPacketRouterAddress::try_from_base58_string(
               "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
            ).unwrap())
            .build();

        assert!(gateway.matches_filter(None, &GatewayFilter::Exit));

        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .build();
        assert!(!gateway.matches_filter(None, &GatewayFilter::Exit));
    }

    #[test]
    fn test_matching_residential() {
        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .location(Location {
                asn: Some(Asn {
                    kind: AsnKind::Residential,
                    asn: "".to_owned(),
                    name: "".to_owned(),
                }),
                ..Default::default()
            })
            .build();

        assert!(gateway.matches_filter(None, &GatewayFilter::Residential));

        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .location(Location {
                asn: Some(Asn {
                    kind: AsnKind::Other,
                    asn: "".to_owned(),
                    name: "".to_owned(),
                }),
                ..Default::default()
            })
            .build();

        assert!(!gateway.matches_filter(None, &GatewayFilter::Residential));
    }

    #[test]
    fn test_matching_quic_enabled() {
        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .bridge_params(Some(BridgeInformation {
                version: String::from("1"),
                transports: vec![BridgeParameters::QuicPlain(QuicClientOptions {
                    addresses: vec!["1.2.3.4:5".parse().unwrap()],
                    host: Some(String::from("test.host")),
                    id_pubkey: String::from("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"),
                })],
            }))
            .build();

        assert!(gateway.matches_filter(None, &GatewayFilter::QuicEnabled));

        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .bridge_params(Some(BridgeInformation {
                version: String::from("1"),
                transports: vec![],
            }))
            .build();

        assert!(!gateway.matches_filter(None, &GatewayFilter::QuicEnabled));
    }

    #[test]
    fn test_matching_vpn_nodes() {
        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .authenticator_address(
                AuthAddress::try_from_base58_string(
                    "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
                ).unwrap()
            )
            .build();

        assert!(gateway.matches_filter(None, &GatewayFilter::Vpn));

        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .build();

        assert!(!gateway.matches_filter(None, &GatewayFilter::Vpn));
    }

    #[test]
    fn test_matching_country() {
        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .location(Location {
                two_letter_iso_country_code: "CA".to_owned(),
                ..Default::default()
            })
            .build();

        assert!(gateway.matches_filter(None, &GatewayFilter::Country("CA".to_owned())));
        assert!(!gateway.matches_filter(None, &GatewayFilter::Country("US".to_owned())));
    }

    #[test]
    fn test_matching_region() {
        let gateway = Gateway::builder()
            .identity(
                NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                    .unwrap(),
            )
            .location(Location {
                two_letter_iso_country_code: "US".to_owned(),
                region: "CA".to_owned(),
                ..Default::default()
            })
            .build();

        assert!(gateway.matches_filter(None, &GatewayFilter::Region("CA".to_owned())));
        assert!(!gateway.matches_filter(None, &GatewayFilter::Region("FL".to_owned())));
    }

    #[test]
    fn test_gateway_random_country() {
        let gateway_list = sample_gateway_list(GatewayType::MixnetEntry);

        assert!(
            gateway_list
                .choose_random(&[GatewayFilter::Country("US".into())])
                .unwrap()
                .is_in_country("US")
        );

        assert!(
            gateway_list
                .choose_random(&[GatewayFilter::Country("DE".into())])
                .unwrap()
                .is_in_country("DE")
        );

        assert!(
            gateway_list
                .choose_random(&[GatewayFilter::Country("BE".into())])
                .is_none()
        );
    }

    #[test]
    fn test_gateway_random_region() {
        let gateway_list = sample_gateway_list(GatewayType::MixnetExit);

        assert!(
            gateway_list
                .choose_random(&[
                    GatewayFilter::Country("US".into()),
                    GatewayFilter::Region("CA".into())
                ])
                .unwrap()
                .is_in_region("CA")
        );

        assert!(
            gateway_list
                .choose_random(&[
                    GatewayFilter::Country("GB".into()),
                    GatewayFilter::Region("Hampshire".into())
                ])
                .unwrap()
                .is_in_region("Hampshire")
        );

        assert!(
            gateway_list
                .choose_random(&[
                    GatewayFilter::Country("DE".into()),
                    GatewayFilter::Region("XZ".into())
                ])
                .is_none()
        );
    }

    #[test]
    fn test_gateway_non_blacklisted() {
        let gateway_list = sample_gateway_list(GatewayType::MixnetExit);

        let blacklisted = gateway_list.gateways[3].identity;
        let blacklisted_gateways = BlacklistedGateways::new();
        blacklisted_gateways.add(blacklisted).unwrap();

        for _ in 0..64 {
            let chosen = gateway_list
                .choose_random(&[GatewayFilter::NotBlacklisted(blacklisted_gateways.clone())])
                .unwrap();
            assert_ne!(chosen.identity, blacklisted);
        }
    }

    // Create a list of Gateways with different properties set for testing
    fn sample_gateway_list(gw_type: GatewayType) -> GatewayList {
        let asn = Asn {
            asn: "AS12345".to_string(),
            name: "Test ASN".to_string(),
            kind: AsnKind::Residential,
        };
        let addr = "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e";
        let ipr = IpPacketRouterAddress::try_from_base58_string(addr).unwrap();
        let aa = AuthAddress::try_from_base58_string(addr).unwrap();
        let variables = [
            (
                // Gateway 1
                "HiVGQq2riqPFoPyYRYCZq3zFmFk15gnJzH4s9mHEbgKH",
                "US",
                "CA",
                None,
                Some(ipr),
                Some(aa),
            ),
            (
                // Gateway 2
                "B4r2xMJYc4VgoEhPmccmNSawQWdYP9zGp9DJqjcz6PoX",
                "US",
                "NY",
                Some(asn.clone()),
                None,
                None,
            ),
            (
                // Gateway 3
                "6tGNU195QKNMaTxkvm917d3NNGLkpTp8mTfxqLzATbtB",
                "DE",
                "BE",
                None,
                None,
                Some(aa),
            ),
            (
                // Gateway 4
                "F618gw6jZaLR1VdMTeaH11MhHQJY5rdpYEDLrMKEHcjk",
                "FR",
                "Aquitaine",
                Some(asn.clone()),
                None,
                Some(aa),
            ),
            (
                // Gateway 5
                "3UBiq22tkNSRhyRNjL5mnw5Yk4z6FvgvjizT4ukeEaeB",
                "US",
                "TX",
                Some(asn.clone()),
                Some(ipr),
                None,
            ),
            (
                // Gateway 6
                "2djmrzZ62M8jpzpYb7MMq6QjP15CkbnKHf3ZV3kSCXUE",
                "GB",
                "Hampshire",
                None,
                None,
                None,
            ),
        ];

        let mut instance = 0;
        let gateways: Vec<Gateway> = variables
            .into_iter()
            .map(|(identity, country, region, asn, ipr, aa)| {
                instance += 1;
                Gateway {
                    identity: NodeIdentity::from_base58_string(identity).unwrap(),
                    name: format!("Gateway {instance}"),
                    description: None,
                    location: Some(Location {
                        two_letter_iso_country_code: country.to_string(),
                        region: region.to_string(),
                        asn,
                        ..Default::default()
                    }),
                    ipr_address: ipr,
                    authenticator_address: aa,
                    nr_address: None,
                    bridge_params: None,
                    last_probe: None,
                    ips: Vec::new(),
                    host: None,
                    clients_ws_port: None,
                    clients_wss_port: None,
                    mixnet_performance: Some(Percent::from_percentage_value(75).unwrap()),
                    performance: Some(Performance {
                        last_updated_utc: "2024-01-01T00:00:00Z".to_string(),
                        score: ScoreValue::High,
                        mixnet_score: ScoreValue::High,
                        load: ScoreValue::Low,
                        uptime_percentage_last_24_hours: 0.75,
                    }),
                    version: None,
                }
            })
            .collect();
        GatewayList::new(Some(gw_type), gateways)
    }

    fn create_test_gateway(identity: &str, country: &str, score: ScoreValue) -> Gateway {
        Gateway {
            identity: NodeIdentity::from_base58_string(identity).unwrap(),
            name: format!("Test Gateway {}", country),
            description: None,
            location: Some(Location {
                two_letter_iso_country_code: country.to_string(),
                ..Default::default()
            }),
            ipr_address: None,
            authenticator_address: None,
            nr_address: None,
            bridge_params: None,
            last_probe: None,
            ips: Vec::new(),
            host: None,
            clients_ws_port: None,
            clients_wss_port: None,
            mixnet_performance: None,
            performance: Some(Performance {
                last_updated_utc: "2025-10-22T00:00:00Z".to_string(),
                score,
                mixnet_score: ScoreValue::High,
                load: ScoreValue::Low,
                uptime_percentage_last_24_hours: 0.99,
            }),
            version: None,
        }
    }

    #[test]
    fn test_low_performance_fallback_for_country_selection() {
        // Previously High -> Medium before failing
        // Now tries High -> Medium -> Low which allows connection to more gateways
        let entry_point = EntryPoint::Country {
            two_letter_iso_country_code: "VN".to_string(),
        };

        let gateways = GatewayList::new(
            Some(GatewayType::Wg),
            vec![create_test_gateway(
                "DoezvC92kAVDhFpBbsRj52rErhikj2vtPi1Lup2EhbZ4",
                "VN",
                ScoreValue::Low,
            )],
        );

        // Without Low fallback, this would fail
        let blacklisted_gateways = BlacklistedGateways::new();
        let base_filters =
            GatewayList::build_entry_filters(Some(ScoreValue::Low), &blacklisted_gateways);
        let result = gateways.find_entry_gateway(&entry_point, &base_filters);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().performance.as_ref().unwrap().score,
            ScoreValue::Low
        );
    }
}
