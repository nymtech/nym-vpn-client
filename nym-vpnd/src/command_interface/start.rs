// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use futures::TryStreamExt;
use nym_task::TaskManager;
use tokio::sync::mpsc::UnboundedReceiver;
use tonic::transport::Server;
use tracing::info;

use crate::service::VpnServiceCommand;

use super::listener::CommandInterface;

use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::ReadBuf;
use tonic::transport::server::Connected;

#[derive(Debug)]
struct StreamBox<T: AsyncRead + AsyncWrite>(pub T);

impl<T: AsyncRead + AsyncWrite> Connected for StreamBox<T> {
    type ConnectInfo = Option<()>;

    fn connect_info(&self) -> Self::ConnectInfo {
        None
    }
}
impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for StreamBox<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}
impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for StreamBox<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

fn setup_incoming(
    socket_path: &Path,
) -> impl futures::Stream<Item = Result<impl AsyncRead + AsyncWrite, std::io::Error>> {
    let mut endpoint = parity_tokio_ipc::Endpoint::new(socket_path.to_string_lossy().to_string());
    endpoint.set_security_attributes(
        parity_tokio_ipc::SecurityAttributes::allow_everyone_create()
            .unwrap()
            .set_mode(0o766)
            .unwrap(),
    );
    endpoint.incoming().unwrap()
}

pub(crate) fn start_command_interface(
    mut task_manager: TaskManager,
) -> (
    std::thread::JoinHandle<()>,
    UnboundedReceiver<VpnServiceCommand>,
) {
    info!("Starting unix socket command interface");
    // Channel to send commands to the vpn service
    let (vpn_command_tx, vpn_command_rx) = tokio::sync::mpsc::unbounded_channel();

    let socket_path = Path::new("/var/run/nym-vpn.socket");
    let handle = std::thread::spawn(move || {
        let command_rt = tokio::runtime::Runtime::new().unwrap();
        command_rt.block_on(async {
            // Spawn command interface
            tokio::task::spawn(async {
                let c = CommandInterface::new(vpn_command_tx, socket_path);
                let incoming = setup_incoming(socket_path).map_ok(StreamBox);
                Server::builder()
                    .add_service(crate::nym_vpn_service_server::NymVpnServiceServer::new(c))
                    .serve_with_incoming(incoming)
                    .await
                    .unwrap();

                // c .listen() .await
            });

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
