// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use clap::Parser;
use cli::Command;
use nym_gateway_directory::{EntryPoint, ExitPoint, NodeIdentity, Recipient};
use nym_vpn_proto::{
    nym_vpnd_client::NymVpndClient, ConnectRequest, DisconnectRequest, ImportUserCredentialRequest,
    StatusRequest,
};
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use tonic::transport::{Channel as TonicChannel, Endpoint as TonicEndpoint};

use crate::cli::ImportCredentialTypeEnum;

mod cli;

fn parse_entry_point(args: &cli::ConnectArgs) -> anyhow::Result<Option<EntryPoint>> {
    if let Some(ref entry_gateway_id) = args.entry.entry_gateway_id {
        Ok(Some(EntryPoint::Gateway {
            identity: NodeIdentity::from_base58_string(entry_gateway_id.clone())
                .map_err(|_| anyhow!("Failed to parse gateway id"))?,
        }))
    } else if let Some(ref entry_gateway_country) = args.entry.entry_gateway_country {
        Ok(Some(EntryPoint::Location {
            location: entry_gateway_country.clone(),
        }))
    } else if args.entry.entry_gateway_low_latency {
        Ok(Some(EntryPoint::RandomLowLatency))
    } else if args.entry.entry_gateway_random {
        Ok(Some(EntryPoint::Random))
    } else {
        Ok(None)
    }
}

fn parse_exit_point(args: &cli::ConnectArgs) -> anyhow::Result<Option<ExitPoint>> {
    if let Some(ref exit_router_address) = args.exit.exit_router_address {
        Ok(Some(ExitPoint::Address {
            address: Recipient::try_from_base58_string(exit_router_address.clone())
                .map_err(|_| anyhow!("Failed to parse exit node address"))?,
        }))
    } else if let Some(ref exit_router_id) = args.exit.exit_gateway_id {
        Ok(Some(ExitPoint::Gateway {
            identity: NodeIdentity::from_base58_string(exit_router_id.clone())
                .map_err(|_| anyhow!("Failed to parse gateway id"))?,
        }))
    } else if let Some(ref exit_gateway_country) = args.exit.exit_gateway_country {
        Ok(Some(ExitPoint::Location {
            location: exit_gateway_country.clone(),
        }))
    } else if args.exit.exit_gateway_random {
        Ok(Some(ExitPoint::Random))
    } else {
        Ok(None)
    }
}

fn parse_encoded_credential_data(raw: &str) -> bs58::decode::Result<Vec<u8>> {
    bs58::decode(raw).into_vec()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::CliArgs::parse();
    match args.command {
        Command::Connect(ref connect_args) => connect(&args, connect_args).await?,
        Command::Disconnect => disconnect(&args).await?,
        Command::Status => status(&args).await?,
        Command::ImportCredential(ref import_args) => import_credential(&args, import_args).await?,
    }
    Ok(())
}

fn get_socket_path() -> PathBuf {
    Path::new("/var/run/nym-vpn.sock").to_path_buf()
}

async fn get_channel(socket_path: PathBuf) -> anyhow::Result<TonicChannel> {
    // NOTE: the uri here is ignored
    Ok(TonicEndpoint::from_static("http://[::1]:53181")
        .connect_with_connector(tower::service_fn(move |_| {
            IpcEndpoint::connect(socket_path.clone())
        }))
        .await?)
}

fn default_endpoint() -> String {
    "http://[::1]:53181".to_string()
}

async fn get_client(args: &cli::CliArgs) -> anyhow::Result<NymVpndClient<TonicChannel>> {
    if args.http {
        let endpoint = default_endpoint();
        let client = NymVpndClient::connect(endpoint.clone())
            .await
            .with_context(|| format!("Failed to connect to: {}", endpoint))?;
        Ok(client)
    } else {
        let socket_path = get_socket_path();
        let channel = get_channel(socket_path.clone())
            .await
            .with_context(|| format!("Failed to connect to: {:?}", socket_path))?;
        let client = NymVpndClient::new(channel);
        Ok(client)
    }
}

