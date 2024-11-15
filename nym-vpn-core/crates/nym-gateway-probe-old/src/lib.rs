// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use clap::Parser;

mod error;
mod netstack;
pub mod probe;

pub(crate) fn setup_logging() {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("hyper::proto=info".parse().unwrap())
        .add_directive("netlink_proto=info".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();
}

#[derive(Parser, Clone)]
#[clap(author, version, about)]
pub struct CliArgs {
    /// Path pointing to an env file describing the network.
    #[arg(short, long)]
    config_env_file: Option<PathBuf>,

    #[arg(long, short)]
    gateway: Option<String>,

    #[arg(long, short)]
    no_log: bool,

    #[arg(long, short)]
    pub version: u8
}
