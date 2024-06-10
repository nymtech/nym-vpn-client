// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use nym_vpn_proto::{
    ConnectRequest, DisconnectRequest, Empty, ImportUserCredentialRequest, StatusRequest,
};
use vpnd_client::ClientType;

use crate::{
    cli::{Command, ImportCredentialTypeEnum},
    protobuf_conversion::{into_entry_point, into_exit_point, ipaddr_into_string},
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
        Command::ListenToStatus => listen_to_status(client_type).await?,
        Command::ListenToStateChanges => listen_to_state_changes(client_type).await?,
    }
    Ok(())
}

async fn connect(client_type: ClientType, connect_args: &cli::ConnectArgs) -> Result<()> {
    let entry = cli::parse_entry_point(connect_args)?;
    let exit = cli::parse_exit_point(connect_args)?;

    let request = tonic::Request::new(ConnectRequest {
        entry: entry.map(into_entry_point),
        exit: exit.map(into_exit_point),
        dns: connect_args.dns.map(ipaddr_into_string),
        disable_routing: connect_args.disable_routing,
        enable_two_hop: connect_args.enable_two_hop,
        enable_poisson_rate: connect_args.enable_poisson_rate,
        disable_background_cover_traffic: connect_args.disable_background_cover_traffic,
        enable_credentials_mode: connect_args.enable_credentials_mode,
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

    let utc_since = response
        .details
        .and_then(|details| details.since)
        .map(|timestamp| {
            time::OffsetDateTime::from_unix_timestamp(timestamp.seconds)
                .map(|t| t + time::Duration::nanoseconds(timestamp.nanos as i64))
        });

    if let Some(utc_since) = utc_since {
        match utc_since {
            Ok(utc_since) => {
                println!("since (utc): {:?}", utc_since);
                println!("duration: {}", time::OffsetDateTime::now_utc() - utc_since);
            }
            Err(err) => eprintln!("failed to parse timestamp: {err}"),
        }
    }
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

async fn listen_to_status(client_type: ClientType) -> Result<()> {
    let mut client = vpnd_client::get_client(client_type).await?;
    let request = tonic::Request::new(Empty {});
    let mut stream = client
        .listen_to_connection_status(request)
        .await?
        .into_inner();
    while let Some(response) = stream.message().await? {
        println!("{:?}", response);
    }
    Ok(())
}

async fn listen_to_state_changes(client_type: ClientType) -> Result<()> {
    let mut client = vpnd_client::get_client(client_type).await?;
    let request = tonic::Request::new(Empty {});
    let mut stream = client
        .listen_to_connection_state_changes(request)
        .await?
        .into_inner();
    while let Some(response) = stream.message().await? {
        println!("{:?}", response);
    }
    Ok(())
}
