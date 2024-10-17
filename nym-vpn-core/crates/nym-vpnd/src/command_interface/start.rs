// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::SocketAddr, path::PathBuf};

use nym_vpn_lib::tunnel_state_machine::MixnetEvent;
use nym_vpn_proto::{nym_vpnd_server::NymVpndServer, VPN_FD_SET};
use tokio::{
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
    task::{JoinHandle, JoinSet},
};
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;

use super::{
    config::{default_socket_path, default_uri_addr},
    listener::CommandInterface,
    socket_stream::setup_socket_stream,
};
use crate::service::{VpnServiceCommand, VpnServiceStateChange};

fn grpc_span(req: &http::Request<()>) -> tracing::Span {
    let service = req.uri().path().trim_start_matches('/');
    let method = service.split('/').last().unwrap_or(service);
    if service.contains("grpc.reflection.v1") {
        let span = tracing::trace_span!("grpc_reflection");
        tracing::trace!(target: "grpc_reflection", "← {} {:?}", method, req.body());
        return span;
    }
    if service.contains("grpc.health.v1") {
        let span = tracing::debug_span!("grpc_health");
        tracing::debug!(target: "grpc_health", "← {} {:?}", method, req.body());
        return span;
    }
    let span = tracing::info_span!("grpc_vpnd");
    tracing::info!(target: "grpc_vpnd", "← {} {:?}", method, req.body());
    span
}

async fn run_uri_listener(
    vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,
    status_rx: broadcast::Receiver<MixnetEvent>,
    addr: SocketAddr,
    shutdown_token: CancellationToken,
) -> Result<(), tonic::transport::Error> {
    tracing::info!("Starting HTTP listener on: {addr}");

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<NymVpndServer<CommandInterface>>()
        .await;
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(VPN_FD_SET)
        .build()
        .unwrap();
    let command_interface =
        CommandInterface::new_with_uri(vpn_state_changes_rx, vpn_command_tx, status_rx, addr);

    Server::builder()
        .trace_fn(grpc_span)
        .add_service(health_service)
        .add_service(reflection_service)
        .add_service(NymVpndServer::new(command_interface))
        .serve_with_shutdown(addr, shutdown_token.cancelled_owned())
        .await
}

async fn run_socket_listener(
    vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,
    status_rx: broadcast::Receiver<MixnetEvent>,
    socket_path: PathBuf,
    shutdown_token: CancellationToken,
) -> Result<(), tonic::transport::Error> {
    tracing::info!("Starting socket listener on: {}", socket_path.display());
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<NymVpndServer<CommandInterface>>()
        .await;
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(VPN_FD_SET)
        .build()
        .unwrap();
    let command_interface = CommandInterface::new_with_path(
        vpn_state_changes_rx,
        vpn_command_tx,
        status_rx,
        &socket_path,
    );
    command_interface.remove_previous_socket_file();

    // Wrap the unix socket into a stream that can be used by tonic
    let incoming = setup_socket_stream(&socket_path);

    Server::builder()
        .trace_fn(grpc_span)
        .add_service(health_service)
        .add_service(reflection_service)
        .add_service(NymVpndServer::new(command_interface))
        .serve_with_incoming_shutdown(incoming, shutdown_token.cancelled_owned())
        .await
}

#[derive(Default)]
pub(crate) struct CommandInterfaceOptions {
    pub(crate) disable_socket_listener: bool,
    pub(crate) enable_http_listener: bool,
}

pub(crate) fn start_command_interface(
    vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
    status_rx: broadcast::Receiver<MixnetEvent>,
    command_interface_options: Option<CommandInterfaceOptions>,
    shutdown_token: CancellationToken,
) -> (JoinHandle<()>, UnboundedReceiver<VpnServiceCommand>) {
    tracing::info!("Starting command interface");

    let (vpn_command_tx, vpn_command_rx) = mpsc::unbounded_channel();
    let command_interface_options = command_interface_options.unwrap_or_default();
    let socket_path = default_socket_path();
    let uri_addr = default_uri_addr();

    let handle = tokio::spawn(async move {
        let mut join_set = JoinSet::new();

        if !command_interface_options.disable_socket_listener {
            join_set.spawn(run_socket_listener(
                vpn_state_changes_rx.resubscribe(),
                vpn_command_tx.clone(),
                status_rx.resubscribe(),
                socket_path.to_path_buf(),
                shutdown_token.child_token(),
            ));
        }

        if command_interface_options.enable_http_listener {
            join_set.spawn(run_uri_listener(
                vpn_state_changes_rx,
                vpn_command_tx.clone(),
                status_rx.resubscribe(),
                uri_addr,
                shutdown_token.child_token(),
            ));
        }

        let mut i = 0;

        while let Some(result) = join_set.join_next().await {
            i += 1;

            match result {
                Ok(Ok(_)) => {
                    tracing::trace!("Listener ({i}) has finished.")
                }
                Ok(Err(e)) => {
                    tracing::error!("Listener ({i}) exited with error: {e}");
                }
                Err(e) => {
                    tracing::error!("Failed to join on listener ({i}): {e}");
                }
            }
        }

        tracing::info!("Command interface exiting");
    });

    (handle, vpn_command_rx)
}
