// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use anyhow::Context;
use nym_vpn_proto::nym_vpnd_client::NymVpndClient;
use tonic::transport::{Channel as TonicChannel, Endpoint as TonicEndpoint};

use crate::config;

#[derive(Debug, Clone)]
pub enum ClientType {
    Http,
    Ipc,
}

pub async fn get_client(client_type: &ClientType) -> anyhow::Result<NymVpndClient<TonicChannel>> {
    match client_type {
        ClientType::Http => get_http_client().await,
        ClientType::Ipc => get_ipc_client().await,
    }
    .with_context(|| "failed to connect to `nym-vpnd`. Is it running?")
}

async fn get_channel(socket_path: PathBuf) -> anyhow::Result<TonicChannel> {
    // NOTE: the uri here is ignored
    Ok(TonicEndpoint::from_static("http://[::1]:53181")
        .connect_with_connector(tower::service_fn(move |_| {
            nym_ipc::client::connect(socket_path.clone())
        }))
        .await?)
}

async fn get_http_client() -> anyhow::Result<NymVpndClient<TonicChannel>> {
    let endpoint = config::default_endpoint();
    let client = NymVpndClient::connect(endpoint.clone())
        .await
        .with_context(|| format!("failed to connect to: {}", endpoint))?;
    Ok(client)
}

async fn get_ipc_client() -> anyhow::Result<NymVpndClient<TonicChannel>> {
    let socket_path = config::get_socket_path();
    let channel = get_channel(socket_path.clone())
        .await
        .with_context(|| format!("failed to connect to: {:?}", socket_path))?;
    let client = NymVpndClient::new(channel);
    Ok(client)
}
