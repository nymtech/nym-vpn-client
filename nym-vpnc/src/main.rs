// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use nym_vpn_proto::{
    nym_vpnd_client::NymVpndClient, ConnectRequest, DisconnectRequest, StatusRequest,
};
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use tonic::transport::{Channel as TonicChannel, Endpoint as TonicEndpoint};

#[derive(Parser)]
#[clap(author = "Nymtech", version, about)]
struct CliArgs {
    /// Use HTTP instead of socket file for IPC with the daemon.
    #[arg(long)]
    http: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Connect,
    Disconnect,
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    match args.command {
        Command::Connect => connect(&args).await?,
        Command::Disconnect => disconnect(&args).await?,
        Command::Status => status(&args).await?,
    }
    Ok(())
}

fn get_socket_path() -> PathBuf {
    Path::new("/var/run/nym-vpn.socket").to_path_buf()
}

async fn get_channel() -> TonicChannel {
    // NOTE: the uri here is ignored
    TonicEndpoint::from_static("http://[::1]:53181")
        .connect_with_connector(tower::service_fn(move |_| {
            IpcEndpoint::connect(get_socket_path())
        }))
        .await
        .unwrap()
}

fn default_endpoint() -> String {
    "http://[::1]:53181".to_string()
}

async fn get_client(args: &CliArgs) -> NymVpndClient<TonicChannel> {
    if args.http {
        NymVpndClient::connect(default_endpoint()).await.unwrap()
    } else {
        NymVpndClient::new(get_channel().await)
    }
}

async fn connect(args: &CliArgs) -> anyhow::Result<()> {
    let mut client = get_client(args).await;
    let request = tonic::Request::new(ConnectRequest {});
    let response = client.vpn_connect(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}

async fn disconnect(args: &CliArgs) -> anyhow::Result<()> {
    let mut client = get_client(args).await;
    let request = tonic::Request::new(DisconnectRequest {});
    let response = client.vpn_disconnect(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}

async fn status(args: &CliArgs) -> anyhow::Result<()> {
    let mut client = get_client(args).await;
    let request = tonic::Request::new(StatusRequest {});
    let response = client.vpn_status(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}
