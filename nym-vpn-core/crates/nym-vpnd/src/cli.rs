// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, sync::OnceLock};

use clap::{Parser, Subcommand};

// Helper for passing LONG_VERSION to clap
fn pretty_build_info_static() -> &'static str {
    static PRETTY_BUILD_INFORMATION: OnceLock<String> = OnceLock::new();
    PRETTY_BUILD_INFORMATION.get_or_init(|| nym_bin_common::bin_info_local_vergen!().pretty_print())
}

#[derive(Parser, Debug)]
#[clap(author = "Nymtech", version, about, long_version = pretty_build_info_static())]
pub struct CliArgs {
    /// Logging verbosity.
    #[arg(long, short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Path pointing to an env file describing the network.
    #[arg(short, long, value_parser = check_path)]
    pub config_env_file: Option<PathBuf>,

    #[arg(short, long, hide = true)]
    pub network: Option<String>,

    /// Override the default user agent string.
    #[arg(long, value_parser = parse_user_agent)]
    pub user_agent: Option<nym_vpn_lib::UserAgent>,

    /// Override the otherwise random statistics_id_seed
    #[arg(long, hide = true)]
    pub stats_id_seed: Option<String>,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Command>,
}

impl CliArgs {
    pub fn verbosity_level(&self) -> tracing::Level {
        match self.verbose {
            0 => tracing::Level::INFO,
            1 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        }
    }

    pub fn is_run_as_service(&self) -> bool {
        matches!(self.command, Some(Command::RunAsService))
    }
}

#[derive(Debug, Copy, Clone, Default, Subcommand)]
pub enum Command {
    #[cfg(windows)]
    /// Install windows service
    InstallService,

    #[cfg(windows)]
    /// Uninstall windows service
    UninstallService,

    #[cfg(windows)]
    /// Start windows service
    StartService,

    /// Run daemon as a system service
    RunAsService,

    /// Run daemon standalone
    #[default]
    #[clap(skip)]
    RunStandalone,
}

fn check_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!("Path {} does not exist", path.display()));
    }
    if !path.is_file() {
        return Err(format!("Path {} is not a file", path.display()));
    }
    Ok(path)
}

fn parse_user_agent(user_agent: &str) -> Result<nym_vpn_lib::UserAgent, String> {
    nym_vpn_lib::UserAgent::try_from(user_agent).map_err(|e| e.to_string())
}
