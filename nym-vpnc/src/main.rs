// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use clap::{Parser, Subcommand};
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use tonic::transport::Endpoint;

use crate::nym_vpn_service_client::NymVpnServiceClient;

tonic::include_proto!("nym.vpn");

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
    let socket_path = Path::new("/var/run/nym-vpn.socket");
    let endpoint = Endpoint::from_static("http://[::1]:50051")
        .connect_with_connector(tower::service_fn(move |_| {
            IpcEndpoint::connect(socket_path)
        }))
        .await
        .unwrap();
    // let mut client = NymVpnServiceClient::connect("http://[::1]:50051").await?;
    let mut client = NymVpnServiceClient::new(endpoint);
    let request = tonic::Request::new(ConnectRequest {});
    let response = client.vpn_connect(request).await?;
    println!("RESPONSE={:?}", response);
    Ok(())
}

async fn disconnect() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = Path::new("/var/run/nym-vpn.socket");
    let endpoint = Endpoint::from_static("http://[::1]:50051")
        .connect_with_connector(tower::service_fn(move |_| {
            IpcEndpoint::connect(socket_path)
        }))
        .await
        .unwrap();
    // let mut client = NymVpnServiceClient::connect("http://[::1]:50051").await?;
    let mut client = NymVpnServiceClient::new(endpoint);
    let request = tonic::Request::new(DisconnectRequest {});
    let response = client.vpn_disconnect(request).await?;
    println!("RESPONSE={:?}", response);
    Ok(())
}
