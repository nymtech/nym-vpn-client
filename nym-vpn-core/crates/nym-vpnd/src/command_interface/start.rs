// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::SocketAddr, path::PathBuf, time::Duration};

use futures::FutureExt;
use nym_vpn_lib_types::TunnelEvent;
use nym_vpn_network_config::Network;
use nym_vpn_proto::{nym_vpnd_server::NymVpndServer, VPN_FD_SET};
use tokio::{
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
    task::{JoinHandle, JoinSet},
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;
use tonic_health::pb::health_server::{Health, HealthServer};

use super::{
    config::{default_socket_path, default_uri_addr},
    listener::CommandInterface,
    socket_stream::setup_socket_stream,
};
use crate::service::VpnServiceCommand;

// If the shutdown signal is received, we give the listeners a little extra time to finish
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);

fn grpc_span(req: &http::Request<()>) -> tracing::Span {
    let service = req.uri().path().trim_start_matches('/');
    let method = service.split('/').next_back().unwrap_or(service);
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
    let span = tracing::info_span!("grpc_vpnd", req = method);
    tracing::info!(target: "grpc_vpnd", "← {} {:?}", method, req.body());
    span
}

async fn run_uri_listener<T>(
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,
    tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
    addr: SocketAddr,
    shutdown_token: CancellationToken,
    health_service: HealthServer<T>,
    network_env: Network,
) -> Result<(), tonic::transport::Error>
where
    T: Health,
{
    tracing::info!("Starting HTTP listener on: {addr}");
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(VPN_FD_SET)
        .build()
        .unwrap();
    let command_interface =
        CommandInterface::new_with_uri(vpn_command_tx, tunnel_event_rx, addr, network_env);

    Server::builder()
        .trace_fn(grpc_span)
        .add_service(health_service)
        .add_service(reflection_service)
        .add_service(NymVpndServer::new(command_interface))
        .serve_with_shutdown(addr, shutdown_token.cancelled_owned())
        .await
}

async fn run_socket_listener<T>(
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,
    tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
    socket_path: PathBuf,
    shutdown_token: CancellationToken,
    health_service: HealthServer<T>,
    network_env: Network,
) -> Result<(), tonic::transport::Error>
where
    T: Health,
{
    tracing::info!("Starting socket listener on: {}", socket_path.display());
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(VPN_FD_SET)
        .build()
        .unwrap();
    let command_interface =
        CommandInterface::new_with_path(vpn_command_tx, tunnel_event_rx, &socket_path, network_env);
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

async fn setup_health_service(
    shutdown_token: CancellationToken,
) -> (HealthServer<impl Health>, JoinHandle<()>) {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<NymVpndServer<CommandInterface>>()
        .await;

    let handle = tokio::spawn(async move {
        shutdown_token.cancelled().await;
        tracing::debug!("Reporting not serving on health service");
        health_reporter
            .set_not_serving::<NymVpndServer<CommandInterface>>()
            .await;
    });

    (health_service, handle)
}

pub(crate) fn start_command_interface(
    tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
    command_interface_options: Option<CommandInterfaceOptions>,
    network_env: Network,
    shutdown_token: CancellationToken,
) -> (JoinHandle<()>, UnboundedReceiver<VpnServiceCommand>) {
    tracing::info!("Starting command interface");

    let (vpn_command_tx, vpn_command_rx) = mpsc::unbounded_channel();
    let command_interface_options = command_interface_options.unwrap_or_default();
    let socket_path = default_socket_path();
    let uri_addr = default_uri_addr();

    let handle = tokio::spawn(async move {
        let mut join_set = JoinSet::new();

        let (health_service, health_service_handle) =
            setup_health_service(shutdown_token.child_token()).await;

        if !command_interface_options.disable_socket_listener {
            join_set.spawn(run_socket_listener(
                vpn_command_tx.clone(),
                tunnel_event_rx.resubscribe(),
                socket_path.to_path_buf(),
                shutdown_token.child_token(),
                health_service.clone(),
                network_env.clone(),
            ));
        }

        if command_interface_options.enable_http_listener {
            join_set.spawn(run_uri_listener(
                vpn_command_tx.clone(),
                tunnel_event_rx.resubscribe(),
                uri_addr,
                shutdown_token.child_token(),
                health_service,
                network_env,
            ));
        }

        wait_for_shutdown(shutdown_token, join_set, health_service_handle).await;
        tracing::info!("Command interface exiting");
    });

    (handle, vpn_command_rx)
}

async fn wait_for_shutdown(
    shutdown_token: CancellationToken,
    mut join_set: JoinSet<Result<(), tonic::transport::Error>>,
    health_service_handle: JoinHandle<()>,
) {
    let delayed_cancel = shutdown_token
        .cancelled()
        .then(|_| sleep(SHUTDOWN_TIMEOUT))
        .fuse();
    tokio::pin!(delayed_cancel);

    let mut i = 0;
    loop {
        tokio::select! {
            _ = &mut delayed_cancel => {
                tracing::info!("Shutdown timeout reached, cancelling all listeners");
                join_set.abort_all();
            }
            result = join_set.join_next() => match result {
                Some(result) => {
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
                },
                None => {
                    tracing::trace!("All listeners have finished");
                    break;
                }
            }
        }
    }

    health_service_handle
        .await
        .inspect_err(|e| tracing::error!("Failed to join on health reporter: {e}"))
        .ok();
}
