// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_registration_common::AssignedAddresses;
use nym_sdk::mixnet::MixnetClient;
use tokio::task::{JoinError, JoinHandle};
use tokio_util::sync::CancellationToken;
use tun::AsyncDevice;

use nym_connection_monitor::ConnectionMonitorTask;

use crate::{
    mixnet::{MixnetError, MixnetProcessorConfig},
    tunnel_state_machine::tunnel::{Result, Tombstone},
};

/// Type representing a connected mixnet tunnel.
pub struct ConnectedTunnel {
    mixnet_client: MixnetClient,
    assigned_addresses: AssignedAddresses,
    cancel_token: CancellationToken,
}

impl ConnectedTunnel {
    pub fn new(
        mixnet_client: MixnetClient,
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

    pub async fn run(self, tun_device: AsyncDevice) -> Result<TunnelHandle> {
        let connection_monitor = ConnectionMonitorTask::setup();

        let processor_config = MixnetProcessorConfig::new(
            self.assigned_addresses.exit_mix_address.into(),
            self.assigned_addresses.interface_addresses,
        );

        let mixnet_client_sender = self.mixnet_client.split_sender();
        let mixnet_cancellation_token = self.mixnet_client.cancellation_token().clone();

        let processor_handle = crate::mixnet::start_processor(
            processor_config,
            tun_device,
            self.mixnet_client,
            &connection_monitor,
            self.cancel_token.clone(),
        )
        .await;

        connection_monitor.start(
            mixnet_client_sender,
            self.assigned_addresses.mixnet_client_address,
            // todo: not fully possible to disable IPv6 because IpPair is passed.
            self.assigned_addresses.interface_addresses,
            self.assigned_addresses.exit_mix_address,
            self.cancel_token.clone(),
        );

        Ok(TunnelHandle {
            processor_handle,
            cancel_token: self.cancel_token,
            mixnet_cancellation_token,
        })
    }
}

pub type ProcessorHandle = JoinHandle<Result<AsyncDevice, MixnetError>>;

/// Type providing a back channel for tunnel errors and a way to wait for tunnel to finish execution.
pub struct TunnelHandle {
    processor_handle: ProcessorHandle,
    cancel_token: CancellationToken,
    mixnet_cancellation_token: CancellationToken,
}

impl TunnelHandle {
    /// Cancel tunnel execution.
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    pub fn mixnet_cancel_token(&self) -> CancellationToken {
        self.mixnet_cancellation_token.clone()
    }

    /// Wait until the tunnel finished execution.
    pub async fn wait(self) -> Result<Result<Tombstone, MixnetError>, JoinError> {
        tracing::trace!("Waiting for mixnet processor handle");
        self.processor_handle
            .await
            .map(|result| result.map(Tombstone::with_tun_device))
    }
}
