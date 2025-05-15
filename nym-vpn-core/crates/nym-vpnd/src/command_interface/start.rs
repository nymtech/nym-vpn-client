// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_vpn_lib_types::TunnelEvent;
use nym_vpn_network_config::Network;
use nym_vpn_proto::nym_vpnd_server::NymVpndServer;
use tokio::{
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver},
    },
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;

use super::{
    config::default_socket_path, error::CommandInterfaceError, listener::CommandInterface,
};
use crate::service::VpnServiceCommand;

// If the shutdown signal is received, we give the listeners a little extra time to finish
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);

fn grpc_span(req: &http::Request<()>) -> tracing::Span {
    let service = req.uri().path().trim_start_matches('/');
    let method = service.split('/').next_back().unwrap_or(service);
    let span = tracing::info_span!("grpc_vpnd", req = method);
    tracing::info!(target: "grpc_vpnd", "← {} {:?}", method, req.body());
    span
}

pub async fn start_command_interface(
    tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
    network_env: Network,
    shutdown_token: CancellationToken,
) -> Result<(JoinHandle<()>, UnboundedReceiver<VpnServiceCommand>), CommandInterfaceError> {
    tracing::debug!("Starting command interface");

    let socket_path = default_socket_path();
    let (vpn_command_tx, vpn_command_rx) = mpsc::unbounded_channel();

    // Remove previous socket file in case if the daemon crashed in the prior run and could not clean up the socket file.
    #[cfg(unix)]
    remove_previous_socket_file(&socket_path).await;
    tracing::info!("Starting socket listener on: {}", socket_path.display());

    // Wrap the unix socket or named pipe into a stream that can be used by tonic
    let incoming = nym_ipc::server::create_incoming(socket_path.clone()).map_err(|source| {
        CommandInterfaceError::CreateIncoming {
            socket_path,
            source,
        }
    })?;

    let server_handle = tokio::spawn(async move {
        let incoming_shutdown_token = shutdown_token.child_token();
        let socket_listener_handle = tokio::spawn(async move {
            let command_interface =
                CommandInterface::new(vpn_command_tx, tunnel_event_rx, network_env);

            let server = Server::builder()
                .trace_fn(grpc_span)
                .add_service(NymVpndServer::new(command_interface));

            match server
                .serve_with_incoming_shutdown(incoming, incoming_shutdown_token.cancelled_owned())
                .await
            {
                Ok(()) => {
                    tracing::info!("Socket listener has finished");
                }
                Err(e) => {
                    tracing::error!("Socket listener exited with error: {}", e);
                }
            }
        });

        shutdown_token.cancelled().await;
        match tokio::time::timeout(SHUTDOWN_TIMEOUT, socket_listener_handle).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => tracing::error!("Failed to join on socket listener: {}", e),
            Err(_) => tracing::warn!("Socket listener did not finish in time"),
        }

        tracing::info!("Command interface exiting");
    });

    Ok((server_handle, vpn_command_rx))
}

#[cfg(unix)]
async fn remove_previous_socket_file(socket_path: &std::path::Path) {
    match tokio::fs::remove_file(socket_path).await {
        Ok(_) => tracing::info!(
            "Removed previous command interface socket: {}",
            socket_path.display()
        ),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            tracing::error!(
                "Failed to remove previous command interface socket: {:?}",
                err
            );
        }
    }
}
