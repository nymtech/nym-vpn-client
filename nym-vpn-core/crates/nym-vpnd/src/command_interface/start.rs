// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::SocketAddr, path::PathBuf};

use nym_task::TaskManager;
use nym_vpn_proto::{nym_vpnd_server::NymVpndServer, VPN_FD_SET};
use tokio::sync::{
    broadcast,
    mpsc::{UnboundedReceiver, UnboundedSender},
};
use tonic::transport::Server;
use tracing::{debug, debug_span, info, info_span, trace, trace_span, Span};

use super::{
    config::{default_socket_path, default_uri_addr},
    listener::CommandInterface,
    socket_stream::setup_socket_stream,
};
use crate::service::{VpnServiceCommand, VpnServiceStateChange};

fn grpc_span(req: &http::Request<()>) -> Span {
    let service = req.uri().path().trim_start_matches('/');
    let method = service.split('/').last().unwrap_or(service);
    if service.contains("grpc.reflection.v1") {
        let span = trace_span!("grpc_reflection");
        trace!(target: "grpc_reflection", "← {} {:?}", method, req.body());
        return span;
    }
    if service.contains("grpc.health.v1") {
        let span = debug_span!("grpc_health");
        debug!(target: "grpc_health", "← {} {:?}", method, req.body());
        return span;
    }
    let span = info_span!("grpc_vpnd");
    info!(target: "grpc_vpnd", "← {} {:?}", method, req.body());
    span
}

fn spawn_uri_listener(
    vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,
    addr: SocketAddr,
) {
    info!("Starting HTTP listener on: {addr}");
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
            .serve(addr)
            .await
            .unwrap();
    });
}

fn spawn_socket_listener(
    vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,
    socket_path: PathBuf,
) {
    info!("Starting socket listener on: {}", socket_path.display());
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
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });
}

#[derive(Default)]
pub(crate) struct CommandInterfaceOptions {
    pub(crate) disable_socket_listener: bool,
    pub(crate) enable_http_listener: bool,
}

pub(crate) fn start_command_interface(
    vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
    mut task_manager: TaskManager,
    command_interface_options: Option<CommandInterfaceOptions>,
    mut event_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
) -> (
    std::thread::JoinHandle<()>,
    UnboundedReceiver<VpnServiceCommand>,
) {
    info!("Starting command interface");
    // Channel to send commands to the vpn service
    let (vpn_command_tx, vpn_command_rx) = tokio::sync::mpsc::unbounded_channel();

    let command_interface_options = command_interface_options.unwrap_or_default();
    let socket_path = default_socket_path();
    let uri_addr = default_uri_addr();

    let handle = std::thread::spawn(move || {
        // Explicitly create a multu-threaded runtime for the command interface.
        // We should consider also explictly setting the number of threads here to something like
        // max(num_vcpu, 4) or something like that.
        let command_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        command_rt.block_on(async move {
            if !command_interface_options.disable_socket_listener {
                spawn_socket_listener(
                    vpn_state_changes_rx.resubscribe(),
                    vpn_command_tx.clone(),
                    socket_path.to_path_buf(),
                );
            }

            if command_interface_options.enable_http_listener {
                spawn_uri_listener(vpn_state_changes_rx, vpn_command_tx.clone(), uri_addr);
            }

            // Using TaskManager::catch_interrupt() here is a bit of a hack that we use for now.
            // The real solution is to:
            //
            // 1. Register signal handler as a separate task. This handler will need the ability to
            //    signal shutdown, which TaskClient currently doesn't have
            //
            // 2. Register error message handler that listens for errors / drops of the tasks, this
            //    will also signal shutdown.
            //
            // 3. (TaskManager needs to be updated so that the vpn-lib does not wait for signals.
            //    Or maybe we can just not call catch_interrupt() at all in vpn-lib.)
            //
            // 4. Then we need to wait for all tasks to finish, like in
            //    TaskManager::wait_for_shutdown(). Since this also listens to ctrl-c, we can't
            //    start this until shutdown has already been signalled.
            //
            // 5. Back up at the top-level where we join the threads, we check for return errors
            //    and handle that there.

            // This call will:
            // Wait for interrupt
            // Send shutdown signal to all tasks
            // Wait for all tasks to finish
            tokio::select! {
                _ = task_manager.catch_interrupt() => {
                    info!("Caught interrupt");
                },
                _ = event_rx.recv() => {
                    info!("Caught event stop");
                    info!("Signalling shutdown...");
                    task_manager.signal_shutdown().ok();
                    info!("Waiting for shutdown ...");
                    task_manager.wait_for_shutdown().await;
                }
            }

            info!("Command interface exiting");
        });
    });

    (handle, vpn_command_rx)
}
