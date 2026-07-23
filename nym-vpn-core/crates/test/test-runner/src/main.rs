// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::{SinkExt, StreamExt};
use std::path::{Path, PathBuf};

use crate::server_nym::NymTestServer;
use tarpc::server::Channel;
use test_rpc::{
    Service,
    nym_daemon::{NYMVPN_SOCKET_PATH, ServiceStatus},
    transport::{GrpcForwarder, forward_framed_bidirectional, length_delimited_framed_halves},
};
use tokio::io::AsyncWriteExt;

mod app_nymvpn;
mod forward;
mod logging;
mod net;
mod package_nym;
mod server_nym;
mod sys;
mod util;

fn get_nymvpn_pipe_status() -> ServiceStatus {
    match Path::new(NYMVPN_SOCKET_PATH).exists() {
        true => ServiceStatus::Running,
        false => ServiceStatus::NotRunning,
    }
}

/// The baud rate of the serial connection between the test manager and the test runner.
/// There is a known issue with setting a baud rate at all or macOS, and the workaround
/// is to set it to zero: https://github.com/serialport/serialport-rs/pull/58
///
/// Keep this constant in sync with `test-manager/src/run_tests.rs`
const BAUD: u32 = if cfg!(target_os = "macos") { 0 } else { 115200 };

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unknown RPC")]
    UnknownRpc,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::init_logger().unwrap();

    let mut args = std::env::args();
    let _ = args.next();
    let path = args.next().expect("serial/COM path must be provided");

    loop {
        log::info!("Connecting to {}", path);

        let serial_stream =
            tokio_serial::SerialStream::open(&tokio_serial::new(&path, BAUD)).unwrap();
        let (runner_transport, daemon_transport, _completion_handle) =
            test_rpc::transport::create_server_transports(serial_stream);

        log::info!("Running server");

        // TODO dz THIS DISABLES TEST-MANAGER COMMS WITH THE MULLVAD DAEMON
        // because we forward to Nym daemon instead!

        // tokio::spawn(forward_to_mullvad_daemon_interface(
        //     mullvad_daemon_transport,
        // ));

        // and enables communication with Nym daemon instead
        tokio::spawn(forward_to_nym_daemon_interface(daemon_transport));

        let server = tarpc::server::BaseChannel::with_defaults(runner_transport);
        server.execute(NymTestServer::default().serve()).await;

        log::error!("Restarting server since it stopped");
    }
}

/// Forward data between the test manager and Mullvad management interface socket
async fn forward_to_nym_daemon_interface(proxy_transport: GrpcForwarder) {
    let (mut proxy_read, mut proxy_write) = length_delimited_framed_halves(proxy_transport);

    loop {
        // Wait for input from the test manager before connecting to the UDS or named pipe.
        // Connect at the last moment since the daemon may not even be running when the
        // test runner first starts.
        let first_message = match proxy_read.next().await {
            Some(Ok(bytes)) => {
                if bytes.is_empty() {
                    if let Err(error) = proxy_write.send(bytes::Bytes::new()).await {
                        log::error!(
                            "failed to acknowledge daemon session synchronization: {error}"
                        );
                        break;
                    }
                    continue;
                }
                bytes
            }
            Some(Err(error)) => {
                log::error!("daemon client channel error: {error}");
                break;
            }
            None => break,
        };

        log::info!("🌚 nym daemon: connecting");

        // let mut daemon_socket_endpoint =
        //     match parity_tokio_ipc::Endpoint::connect(NYMVPN_SOCKET_PATH).await {
        let mut daemon_socket_endpoint =
            match nym_ipc::client::connect(PathBuf::from(NYMVPN_SOCKET_PATH)).await {
                Ok(uds_endpoint) => uds_endpoint.into_inner(),
                Err(error) => {
                    log::error!("🌚 nym daemon: failed to connect: {error}");
                    // send EOF
                    let _ = proxy_write.send(bytes::Bytes::new()).await;
                    continue;
                }
            };

        log::info!("🌚 nym daemon: connected");

        if let Err(error) = daemon_socket_endpoint.write_all(&first_message).await {
            log::error!("writing to uds failed: {error}");
            continue;
        }

        if let Err(error) =
            forward_framed_bidirectional(daemon_socket_endpoint, &mut proxy_read, &mut proxy_write)
                .await
        {
            log::error!("nym daemon forwarding failed: {error}");
        } else {
            log::debug!("nym daemon forwarding reached EOF");
        }

        log::info!("🌚 nym daemon: disconnected");
    }
}

#[cfg(test)]
mod daemon_forwarding_tests {
    use super::forward_to_nym_daemon_interface;
    use bytes::Bytes;
    use futures::{SinkExt, StreamExt};
    use tokio_util::codec::{Decoder, LengthDelimitedCodec};

    #[tokio::test]
    async fn idle_daemon_forwarder_acknowledges_session_synchronization() {
        let (forwarder, peer) = tokio::io::duplex(64);
        let task = tokio::spawn(forward_to_nym_daemon_interface(forwarder));
        let mut peer = LengthDelimitedCodec::new().framed(peer);

        peer.send(Bytes::new()).await.expect("send sync marker");
        let acknowledgement = peer
            .next()
            .await
            .expect("forwarder remains open")
            .expect("sync acknowledgement is valid");

        assert!(acknowledgement.is_empty());
        task.abort();
    }
}
