// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{str::FromStr, sync::OnceLock};

use clap::{ArgAction, Parser, Subcommand};

use nym_vpn_lib::UserAgent;

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

    /// Override the default user agent string.
    #[arg(long, value_parser = parse_user_agent)]
    pub user_agent: Option<UserAgent>,

    /// Format output as JSON
    #[arg(long, action = ArgAction::SetTrue)]
    pub json_output: bool,

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
        matches!(self.command, Some(Command::RunAsService(_)))
    }
}

#[derive(Debug, Copy, Clone, Default, Subcommand)]
#[allow(clippy::enum_variant_names)]
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
    RunAsService(RunArgs),

    /// Run daemon standalone with some additional arguments
    RunWithArgs(RunArgs),

    /// Run daemon standalone
    #[default]
    #[clap(skip)]
    RunStandalone,
}

#[derive(Debug, Default, Clone, Copy, clap::Args)]
pub struct RunArgs {
    /// WARNING this flag is UNSAFE and should only be used for debug purposes.
    /// It disables the checks that the daemon does on the clients to ensure
    /// they come from legitimate sources (Nym signed applications/authenticated users)
    #[clap(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub disable_client_verification: bool,
}

fn parse_user_agent(user_agent: &str) -> Result<UserAgent, String> {
    UserAgent::from_str(user_agent).map_err(|e| e.to_string())
}
