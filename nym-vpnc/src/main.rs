// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use clap::{Parser, Subcommand};
use nym_vpn_proto::{
    nym_vpnd_client::NymVpndClient, ConnectRequest, DisconnectRequest, StatusRequest,
};
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use tonic::transport::Endpoint;

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
    Status,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    match args.command {
        Command::Connect => connect().await?,
        Command::Disconnect => disconnect().await?,
        Command::Status => status().await?,
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
    let mut client = NymVpndClient::new(endpoint);
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
    let mut client = NymVpndClient::new(endpoint);
    let request = tonic::Request::new(DisconnectRequest {});
    let response = client.vpn_disconnect(request).await?;
    println!("RESPONSE={:?}", response);
    Ok(())
}

async fn status() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = Path::new("/var/run/nym-vpn.socket");
    let endpoint = Endpoint::from_static("http://[::1]:50051")
        .connect_with_connector(tower::service_fn(move |_| {
            IpcEndpoint::connect(socket_path)
        }))
        .await
        .unwrap();
    let mut client = NymVpndClient::new(endpoint);
    let request = tonic::Request::new(StatusRequest {});
    let response = client.vpn_status(request).await?;
    println!("RESPONSE={:?}", response);
    Ok(())
}
