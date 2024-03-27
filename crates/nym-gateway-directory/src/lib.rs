// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod described_gateway;
mod entry_point;
mod error;
mod exit_point;
mod gateway_client;
mod helpers;
mod ipr_address;

pub use crate::{
    entry_point::EntryPoint,
    error::Error,
    exit_point::ExitPoint,
    gateway_client::{Config, GatewayClient},
    ipr_address::IpPacketRouterAddress,
};

pub use nym_sdk::mixnet::{NodeIdentity, Recipient};

const FORCE_TLS_FOR_GATEWAY_SELECTION: bool = false;
