// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(windows)]
use clap::Args;
use clap::Parser;
use nym_vpn_lib::nym_bin_common::bin_info_local_vergen;
use std::{path::PathBuf, sync::OnceLock};

// Helper for passing LONG_VERSION to clap
fn pretty_build_info_static() -> &'static str {
    static PRETTY_BUILD_INFORMATION: OnceLock<String> = OnceLock::new();
    // PRETTY_BUILD_INFORMATION.get_or_init(|| bin_info_local_vergen!().pretty_print())
    PRETTY_BUILD_INFORMATION.get_or_init(|| "PLACEHOLDER".to_string())
}

#[derive(Parser, Clone, Debug)]
#[clap(author = "Nymtech", version, about, long_version = pretty_build_info_static())]
pub(crate) struct CliArgs {
    /// Path pointing to an env file describing the network.
    #[arg(short, long, value_parser = check_path)]
    pub(crate) config_env_file: Option<PathBuf>,

    #[arg(long)]
    pub(crate) enable_http_listener: bool,

    #[arg(long)]
    pub(crate) disable_socket_listener: bool,

    #[cfg(windows)]
    #[arg(long)]
    pub(crate) disable_service: bool,

    #[cfg(windows)]
    #[command(flatten)]
    pub(crate) command: Command,
}

#[cfg(windows)]
#[derive(Args, Debug, Clone)]
#[group(multiple = false)]
pub(crate) struct Command {
    #[arg(long)]
    pub(crate) install: bool,

    #[arg(long)]
    pub(crate) uninstall: bool,

    #[arg(long)]
    pub(crate) start: bool,

    #[arg(long)]
    pub(crate) run_as_service: bool,
}

#[cfg(windows)]
impl Command {
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
