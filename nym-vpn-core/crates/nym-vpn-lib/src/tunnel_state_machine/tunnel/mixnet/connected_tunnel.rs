// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::task::{JoinError, JoinHandle};
use tokio_util::sync::CancellationToken;
use tun::AsyncDevice;

use nym_common::trace_err_chain;
use nym_gateway_directory::IpPacketRouterAddress;
use nym_registration_common::AssignedAddresses;
use nym_sdk::mixnet::{EventReceiver, MixnetClient};

use crate::{
    mixnet::MixnetError,
    tunnel_state_machine::tunnel::{Result, Tombstone},
};

pub async fn start_mixnet_tunnel(
    mixnet_client: MixnetClient,
    assigned_addresses: AssignedAddresses,
    tun_device: AsyncDevice,
    cancel_token: CancellationToken,
    event_rx: EventReceiver,
) -> Result<TunnelHandle> {
    let processor_handle = crate::mixnet::start_processor(
        IpPacketRouterAddress::from(assigned_addresses.exit_mix_address),
        tun_device,
        mixnet_client,
        cancel_token.clone(),
        event_rx,
    )
    .await;

    Ok(TunnelHandle {
        processor_handle,
        cancel_token,
    })
}

pub type ProcessorHandle = JoinHandle<Result<AsyncDevice, MixnetError>>;

/// Type providing a back channel for tunnel errors and a way to wait for tunnel to finish execution.
pub struct TunnelHandle {
    processor_handle: ProcessorHandle,
    cancel_token: CancellationToken,
}

impl TunnelHandle {
    /// Cancel tunnel execution.
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Wait until the tunnel finished execution.
    pub async fn wait(self) -> Result<Result<Tombstone, MixnetError>, JoinError> {
        tracing::trace!("Waiting for mixnet processor handle");
        self.processor_handle
            .await
            .inspect_err(|err| trace_err_chain!(err, "mixnet processor exited with error"))
            .map(|result| result.map(Tombstone::with_tun_device))
    }
}
