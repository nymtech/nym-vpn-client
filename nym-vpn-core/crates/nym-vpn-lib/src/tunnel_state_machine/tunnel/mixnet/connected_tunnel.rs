// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{error::Error as StdError, time::Duration};

use nym_task::TaskManager;
use tokio::task::{JoinError, JoinHandle};
use tokio_util::sync::CancellationToken;
use tun::AsyncDevice;

use nym_connection_monitor::ConnectionMonitorTask;
use nym_mixnet_client::SharedMixnetClient;

use super::connector::AssignedAddresses;
use crate::{mixnet::MixnetError, tunnel_state_machine::tunnel::Tombstone};

/// Type representing a connected mixnet tunnel.
pub struct ConnectedTunnel {
    task_manager: TaskManager,
    mixnet_client: SharedMixnetClient,
    assigned_addresses: AssignedAddresses,
    ipr_cancel_token: CancellationToken,
}

impl ConnectedTunnel {
    pub fn new(
        task_manager: TaskManager,
        mixnet_client: SharedMixnetClient,
        assigned_addresses: AssignedAddresses,
        ipr_cancel_token: CancellationToken,
    ) -> Self {
        Self {
            task_manager,
            mixnet_client,
            assigned_addresses,
            ipr_cancel_token,
        }
    }

    pub fn assigned_addresses(&self) -> &AssignedAddresses {
        &self.assigned_addresses
    }

    pub async fn run(self, tun_device: AsyncDevice) -> TunnelHandle {
        let connection_monitor = ConnectionMonitorTask::setup();

        let processor_config =
            crate::mixnet::Config::new(self.assigned_addresses.exit_mix_addresses.0);

        let (ipr_disconnect_tx, ipr_disconnect_rx) = tokio::sync::oneshot::channel();

        let processor_handle = crate::mixnet::start_processor(
            processor_config,
            tun_device,
            self.mixnet_client.clone(),
            &self.task_manager,
            self.assigned_addresses.interface_addresses,
            &connection_monitor,
            self.ipr_cancel_token.clone(),
            ipr_disconnect_tx,
        )
        .await;

        let mixnet_client_sender = self.mixnet_client.split_sender().await;
        connection_monitor.start(
            mixnet_client_sender,
            self.assigned_addresses.mixnet_client_address,
            self.assigned_addresses.interface_addresses,
            self.assigned_addresses.exit_mix_addresses.0,
            &self.task_manager,
        );

        TunnelHandle {
            task_manager: self.task_manager,
            processor_handle,
            ipr_cancel_token: self.ipr_cancel_token,
            ipr_disconnect_rx: Some(ipr_disconnect_rx),
        }
    }
}

pub type ProcessorHandle = JoinHandle<Result<AsyncDevice, MixnetError>>;

/// Type providing a back channel for tunnel errors and a way to wait for tunnel to finish execution.
pub struct TunnelHandle {
    task_manager: TaskManager,
    processor_handle: ProcessorHandle,
    ipr_cancel_token: CancellationToken,
    ipr_disconnect_rx: Option<tokio::sync::oneshot::Receiver<()>>,
}

impl TunnelHandle {
    /// Cancel tunnel execution.
    pub async fn cancel(&mut self) {
        tracing::info!("TunnelHandle::cancel()");
        tracing::info!("self.ipr_cancel_token: {:?}", self.ipr_cancel_token);
        tracing::info!("Cancelling ipr_cancel_token");
        self.ipr_cancel_token.cancel();
        // Here we need to wait for ipr finish

        // tracing::info!("Sleeping for ipr to finish");
        // tokio::time::sleep(Duration::from_secs(2)).await;
        tracing::info!("Waiting for ipr_disconnect_rx");
        let ipr_disconnect_rx = self.ipr_disconnect_rx.take().unwrap();
        ipr_disconnect_rx.await.unwrap();

        if let Err(e) = self.task_manager.signal_shutdown() {
            tracing::error!("Failed to signal task manager shutdown: {}", e);
        }
    }

    /// Wait for the next error.
    ///
    /// This method is cancel safe.
    /// Returns `None` if the underlying channel has been closed.
    pub async fn recv_error(&mut self) -> Option<Box<dyn StdError + 'static + Send + Sync>> {
        self.task_manager.wait_for_error().await
    }

    /// Wait until the tunnel finished execution.
    pub async fn wait(mut self) -> Result<Result<Tombstone, MixnetError>, JoinError> {
        // First we need to wait for all the mixnet tasks to finish
        tracing::info!("TunnelHandle::wait()");
        tracing::info!("Waiting for graceful shutdown");
        self.task_manager.wait_for_graceful_shutdown().await;

        tracing::info!("Waiting for processor_handle");
        self.processor_handle
            .await
            .map(|result| result.map(Tombstone::with_tun_device))
    }
}
