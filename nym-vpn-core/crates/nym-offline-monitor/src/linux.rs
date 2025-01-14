// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use futures::StreamExt;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

use super::Connectivity;
use nym_common::ErrorExt;
use nym_routing::RouteManagerHandle;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The route manager returned an error")]
    RouteManagerError(#[source] nym_routing::Error),
}

pub struct MonitorHandle {
    route_manager: RouteManagerHandle,
    fwmark: Option<u32>,
}

/// A non-local IPv4 address.
const PUBLIC_INTERNET_ADDRESS_V4: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
/// A non-local IPv6 address.
const PUBLIC_INTERNET_ADDRESS_V6: IpAddr = IpAddr::V6(Ipv6Addr::new(
    0x2606, 0x4700, 0x4700, 0x0, 0x0, 0x0, 0x0, 0x1111,
));

impl MonitorHandle {
    pub async fn connectivity(&self) -> Connectivity {
        check_connectivity(&self.route_manager, self.fwmark).await
    }
}

pub async fn spawn_monitor(
    notify_tx: watch::Sender<Connectivity>,
    route_manager: RouteManagerHandle,
    fwmark: Option<u32>,
    shutdown_token: CancellationToken,
) -> Result<MonitorHandle> {
    let mut listener = route_manager
        .change_listener()
        .await
        .map_err(Error::RouteManagerError)?;

    let mut connectivity = check_connectivity(&route_manager, fwmark).await;

    let monitor_handle = MonitorHandle {
        route_manager: route_manager.clone(),
        fwmark,
    };

    tokio::spawn(async move {
        loop {
            tokio::select! {
                event = listener.next() => {
                    if event.is_none() {
                        break;
                    }
                    let new_connectivity = check_connectivity(&route_manager, fwmark).await;
                    if new_connectivity != connectivity {
                        connectivity = new_connectivity;
                        if notify_tx.send(connectivity).is_err() {
                            break;
                        }
                    }
                },
                _ = shutdown_token.cancelled() => {
                    break;
                }
            }
        }

        tracing::debug!("Offline monitor exiting");
    });

    Ok(monitor_handle)
}

async fn check_connectivity(handle: &RouteManagerHandle, fwmark: Option<u32>) -> Connectivity {
    let route_exists = |destination| async move {
        handle
            .get_destination_route(destination, fwmark)
            .await
            .map(|route| route.is_some())
    };

    match (
        route_exists(PUBLIC_INTERNET_ADDRESS_V4).await,
        route_exists(PUBLIC_INTERNET_ADDRESS_V6).await,
    ) {
        (Ok(ipv4), Ok(ipv6)) => Connectivity::Status { ipv4, ipv6 },
        // If we fail to retrieve the IPv4 route, always assume we're connected
        (Err(err), _) => {
            tracing::error!(
                "Failed to verify offline state: {}. Presuming connectivity",
                err
            );
            Connectivity::PresumeOnline
        }
        // Errors for IPv6 likely mean it's disabled, so assume it's unavailable
        (Ok(ipv4), Err(err)) => {
            tracing::trace!(
                "{}",
                err.display_chain_with_msg(
                    "Failed to infer offline state for IPv6. Assuming it's unavailable"
                )
            );
            Connectivity::Status { ipv4, ipv6: false }
        }
    }
}
