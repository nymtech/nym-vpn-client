// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use nym_vpn_proto::{
    ConnectRequest, DisconnectRequest, ImportUserCredentialRequest, StatusRequest,
};
use vpnd_client::ClientType;

use crate::{
    cli::{Command, ImportCredentialTypeEnum},
    protobuf_conversion::{into_entry_point, into_exit_point},
};

mod cli;
mod config;
mod protobuf_conversion;
mod vpnd_client;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::CliArgs::parse();
    let client_type = if args.http {
        vpnd_client::ClientType::Http
    } else {
        vpnd_client::ClientType::Ipc
    };
    match args.command {
        Command::Connect(ref connect_args) => connect(client_type, connect_args).await?,
        Command::Disconnect => disconnect(client_type).await?,
        Command::Status => status(client_type).await?,
        Command::ImportCredential(ref import_args) => {
            import_credential(client_type, import_args).await?
        }
    }
    Ok(())
}

async fn connect(client_type: ClientType, connect_args: &cli::ConnectArgs) -> Result<()> {
    let entry = cli::parse_entry_point(connect_args)?;
    let exit = cli::parse_exit_point(connect_args)?;

    let request = tonic::Request::new(ConnectRequest {
        entry: entry.map(into_entry_point),
        exit: exit.map(into_exit_point),
        disable_routing: false,
        enable_two_hop: false,
        enable_poisson_rate: false,
        disable_background_cover_traffic: false,
        enable_credentials_mode: false,
    });

    let mut client = vpnd_client::get_client(client_type).await?;
    let response = client.vpn_connect(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}

async fn disconnect(client_type: ClientType) -> Result<()> {
    let mut client = vpnd_client::get_client(client_type).await?;
    let request = tonic::Request::new(DisconnectRequest {});
    let response = client.vpn_disconnect(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}

async fn status(client_type: ClientType) -> Result<()> {
    let mut client = vpnd_client::get_client(client_type).await?;
    let request = tonic::Request::new(StatusRequest {});
    let response = client.vpn_status(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}

async fn import_credential(
    client_type: ClientType,
    import_args: &cli::ImportCredentialArgs,
) -> Result<()> {
    let import_type: ImportCredentialTypeEnum = import_args.credential_type.clone().into();
    let raw_credential = match import_type {
        ImportCredentialTypeEnum::Path(path) => std::fs::read(path)?,
        ImportCredentialTypeEnum::Data(data) => parse_encoded_credential_data(&data)?,
    };
    let request = tonic::Request::new(ImportUserCredentialRequest {
        credential: raw_credential,
    });
    let mut client = vpnd_client::get_client(client_type).await?;
    let response = client.import_user_credential(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}

fn parse_encoded_credential_data(raw: &str) -> bs58::decode::Result<Vec<u8>> {
    bs58::decode(raw).into_vec()
}
