// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod any_tunnel_handle;
mod gateway_selector;
pub mod mixnet;
mod tombstone;
pub mod transports;
pub mod wireguard;

pub use gateway_selector::SelectedGateways;
use nym_gateway_directory::GatewayCacheHandle;
use nym_vpn_api_client::types::ScoreThresholds;
use tokio_util::sync::CancellationToken;

#[cfg(windows)]
use super::route_handler;
use crate::{GatewayDirectoryError, MixnetError, tunnel_state_machine::TunnelSettings};
pub use any_tunnel_handle::AnyTunnelHandle;
pub use tombstone::Tombstone;

pub async fn select_gateways(
    gateway_cache_handle: GatewayCacheHandle,
    tunnel_settings: &TunnelSettings,
    wg_score_thresholds: Option<ScoreThresholds>,
    mix_score_thresholds: Option<ScoreThresholds>,
    cancel_token: CancellationToken,
) -> Result<SelectedGateways> {
    let select_gateways_fut = gateway_selector::select_gateways(
        gateway_cache_handle,
        tunnel_settings,
        wg_score_thresholds,
        mix_score_thresholds,
    );

    cancel_token
        .run_until_cancelled(select_gateways_fut)
        .await
        .ok_or(Error::Cancelled)?
        .map_err(|err| Error::SelectGateways(Box::new(err)))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to select gateways")]
    SelectGateways(#[source] Box<GatewayDirectoryError>),

    #[error("mixnet tunnel has failed")]
    MixnetClient(#[from] MixnetError),

    #[error("{} has no ip addresses announced", gateway_id)]
    NoIpAddressAnnounced { gateway_id: String },

    #[error("bandwidth controller error")]
    BandwidthController(#[from] crate::bandwidth_controller::Error),

    #[error("registration client error")]
    RegistrationClient(#[source] Box<nym_registration_client::RegistrationClientError>),

    #[cfg(target_os = "ios")]
    #[error("failed to resolve using dns64")]
    ResolveDns64(#[from] wireguard::dns64::Error),

    #[error("WireGuard error")]
    Wireguard(#[from] nym_wg_go::Error),

    #[error("failed to dup tunnel file descriptor")]
    DupFd(#[source] std::io::Error),

    #[cfg(windows)]
    #[error("failed to add default route listener")]
    AddDefaultRouteListener(#[source] route_handler::Error),

    #[error("transport error")]
    Transport(#[from] transports::TransportError),

    #[error("connection cancelled")]
    Cancelled,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
