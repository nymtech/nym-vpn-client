// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::io::Write;

use anyhow::Result;
use clap::Parser;

use nym_vpn_proto::rpc_client::RpcClient;

use crate::Command;

pub(crate) async fn execute(rpc_client: RpcClient) -> Result<()> {
    loop {
        let line = readline()?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match respond(line, rpc_client.clone()).await {
            Ok(quit) => {
                if quit {
                    break;
                }
            }
            Err(err) => {
                write!(std::io::stdout(), "{err}")?;
                std::io::stdout().flush()?;
            }
        }
    }

    Ok(())
}

async fn respond(line: &str, rpc_client: RpcClient) -> Result<bool> {
    let args = shlex::split(line).ok_or(anyhow::anyhow!("error: Invalid quoting"))?;
    let cli = SessionCli::try_parse_from(args)?;
    match cli.command {
        Command::StartSession => {
            writeln!(
                std::io::stdout(),
                "Can't start a session inside an existing session ..."
            )?;
            std::io::stdout().flush()?;
        }
        Command::ExitSession => {
            writeln!(std::io::stdout(), "Exiting ...")?;
            std::io::stdout().flush()?;
            return Ok(true);
        }
        Command::Connect { wait } => Command::connect(rpc_client, wait).await?,
        Command::Reconnect => Command::reconnect(rpc_client).await?,
        Command::Disconnect { wait } => Command::disconnect(rpc_client, wait).await?,
        Command::Status { listen } => Command::status(rpc_client, listen).await?,
        Command::Info => Command::info(rpc_client).await?,
        Command::GetConfig => Command::get_config(rpc_client).await?,
        Command::Gateway(args) => args.execute(rpc_client).await?,
        Command::Tunnel { subcommand } => subcommand.execute(rpc_client).await?,
        Command::Lan { subcommand } => subcommand.execute(rpc_client).await?,
        Command::AdBlock { subcommand } => subcommand.execute(rpc_client).await?,
        Command::Dns { subcommand } => subcommand.execute(rpc_client).await?,
        Command::Network { subcommand } => subcommand.execute(rpc_client).await?,
        Command::Account { subcommand } => subcommand.execute(rpc_client).await?,
        Command::Device(args) => args.execute(rpc_client).await?,
        Command::Sentry { subcommand } => subcommand.execute(rpc_client).await?,
        Command::Socks5 { subcommand } => subcommand.execute(rpc_client).await?,
        Command::GeoExclusion { subcommand } => subcommand.execute(rpc_client).await?,
        Command::NetworkStats { subcommand } => subcommand.execute(rpc_client).await?,
        Command::Diagnostic { subcommand } => {
            crate::commands::diagnostic::execute(subcommand, rpc_client).await?
        }
        Command::Favorites { subcommand } => subcommand.execute().await?,
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        Command::SplitTunnel { subcommand } => subcommand.execute(rpc_client).await?,
    }

    Ok(false)
}

#[derive(Debug, Parser)]
#[command(multicall = true)]
struct SessionCli {
    #[command(subcommand)]
    command: Command,
}

fn readline() -> Result<String> {
    write!(std::io::stdout(), "$ ")?;
    std::io::stdout().flush()?;
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    Ok(buffer)
}
