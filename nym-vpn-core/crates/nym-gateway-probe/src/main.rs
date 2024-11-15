// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use clap::Parser as _;
use nym_gateway_probe::CliArgs;

#[cfg(unix)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    if args.version == 4 {
        println!("Running V4 probe");
        nym_gateway_probe::probe::v4::run::run(args.clone()).await?;
    }

    if args.version == 3 {
        println!("Running V3 probe");
        nym_gateway_probe::probe::v3::run::run(args).await?;
    }

    Ok(())
}

#[cfg(not(unix))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    eprintln!("This tool is only supported on Unix systems");
    std::process::exit(1)
}
