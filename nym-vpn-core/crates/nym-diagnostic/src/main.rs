// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    cli::{CliArgs, Command},
    diagnostic::DiagnosticHandler,
};

use anyhow::{Context, bail};
use nym_vpn_network_config::{Discovery, Fetcher, Network};

use clap::Parser;

mod cli;
mod diagnostic;
mod error;
mod logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    logging::setup_tracing_logger(&args)?;

    let network = if args.network == "mainnet" {
        // Special case for 99% of cases, skip discovery through network
        Network::mainnet_default().with_context(|| "Missing network config")?
    } else if let Some(discovery) = Discovery::default_discovery(&args.network) {
        let fetcher = Fetcher::new(discovery.clone(), None, None)
            .context("Failed to build non mainnet env : fetcher")?;
        let network_details = fetcher
            .fetch_network_details()
            .await
            .context("Failed to build non mainnet env : network details")?;
        crate::Network::new_from_discovery(discovery, *network_details)
            .context("Failed to build non mainnet env : build")?
    } else {
        bail!("Unknown network name");
    };

    match args.command {
        Command::Run(parameters) => {
            let report = DiagnosticHandler::run(network, parameters.into()).await;
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
        Command::Register(parameters) => {
            let report = Box::pin(DiagnosticHandler::register(network, parameters.into())).await;
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
    }
}
