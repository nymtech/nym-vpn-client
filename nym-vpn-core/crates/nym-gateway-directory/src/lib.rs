// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod blacklisted_gateways;
mod builtin;
mod entries;
mod error;
pub mod gateway_cache;
mod gateway_client;
mod gateway_store;
mod helpers;

pub use nym_sdk::mixnet::{NodeIdentity, Recipient};
pub use nym_vpn_api_client::types::{GatewayMinPerformance, NaiveFloat, Percent};

pub use crate::{
    blacklisted_gateways::BlacklistedGateways,
    entries::{
        auth_addresses::AuthAddress,
        country::Country,
        entry_point::EntryPoint,
        exit_point::ExitPoint,
        gateway::{
            Asn, AsnKind, Entry, Exit, Gateway, GatewayFilter, GatewayFilters, GatewayList,
            GatewayType, Location, LookupGatewayFilters, Lp, NymNode, NymNodeList, Performance,
            Probe, ProbeOutcome, ScoreValue, Socks5,
        },
        ipr_addresses::IpPacketRouterAddress,
    },
    error::Error,
    gateway_cache::{GatewayCache, GatewayCacheHandle},
    gateway_client::{Config, GatewayClient, ResolvedConfig},
    helpers::{resolve_config, split_ips},
};
