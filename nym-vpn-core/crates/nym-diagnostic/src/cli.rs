// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use clap::Parser;
use std::{path::PathBuf, sync::OnceLock};

// Helper for passing LONG_VERSION to clap
fn pretty_build_info_static() -> &'static str {
    static PRETTY_BUILD_INFORMATION: OnceLock<String> = OnceLock::new();
    PRETTY_BUILD_INFORMATION.get_or_init(|| nym_bin_common::bin_info_local_vergen!().pretty_print())
}

#[derive(Clone, Debug, Parser)]
#[clap(author = "Nymtech", version, long_version = pretty_build_info_static(), about)]
pub(crate) struct CliArgs {
    /// Logging verbosity.
    #[arg(long, short = 'v', global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Disable logging altogether
    #[arg(long, global = true, action = clap::ArgAction::SetTrue)]
    pub no_log: bool,

    /// Env to run the diagnostic in
    #[arg(short, long, global = true, value_parser = ["mainnet", "sandbox", "canary","evil"], default_value = "mainnet")]
    pub network: String,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Command,
}

impl CliArgs {
    #[allow(dead_code)] // false positive, it's used in the binary
    pub fn verbosity_level(&self) -> tracing::Level {
        match self.verbose {
            0 => tracing::Level::INFO,
            1 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        }
    }
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Run diagnostic
    Run(RunParams),

    /// Register to a gateway for diagnostic. SUCCESSFUL RUNS ARE SPENDING AN ENTRY TICKET
    Register(RegisterParams),
}

#[derive(Debug, Clone, clap::Args)]
pub struct RunParams {
    /// Id of the gateway we are going to connect to.
    #[arg(long)]
    pub gateway: Option<String>,

    /// Skip DNS diagnostic
    #[clap(long, action = clap::ArgAction::SetTrue)]
    pub skip_dns: bool,

    /// Skip HTTP diagnostic
    #[clap(long, action = clap::ArgAction::SetTrue)]
    pub skip_http: bool,

    /// Skip CTAP 2.2 Hybrid Transport reachability diagnostic
    #[clap(long, action = clap::ArgAction::SetTrue)]
    pub skip_hybrid_transport: bool,
}

impl From<RunParams> for nym_vpn_lib_types::DiagnosticRunParams {
    fn from(value: RunParams) -> Self {
        Self {
            gateway: value.gateway,
            skip_dns: value.skip_dns,
            skip_http: value.skip_http,
            skip_hybrid_transport: value.skip_hybrid_transport,
        }
    }
}

#[derive(Debug, Clone, clap::Args)]
#[command(group(
    clap::ArgGroup::new("mode")
        .args(["mixnet", "lp"])
        .multiple(false) // mutually exclusive
))]
pub struct RegisterParams {
    /// Id of the gateway we are going to connect to.
    #[arg(long)]
    pub gateway: String,

    /// Path to the storage dir
    #[arg(long)]
    pub storage_path: Option<PathBuf>,

    /// Skip Wireguard diagnostic
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub skip_wireguard: bool,

    /// Registration via mixnet
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub mixnet: bool,

    /// Registration via LP
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub lp: bool,
}

impl From<RegisterParams> for nym_vpn_lib_types::DiagnosticRegisterParams {
    fn from(value: RegisterParams) -> Self {
        // Mixnet is the default
        let registration_mode =
            nym_vpn_lib_types::RegistrationMode::from_cli_flags(value.mixnet, value.lp);
        Self {
            gateway: value.gateway,
            storage_path: value.storage_path,
            skip_wireguard: value.skip_wireguard,
            registration_mode,
        }
    }
}
