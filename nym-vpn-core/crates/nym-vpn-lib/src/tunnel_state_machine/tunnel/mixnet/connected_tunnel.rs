// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_task::TaskManager;
use tokio::{
    sync::oneshot,
    task::{JoinError, JoinHandle},
};
use tokio_util::sync::CancellationToken;
use tun::AsyncDevice;

use nym_connection_monitor::ConnectionMonitorTask;

use super::connector::AssignedAddresses;
use crate::{
    mixnet::{MixnetError, MixnetProcessorConfig, SharedMixnetClient},
    tunnel_state_machine::tunnel::{Error, Result, Tombstone},
};

/// Type representing a connected mixnet tunnel.
pub struct ConnectedTunnel {
    mixnet_client: SharedMixnetClient,
    assigned_addresses: AssignedAddresses,
    cancel_token: CancellationToken,
}

impl ConnectedTunnel {
    pub fn new(
        mixnet_client: SharedMixnetClient,
        assigned_addresses: AssignedAddresses,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            mixnet_client,
            assigned_addresses,
            cancel_token,
        }
    }

    pub fn assigned_addresses(&self) -> &AssignedAddresses {
        &self.assigned_addresses
    }

    pub async fn run(
        self,
        task_manager: &TaskManager,
        tun_device: AsyncDevice,
    ) -> Result<TunnelHandle> {
        let connection_monitor = ConnectionMonitorTask::setup();

        let processor_config = MixnetProcessorConfig::new(
            self.assigned_addresses.exit_mix_addresses,
            self.assigned_addresses.interface_addresses,
        );

        let (ipr_disconnect_tx, ipr_disconnect_rx) = oneshot::channel();

        let processor_handle = crate::mixnet::start_processor(
            processor_config,
            tun_device,
            self.mixnet_client.clone(),
            task_manager,
            &connection_monitor,
            self.cancel_token.clone(),
            ipr_disconnect_tx,
        )
        .await;

        let mixnet_client_sender = self
            .mixnet_client
            .lock()
            .await
            .as_ref()
            .ok_or(Error::MixnetClientDisposed)?
            .split_sender();
        connection_monitor.start(
            mixnet_client_sender,
            self.assigned_addresses.mixnet_client_address,
            // todo: not fully possible to disable IPv6 because IpPair is passed.
            self.assigned_addresses.interface_addresses,
            self.assigned_addresses.exit_mix_addresses.into(),
            task_manager,
        );

        Ok(TunnelHandle {
            processor_handle,
            processor_disconnected: Some(ipr_disconnect_rx),
        })
    }
}

pub type ProcessorHandle = JoinHandle<Result<AsyncDevice, MixnetError>>;

/// Type providing a back channel for tunnel errors and a way to wait for tunnel to finish execution.
pub struct TunnelHandle {
    processor_handle: ProcessorHandle,
    processor_disconnected: Option<oneshot::Receiver<()>>,
}

impl TunnelHandle {
    /// Cancel tunnel execution.
    pub async fn cancel(&mut self) {
        self.wait_for_processor_disconnect().await;
    }

    async fn wait_for_processor_disconnect(&mut self) {
        tracing::info!("Waiting for the mixnet processor to disconnect");
        if let Some(processor_disconnected) = self.processor_disconnected.take() {
            tokio::time::timeout(Duration::from_secs(10), processor_disconnected)
                .await
                .unwrap_or_else(|_| {
                    tracing::error!(
                        "Timed out waiting for processor to disconnect. Forcing shutdown."
                    );
                    Ok(())
                })
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to wait for processor to disconnect: {e}");
                });
        } else {
            tracing::error!("Processor has already disconnected");
        }
    }

    /// Wait until the tunnel finished execution.
    pub async fn wait(self) -> Result<Result<Tombstone, MixnetError>, JoinError> {
        tracing::trace!("Waiting for mixnet processor handle");
        self.processor_handle
            .await
            .map(|result| result.map(Tombstone::with_tun_device))
    }
}
