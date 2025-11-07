// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
mod service_binary;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct ProgramArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Prepare daemon for restart
    PrepareRestart,

    /// Install service
    InstallService,
}

fn main() -> Result<()> {
    let args = ProgramArgs::parse();
    match args.command {
        Command::PrepareRestart => {
            // todo: implement preparation for restart
            Ok(())
        }
        Command::InstallService => {
            // todo: implement service installation
            #[cfg(target_os = "macos")]
            macos::install_service()?;

            Ok(())
        }
    }
}
