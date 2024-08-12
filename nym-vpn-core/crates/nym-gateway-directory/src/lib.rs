// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod entries;
mod error;
mod gateway_client;
mod helpers;

pub use crate::{
    entries::{
        auth_addresses::{AuthAddress, AuthAddresses},
        country::Country,
        entry_point::EntryPoint,
        exit_point::ExitPoint,
        gateway::{Entry, Exit, Gateway, GatewayList, Location, Probe, ProbeOutcome},
        ipr_addresses::IpPacketRouterAddress,
    },
    error::Error,
    gateway_client::{Config, GatewayClient},
};

pub use nym_sdk::mixnet::{NodeIdentity, Recipient};
pub use nym_validator_client::models::DescribedGateway;

const FORCE_TLS_FOR_GATEWAY_SELECTION: bool = false;
