// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, time::Duration};

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

use super::{config::default_socket_path, listener::CommandInterface};
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
        .build_v1()
        .unwrap();
    let command_interface = CommandInterface::new(vpn_command_tx, tunnel_event_rx, network_env);

    // Remove previous socket file in case if the daemon crashed in the prior run and could not clean up the socket file.
    #[cfg(unix)]
    remove_previous_socket_file(&socket_path).await;

    // Wrap the unix socket into a stream that can be used by tonic
    let incoming = nym_ipc::server::create_incoming(socket_path).unwrap();

    Server::builder()
        .trace_fn(grpc_span)
        .add_service(health_service)
        .add_service(reflection_service)
        .add_service(NymVpndServer::new(command_interface))
        .serve_with_incoming_shutdown(incoming, shutdown_token.cancelled_owned())
        .await
}

pub fn start_command_interface(
    tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
    network_env: Network,
    shutdown_token: CancellationToken,
) -> (JoinHandle<()>, UnboundedReceiver<VpnServiceCommand>) {
    tracing::debug!("Starting command interface");

    let (vpn_command_tx, vpn_command_rx) = mpsc::unbounded_channel();

    let handle = tokio::spawn(async move {
        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
        let mut join_set = JoinSet::new();

        let child_token = shutdown_token.child_token();
        join_set.spawn(async move {
            health_reporter
                .set_serving::<NymVpndServer<CommandInterface>>()
                .await;

            child_token.cancelled().await;

            health_reporter
                .set_not_serving::<NymVpndServer<CommandInterface>>()
                .await;

            tracing::info!("Health reporter has finished");
        });

        let child_token = shutdown_token.child_token();
        join_set.spawn(async move {
            match run_socket_listener(
                vpn_command_tx.clone(),
                tunnel_event_rx.resubscribe(),
                default_socket_path(),
                child_token,
                health_service.clone(),
                network_env.clone(),
            )
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

        let delayed_cancel = shutdown_token
            .cancelled()
            .then(|_| sleep(SHUTDOWN_TIMEOUT))
            .fuse();
        tokio::pin!(delayed_cancel);

        loop {
            tokio::select! {
                _ = &mut delayed_cancel => {
                    tracing::info!("Timed out while waiting for listeners to finish, shutting down");
                    join_set.abort_all();
                    break;
                }
                next_result = join_set.join_next() => {
                    match next_result {
                        Some(Ok(_)) => {
                            continue;
                        }
                        Some(Err(e)) => {
                            tracing::error!("Failed to join task: {}", e);
                        }
                        None => {
                            tracing::info!("All listeners have finished, shutting down");
                            break;
                        }
                    }
                }
            }
        }

        tracing::info!("Command interface exiting");
    });

    (handle, vpn_command_rx)
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
