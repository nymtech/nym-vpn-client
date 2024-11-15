// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::anyhow;

use nym_bin_common_v2::bin_info_local_vergen;
use nym_config_v2::defaults::setup_env;
use nym_gateway_directory_v2::EntryPoint;
use tracing::*;

use crate::{
    probe::v2::{fetch_gateways_with_ipr, probe},
    setup_logging, CliArgs,
};

use super::types::ProbeResult;

pub async fn run(args: CliArgs) -> anyhow::Result<ProbeResult> {
    if !args.no_log {
        setup_logging();
    }
    debug!("{:?}", bin_info_local_vergen!());
    setup_env(args.config_env_file.as_ref());

    let gateway = if let Some(gateway) = args.gateway {
        EntryPoint::from_base58_string(&gateway)?
    } else {
        fetch_random_gateway_with_ipr().await?
    };

    probe(gateway).await
}

async fn fetch_random_gateway_with_ipr() -> anyhow::Result<EntryPoint> {
    // We're fetching gateways with IPR, since they are more interesting to ping, but we can probe
    // gateways without an IPR as well
    tracing::info!("Selecting random gateway with IPR enabled");
    let gateways = fetch_gateways_with_ipr().await?;
    let gateway = gateways
        .random_gateway()
        .ok_or(anyhow!("No gateways returned by nym-api"))?;
    Ok(EntryPoint::Gateway {
        identity: *gateway.identity(),
    })
}
