// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::{SinkExt, StreamExt, pin_mut};
use std::path::{Path, PathBuf};

use crate::server_nym::NymTestServer;
use tarpc::server::Channel;
use test_rpc::{
    Service,
    nym_daemon::{NYMVPN_SOCKET_PATH, ServiceStatus},
    transport::GrpcForwarder,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::codec::{Decoder, LengthDelimitedCodec};

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
    const IPC_READ_BUF_SIZE: usize = 16 * 1024;

    let mut srv_read_buf = [0u8; IPC_READ_BUF_SIZE];
    let mut proxy_transport = LengthDelimitedCodec::new().framed(proxy_transport);

    loop {
        // Wait for input from the test manager before connecting to the UDS or named pipe.
        // Connect at the last moment since the daemon may not even be running when the
        // test runner first starts.
        let first_message = match proxy_transport.next().await {
            Some(Ok(bytes)) => {
                if bytes.is_empty() {
                    log::debug!("ignoring EOF from client");
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
                    let _ = proxy_transport.send(bytes::Bytes::new()).await;
                    continue;
                }
            };

        log::info!("🌚 nym daemon: connected");

        if let Err(error) = daemon_socket_endpoint.write_all(&first_message).await {
            log::error!("writing to uds failed: {error}");
            continue;
        }

        loop {
            let srv_read = daemon_socket_endpoint.read(&mut srv_read_buf);
            pin_mut!(srv_read);

            match futures::future::select(srv_read, proxy_transport.next()).await {
                futures::future::Either::Left((read, _)) => match read {
                    Ok(num_bytes) => {
                        if num_bytes == 0 {
                            log::debug!("uds EOF; restarting server");
                            break;
                        }
                        if let Err(error) = proxy_transport
                            .send(srv_read_buf[..num_bytes].to_vec().into())
                            .await
                        {
                            log::error!("writing to client channel failed: {error}");
                            break;
                        }
                    }
                    Err(error) => {
                        log::error!("reading from uds failed: {error}");
                        let _ = proxy_transport.send(bytes::Bytes::new()).await;
                        break;
                    }
                },
                futures::future::Either::Right((read, _)) => match read {
                    Some(Ok(bytes)) => {
                        if bytes.is_empty() {
                            log::debug!("management interface EOF; restarting server");
                            break;
                        }
                        if let Err(error) = daemon_socket_endpoint.write_all(&bytes).await {
                            log::error!("writing to uds failed: {error}");
                            break;
                        }
                    }
                    Some(Err(error)) => {
                        log::error!("daemon client channel error: {error}");
                        break;
                    }
                    None => break,
                },
            }
        }

        log::info!("🌚 nym daemon: disconnected");
    }
}
