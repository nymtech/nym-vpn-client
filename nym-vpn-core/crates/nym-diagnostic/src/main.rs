// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    cli::{CliArgs, Command},
    diagnostic::DiagnosticHandler,
};

use nym_vpn_network_config::Network;

use clap::Parser;

mod cli;
mod diagnostic;
mod error;
mod logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    logging::setup_tracing_logger(&args)?;
    let network = Network::mainnet_default().ok_or(anyhow::anyhow!("Missing network config"))?;

    match args.command {
        Command::Run(parameters) => {
            let report = DiagnosticHandler::run(network, parameters.into()).await;
            tracing::info!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
        Command::Register(parameters) => {
            let report = Box::pin(DiagnosticHandler::register(network, parameters.into())).await;
            tracing::info!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
    }
}
