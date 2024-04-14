// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use nym_task::TaskManager;
use nym_vpn_proto::nym_vpnd_server::NymVpndServer;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tonic::transport::Server;
use tracing::info;

use super::{incoming_stream::setup_incoming_stream, listener::CommandInterface};
use crate::{cli::CliArgs, service::VpnServiceCommand};

fn spawn_uri_listener(vpn_command_tx: UnboundedSender<VpnServiceCommand>, addr: SocketAddr) {
    tokio::task::spawn(async move {
        let command_interface = CommandInterface::new_with_uri(vpn_command_tx, addr);
        Server::builder()
            .add_service(NymVpndServer::new(command_interface))
            .serve(addr)
            .await
            .unwrap();
    });
}

fn spawn_path_listener(vpn_command_tx: UnboundedSender<VpnServiceCommand>, socket_path: PathBuf) {
    tokio::task::spawn(async move {
        let command_interface = CommandInterface::new_with_path(vpn_command_tx, &socket_path);
        let incoming = setup_incoming_stream(&socket_path);
        Server::builder()
            .add_service(NymVpndServer::new(command_interface))
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });
}

pub(crate) fn start_command_interface(
    mut task_manager: TaskManager,
    args: &CliArgs,
) -> (
    std::thread::JoinHandle<()>,
    UnboundedReceiver<VpnServiceCommand>,
) {
    info!("Starting command interface");
    // Channel to send commands to the vpn service
    let (vpn_command_tx, vpn_command_rx) = tokio::sync::mpsc::unbounded_channel();

    let args = args.clone();
    let socket_path = Path::new("/var/run/nym-vpn.socket");
    let uri_addr = "[::1]:53181".parse().unwrap();

    let handle = std::thread::spawn(move || {
        let command_rt = tokio::runtime::Runtime::new().unwrap();
        command_rt.block_on(async move {
            if !args.disable_path_listener {
                spawn_path_listener(vpn_command_tx.clone(), socket_path.to_path_buf());
            }

            if args.enable_http_listener {
                spawn_uri_listener(vpn_command_tx.clone(), uri_addr);
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
            let _ = task_manager.catch_interrupt().await;

            info!("Command interface exiting");
        });
    });

    (handle, vpn_command_rx)
}
