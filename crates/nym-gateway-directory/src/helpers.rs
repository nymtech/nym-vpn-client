// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    described_gateway::DescribedGatewayWithLocation, error::Result, Error,
    FORCE_TLS_FOR_GATEWAY_SELECTION,
};
use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};
use itertools::Itertools;
use nym_client_core::init::helpers::choose_gateway_by_latency;
use nym_sdk::mixnet::NodeIdentity;
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

pub(crate) fn select_random_gateway_node<'a, I>(gateways: I) -> Result<NodeIdentity>
where
    I: IntoIterator<Item = &'a DescribedGatewayWithLocation>,
{
    let random_gateway = select_random_described_gateway(gateways)?;
    NodeIdentity::from_base58_string(random_gateway.identity_key())
        .map_err(|_| Error::NodeIdentityFormattingError)
}

pub(crate) async fn select_random_low_latency_gateway_node(
    gateways: &[DescribedGatewayWithLocation],
) -> Result<NodeIdentity> {
    let mut rng = rand::rngs::OsRng;
    let must_use_tls = FORCE_TLS_FOR_GATEWAY_SELECTION;
    let gateway_nodes: Vec<nym_topology::gateway::Node> = gateways
        .iter()
        .filter_map(|gateway| nym_topology::gateway::Node::try_from(&gateway.gateway).ok())
        .collect();
    choose_gateway_by_latency(&mut rng, &gateway_nodes, must_use_tls)
        .await
        .map(|gateway| *gateway.identity())
        .map_err(|err| Error::FailedToSelectGatewayBasedOnLowLatency { source: err })
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
    gateways: &[DescribedGatewayWithLocation],
) -> Result<&DescribedGatewayWithLocation> {
    let low_latency_gateway = select_random_low_latency_gateway_node(gateways).await?;
    gateways
        .iter()
        .find(|gateway| gateway.identity_key() == &low_latency_gateway.to_string())
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