#[allow(unused)]
fn new_entry_node_location(country_code: &str) -> nym_vpn_proto::EntryNode {
    nym_vpn_proto::EntryNode {
        entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::Location(
            nym_vpn_proto::Location {
                two_letter_iso_country_code: country_code.to_string(),
            },
        )),
    }
}

fn into_entry_point(entry: EntryPoint) -> nym_vpn_proto::EntryNode {
    match entry {
        EntryPoint::Gateway { identity } => nym_vpn_proto::EntryNode {
            entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::Gateway(
                nym_vpn_proto::Gateway {
                    id: identity.to_base58_string(),
                },
            )),
        },
        EntryPoint::Location { location } => nym_vpn_proto::EntryNode {
            entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::Location(
                nym_vpn_proto::Location {
                    two_letter_iso_country_code: location,
                },
            )),
        },
        EntryPoint::RandomLowLatency => nym_vpn_proto::EntryNode {
            entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::RandomLowLatency(
                nym_vpn_proto::Empty {},
            )),
        },
        EntryPoint::Random => nym_vpn_proto::EntryNode {
            entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::Random(
                nym_vpn_proto::Empty {},
            )),
        },
    }
}

fn into_exit_point(exit: ExitPoint) -> nym_vpn_proto::ExitNode {
    match exit {
        ExitPoint::Address { address } => nym_vpn_proto::ExitNode {
            exit_node_enum: Some(nym_vpn_proto::exit_node::ExitNodeEnum::Address(
                nym_vpn_proto::Address {
                    nym_address: address.to_string(),
                },
            )),
        },
        ExitPoint::Gateway { identity } => nym_vpn_proto::ExitNode {
            exit_node_enum: Some(nym_vpn_proto::exit_node::ExitNodeEnum::Gateway(
                nym_vpn_proto::Gateway {
                    id: identity.to_base58_string(),
                },
            )),
        },
        ExitPoint::Location { location } => nym_vpn_proto::ExitNode {
            exit_node_enum: Some(nym_vpn_proto::exit_node::ExitNodeEnum::Location(
                nym_vpn_proto::Location {
                    two_letter_iso_country_code: location,
                },
            )),
        },
        ExitPoint::Random => nym_vpn_proto::ExitNode {
            exit_node_enum: Some(nym_vpn_proto::exit_node::ExitNodeEnum::Random(
                nym_vpn_proto::Empty {},
            )),
        },
    }
}

async fn connect(args: &cli::CliArgs, connect_args: &cli::ConnectArgs) -> anyhow::Result<()> {
    // Setup connect arguments
    let entry = parse_entry_point(connect_args)?;
    let exit = parse_exit_point(connect_args)?;

    let request = tonic::Request::new(ConnectRequest {
        entry: entry.map(into_entry_point),
        exit: exit.map(into_exit_point),
    });

    let mut client = get_client(args).await?;
    let response = client.vpn_connect(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}

async fn disconnect(args: &cli::CliArgs) -> anyhow::Result<()> {
    let mut client = get_client(args).await?;
    let request = tonic::Request::new(DisconnectRequest {});
    let response = client.vpn_disconnect(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}

async fn status(args: &cli::CliArgs) -> anyhow::Result<()> {
    let mut client = get_client(args).await?;
    let request = tonic::Request::new(StatusRequest {});
    let response = client.vpn_status(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}

async fn import_credential(
    args: &cli::CliArgs,
    import_args: &cli::ImportCredentialArgs,
) -> anyhow::Result<()> {
    let import_type: ImportCredentialTypeEnum = import_args.credential_type.clone().into();
    let raw_credential = match import_type {
        ImportCredentialTypeEnum::Path(path) => std::fs::read(path)?,
        ImportCredentialTypeEnum::Data(data) => parse_encoded_credential_data(&data)?,
    };
    let request = tonic::Request::new(ImportUserCredentialRequest {
        credential: raw_credential,
    });
    let mut client = get_client(args).await?;
    let response = client.import_user_credential(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}
