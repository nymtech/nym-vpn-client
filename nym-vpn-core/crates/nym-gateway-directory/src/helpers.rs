// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{error::Result, DescribedGatewayWithLocation, Error, FORCE_TLS_FOR_GATEWAY_SELECTION};
use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};
use itertools::Itertools;
use nym_client_core::init::helpers::choose_gateway_by_latency;
use nym_harbour_master_client::Gateway as HmGateway;
use nym_sdk::mixnet::NodeIdentity;
use nym_topology::IntoGatewayNode;
use nym_validator_client::{client::GatewayBond, models::DescribedGateway};
use rand::seq::IteratorRandom;
use std::net::IpAddr;
use tracing::debug;

pub(crate) fn select_random_described_gateway<'a, I>(
    gateways: I,
) -> Result<&'a DescribedGatewayWithLocation>
where
    I: IntoIterator<Item = &'a DescribedGatewayWithLocation>,
{
    gateways
        .into_iter()
        .choose(&mut rand::thread_rng())
        .ok_or(Error::FailedToSelectGatewayRandomly)
}

// pub(crate) fn select_random_gateway_node<'a, I>(
//     gateways: I,
// ) -> Result<(NodeIdentity, Option<String>)>
// where
//     I: IntoIterator<Item = &'a DescribedGatewayWithLocation>,
// {
//     let random_gateway = select_random_described_gateway(gateways)?;
//     let id = NodeIdentity::from_base58_string(random_gateway.identity_key())
//         .map_err(|_| Error::NodeIdentityFormattingError)?;
//     Ok((id, random_gateway.two_letter_iso_country_code()))
// }

// pub(crate) async fn select_random_low_latency_gateway_node(
//     gateways: &[DescribedGatewayWithLocation],
// ) -> Result<(NodeIdentity, Option<String>)> {
//     let mut rng = rand::rngs::OsRng;
//     let must_use_tls = FORCE_TLS_FOR_GATEWAY_SELECTION;
//     let gateway_nodes: Vec<nym_topology::gateway::Node> = gateways
//         .iter()
//         .filter_map(|gateway| nym_topology::gateway::Node::try_from(&gateway.gateway).ok())
//         .collect();
//     let gateway = choose_gateway_by_latency(&mut rng, &gateway_nodes, must_use_tls)
//         .await
//         .map(|gateway| *gateway.identity())
//         .map_err(|err| Error::FailedToSelectGatewayBasedOnLowLatency { source: err })?;
//     let country = gateways
//         .iter()
//         .find(|g| g.identity_key() == &gateway.to_string())
//         .and_then(|gateway| gateway.two_letter_iso_country_code());
//     Ok((gateway, country))
// }

pub(crate) async fn select_random_low_latency_gateway_node(
    gateways: &[GatewayBond],
) -> Result<NodeIdentity> {
    let mut rng = rand::rngs::OsRng;
    let must_use_tls = FORCE_TLS_FOR_GATEWAY_SELECTION;
    let gateway_nodes: Vec<nym_topology::gateway::Node> = gateways
        .iter()
        .filter_map(|gateway| nym_topology::gateway::Node::try_from(gateway).ok())
        .collect();
    let gateway = choose_gateway_by_latency(&mut rng, &gateway_nodes, must_use_tls)
        .await
        .map(|gateway| *gateway.identity())
        .map_err(|err| Error::FailedToSelectGatewayBasedOnLowLatency { source: err })?;
    Ok(gateway)
}

pub(crate) fn list_all_country_iso_codes<'a, I>(gateways: I) -> Vec<String>
where
    I: IntoIterator<Item = &'a DescribedGatewayWithLocation>,
{
    gateways
        .into_iter()
        .filter_map(|gateway| gateway.two_letter_iso_country_code())
        .unique()
        .collect()
}

pub(crate) async fn select_random_low_latency_described_gateway(
    gateways: &[DescribedGateway],
) -> Result<&DescribedGateway> {
    let gateway_nodes = gateways
        .iter()
        .map(|gateway| gateway.bond.clone())
        .collect::<Vec<_>>();
    let low_latency_gateway = select_random_low_latency_gateway_node(&gateway_nodes).await?;
    gateways
        .iter()
        .find(|gateway| gateway.identity() == low_latency_gateway.to_string())
        .ok_or(Error::NoMatchingGateway)
}

pub(crate) async fn try_resolve_hostname(hostname: &str) -> Result<IpAddr> {
    debug!("Trying to resolve hostname: {hostname}");
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    let addrs = resolver.lookup_ip(hostname).await.map_err(|err| {
        tracing::error!("Failed to resolve gateway hostname: {}", err);
        Error::FailedToDnsResolveGateway {
            hostname: hostname.to_string(),
            source: err,
        }
    })?;
    debug!("Resolved to: {addrs:?}");

    // Just pick the first one
    addrs
        .iter()
        .next()
        .ok_or(Error::ResolvedHostnameButNoIp(hostname.to_string()))
}

pub(crate) fn filter_on_exit_gateways(
    gateways: Vec<DescribedGatewayWithLocation>,
) -> Vec<DescribedGatewayWithLocation> {
    gateways
        .into_iter()
        .filter(|gateway| gateway.has_ip_packet_router() && gateway.is_current_build())
        .collect()
}

pub(crate) fn filter_on_harbour_master_entry_data(
    gateways: Vec<DescribedGatewayWithLocation>,
    hm_gateways: Vec<HmGateway>,
) -> Vec<DescribedGatewayWithLocation> {
    let hm_gateway_ids: Vec<_> = hm_gateways
        .into_iter()
        .filter(|gateway| gateway.is_fully_operational_entry())
        .map(|gateway| gateway.gateway_identity_key)
        .collect();
    let no_gateways_before_filtering = gateways.len();
    let filtered_gateways: Vec<_> = gateways
        .into_iter()
        .filter(|gateway| hm_gateway_ids.contains(gateway.identity_key()))
        .collect();
    let no_gateways_after_filtering = filtered_gateways.len();
    log::info!(
        "Filtering out {} out of {} entry gateways as not fully operational",
        no_gateways_before_filtering - no_gateways_after_filtering,
        no_gateways_before_filtering,
    );
    filtered_gateways
}

pub(crate) fn filter_on_harbour_master_exit_data(
    gateways: Vec<DescribedGatewayWithLocation>,
    hm_gateways: Vec<HmGateway>,
) -> Vec<DescribedGatewayWithLocation> {
    let hm_gateway_ids: Vec<_> = hm_gateways
        .into_iter()
        .filter(|gateway| gateway.is_fully_operational_exit())
        .map(|gateway| gateway.gateway_identity_key)
        .collect();
    let no_gateways_before_filtering = gateways.len();
    let filtered_gateways: Vec<_> = gateways
        .into_iter()
        .filter(|gateway| hm_gateway_ids.contains(gateway.identity_key()))
        .collect();
    let no_gateways_after_filtering = filtered_gateways.len();
    log::info!(
        "Filtering out {} out of {} exit gateways as not fully operational",
        no_gateways_before_filtering - no_gateways_after_filtering,
        no_gateways_before_filtering,
    );
    filtered_gateways
}
