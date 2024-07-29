// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{error::Result, Error, FORCE_TLS_FOR_GATEWAY_SELECTION};
use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};
use nym_client_core::init::helpers::choose_gateway_by_latency;
use nym_sdk::mixnet::NodeIdentity;
use nym_topology::IntoGatewayNode;
use nym_validator_client::{client::GatewayBond, models::DescribedGateway};
use std::net::IpAddr;
use tracing::debug;

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
