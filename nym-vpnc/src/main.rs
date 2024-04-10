// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use clap::{Parser, Subcommand};

use crate::vpn_daemon_client::VpnDaemonClient;

tonic::include_proto!("vpnd");

#[derive(Parser)]
#[clap(author = "Nymtech", version, about)]
struct CliArgs {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Connect,
    Disconnect,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    match args.command {
        Command::Connect => connect().await?,
        Command::Disconnect => disconnect().await?,
    }
    Ok(())
}

async fn connect() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = VpnDaemonClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(ConnectRequest {});
    let response = client.vpn_connect(request).await?;
    println!("RESPONSE={:?}", response);
    Ok(())
}

async fn disconnect() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = VpnDaemonClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(DisconnectRequest {});
    let response = client.vpn_disconnect(request).await?;
    println!("RESPONSE={:?}", response);
    Ok(())
}
