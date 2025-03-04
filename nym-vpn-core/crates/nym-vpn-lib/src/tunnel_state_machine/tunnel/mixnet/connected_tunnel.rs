// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error as StdError;

use nym_task::TaskManager;
use tokio::task::{JoinError, JoinHandle};
use tun::AsyncDevice;

use nym_connection_monitor::ConnectionMonitorTask;
use nym_mixnet_client::SharedMixnetClient;

use super::connector::AssignedAddresses;
use crate::{
    mixnet::{MixnetError, MixnetProcessorConfig},
    tunnel_state_machine::tunnel::Tombstone,
};

/// Type representing a connected mixnet tunnel.
pub struct ConnectedTunnel {
    task_manager: TaskManager,
    mixnet_client: SharedMixnetClient,
    assigned_addresses: AssignedAddresses,
}

impl ConnectedTunnel {
    pub fn new(
        task_manager: TaskManager,
        mixnet_client: SharedMixnetClient,
        assigned_addresses: AssignedAddresses,
    ) -> Self {
        Self {
            task_manager,
            mixnet_client,
            assigned_addresses,
        }
    }

    pub fn assigned_addresses(&self) -> &AssignedAddresses {
        &self.assigned_addresses
    }

    pub async fn run(self, tun_device: AsyncDevice) -> TunnelHandle {
        let connection_monitor = ConnectionMonitorTask::setup();

        let processor_config =
            MixnetProcessorConfig::new(self.assigned_addresses.exit_mix_addresses);
        let processor_handle = crate::mixnet::start_processor(
            processor_config,
            tun_device,
            self.mixnet_client.clone(),
            &self.task_manager,
            self.assigned_addresses.interface_addresses,
            &connection_monitor,
        )
        .await;

        let mixnet_client_sender = self.mixnet_client.split_sender().await;
        connection_monitor.start(
            mixnet_client_sender,
            self.assigned_addresses.mixnet_client_address,
            self.assigned_addresses.interface_addresses,
            self.assigned_addresses.exit_mix_addresses.into(),
            &self.task_manager,
        );

        TunnelHandle {
            task_manager: self.task_manager,
            processor_handle,
        }
    }
}

pub type ProcessorHandle = JoinHandle<Result<AsyncDevice, MixnetError>>;

/// Type providing a back channel for tunnel errors and a way to wait for tunnel to finish execution.
pub struct TunnelHandle {
    task_manager: TaskManager,
    processor_handle: ProcessorHandle,
}

impl TunnelHandle {
    /// Cancel tunnel execution.
    pub fn cancel(&self) {
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
    pub async fn wait(self) -> Result<Result<Tombstone, MixnetError>, JoinError> {
        self.processor_handle
            .await
            .map(|result| result.map(Tombstone::with_tun_device))
    }
}
