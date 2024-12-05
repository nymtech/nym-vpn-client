// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, sync::OnceLock};

use clap::{Args, Parser};

// Helper for passing LONG_VERSION to clap
fn pretty_build_info_static() -> &'static str {
    static PRETTY_BUILD_INFORMATION: OnceLock<String> = OnceLock::new();
    PRETTY_BUILD_INFORMATION.get_or_init(|| nym_bin_common::bin_info_local_vergen!().pretty_print())
}

#[derive(Parser, Clone, Debug)]
#[clap(author = "Nymtech", version, about, long_version = pretty_build_info_static())]
pub(crate) struct CliArgs {
    /// Path pointing to an env file describing the network.
    #[arg(short, long, value_parser = check_path)]
    pub(crate) config_env_file: Option<PathBuf>,

    #[arg(short, long, hide = true)]
    pub(crate) network: Option<String>,

    #[arg(long)]
    pub(crate) enable_http_listener: bool,

    #[arg(long)]
    pub(crate) disable_socket_listener: bool,

    /// Override the default user agent string.
    #[arg(long, value_parser = parse_user_agent)]
    pub(crate) user_agent: Option<nym_vpn_lib::UserAgent>,

    #[cfg(windows)]
    #[arg(long)]
    pub(crate) disable_service: bool,

    #[command(flatten)]
    pub(crate) command: Command,
}

#[derive(Args, Debug, Clone)]
#[group(multiple = false)]
pub(crate) struct Command {
    #[cfg(windows)]
    #[arg(long)]
    pub(crate) install: bool,

    #[cfg(windows)]
    #[arg(long)]
    pub(crate) uninstall: bool,

    #[cfg(windows)]
    #[arg(long)]
    pub(crate) start: bool,

    #[arg(long)]
    pub(crate) run_as_service: bool,
}

impl Command {
    #[cfg(windows)]
    pub(crate) fn is_any(&self) -> bool {
        self.install || self.uninstall || self.start || self.run_as_service
    }
}

fn check_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!("Path {:?} does not exist", path));
    }
    if !path.is_file() {
        return Err(format!("Path {:?} is not a file", path));
    }
    Ok(path)
}

// TODO: make use of the From<&str> implementation for UserAgent once that is available in the
// upstream branch.
fn parse_user_agent(user_agent: &str) -> Result<nym_vpn_lib::UserAgent, String> {
    let parts: Vec<&str> = user_agent.split('/').collect();
    if parts.len() != 4 {
        return Err("User agent must have 4 parts".to_string());
    }

    Ok(nym_vpn_lib::UserAgent {
        application: parts[0].to_string(),
        version: parts[1].to_string(),
        platform: parts[2].to_string(),
        git_commit: parts[3].to_string(),
    })
}
