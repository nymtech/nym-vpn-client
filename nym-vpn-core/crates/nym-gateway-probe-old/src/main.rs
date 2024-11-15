// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use clap::Parser as _;
use nym_gateway_probe_old::CliArgs;

#[cfg(unix)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    if args.version == 2 {
        println!("Running V2 probe");
        nym_gateway_probe_old::probe::v2::run::run(args).await?;
    } else {
        eprintln!("Invalid version specified");
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(not(unix))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    eprintln!("This tool is only supported on Unix systems");
    std::process::exit(1)
}
