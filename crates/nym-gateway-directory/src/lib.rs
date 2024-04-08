// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod entries;
mod error;
mod gateway_client;
mod helpers;

pub use crate::{
    entries::{
        described_gateway::{DescribedGatewayWithLocation, LookupGateway},
        entry_point::EntryPoint,
        exit_point::ExitPoint,
        ipr_address::IpPacketRouterAddress,
    },
    error::Error,
    gateway_client::{Config, GatewayClient},
};

pub use nym_sdk::mixnet::{NodeIdentity, Recipient};

const FORCE_TLS_FOR_GATEWAY_SELECTION: bool = false;
