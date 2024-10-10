// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::SocketAddr, path::PathBuf};

use nym_vpn_proto::{nym_vpnd_server::NymVpndServer, VPN_FD_SET};
use tokio::{
    sync::{
        broadcast,
        mpsc::{UnboundedReceiver, UnboundedSender},
    },
    task::JoinHandle,
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

fn spawn_uri_listener(
    vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,
    addr: SocketAddr,
    shutdown_token: CancellationToken,
) -> JoinHandle<()> {
    tracing::info!("Starting HTTP listener on: {addr}");
    tokio::task::spawn(async move {
        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
        health_reporter
            .set_serving::<NymVpndServer<CommandInterface>>()
            .await;
        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(VPN_FD_SET)
            .build()
            .unwrap();
        let command_interface =
            CommandInterface::new_with_uri(vpn_state_changes_rx, vpn_command_tx, addr);

        Server::builder()
            .trace_fn(grpc_span)
            .add_service(health_service)
            .add_service(reflection_service)
            .add_service(NymVpndServer::new(command_interface))
            .serve_with_shutdown(addr, shutdown_token.cancelled_owned())
            .await
            .unwrap();
    })
}

fn spawn_socket_listener(
    vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,
    socket_path: PathBuf,
    shutdown_token: CancellationToken,
) -> JoinHandle<()> {
    tracing::info!("Starting socket listener on: {}", socket_path.display());
    tokio::task::spawn(async move {
        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
        health_reporter
            .set_serving::<NymVpndServer<CommandInterface>>()
            .await;
        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(VPN_FD_SET)
            .build()
            .unwrap();
        let command_interface =
            CommandInterface::new_with_path(vpn_state_changes_rx, vpn_command_tx, &socket_path);
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
            .unwrap();
    })
}

#[derive(Default)]
pub(crate) struct CommandInterfaceOptions {
    pub(crate) disable_socket_listener: bool,
    pub(crate) enable_http_listener: bool,
}

pub(crate) fn start_command_interface(
    vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
    command_interface_options: Option<CommandInterfaceOptions>,
    shutdown_token: CancellationToken,
) -> (JoinHandle<()>, UnboundedReceiver<VpnServiceCommand>) {
    tracing::info!("Starting command interface");
    // Channel to send commands to the vpn service
    let (vpn_command_tx, vpn_command_rx) = tokio::sync::mpsc::unbounded_channel();

    let command_interface_options = command_interface_options.unwrap_or_default();
    let socket_path = default_socket_path();
    let uri_addr = default_uri_addr();

    let handle = tokio::spawn(async {
        let socket_listener_handle = if !command_interface_options.disable_socket_listener {
            Some(spawn_socket_listener(
                vpn_state_changes_rx.resubscribe(),
                vpn_command_tx.clone(),
                socket_path.to_path_buf(),
                shutdown_token.child_token(),
            ))
        } else {
            None
        };

        let url_listener_handle = if command_interface_options.enable_http_listener {
            Some(spawn_uri_listener(
                vpn_state_changes_rx,
                vpn_command_tx.clone(),
                uri_addr,
                shutdown_token.child_token(),
            ))
        } else {
            None
        };

        // Wait for shutdown signal.
        shutdown_token.cancelled().await;

        tracing::info!("Caught event stop");
        tracing::info!("Signalling shutdown...");
        vpn_command_tx.send(VpnServiceCommand::Shutdown).unwrap();

        // Wait for rpc services to shutdown
        if let Some(socket_listener_handle) = socket_listener_handle {
            socket_listener_handle.await;
        }
        if let Some(url_listener_handle) = url_listener_handle {
            url_listener_handle.await;
        }

        tracing::info!("Command interface exiting");
    });

    (handle, vpn_command_rx)
}
