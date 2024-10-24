// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{Error, Result};

use tun::AsyncDevice;

use super::{
    mixnet::connected_tunnel::TunnelHandle as MixnetTunnelHandle,
    wireguard::connected_tunnel::TunnelHandle as WireguardTunnelHandle,
};

pub enum AnyTunnelHandle {
    Mixnet(MixnetTunnelHandle),
    Wireguard(WireguardTunnelHandle),
}

impl From<MixnetTunnelHandle> for AnyTunnelHandle {
    fn from(value: MixnetTunnelHandle) -> Self {
        Self::Mixnet(value)
    }
}

impl From<WireguardTunnelHandle> for AnyTunnelHandle {
    fn from(value: WireguardTunnelHandle) -> Self {
        Self::Wireguard(value)
    }
}

impl AnyTunnelHandle {
    pub fn cancel(&mut self) {
        match self {
            Self::Mixnet(handle) => {
                handle.cancel();
            }
            Self::Wireguard(handle) => {
                handle.cancel();
            }
        }
    }

    pub async fn recv_error(
        &mut self,
    ) -> Option<Box<dyn std::error::Error + 'static + Send + Sync>> {
        match self {
            Self::Mixnet(handle) => handle.recv_error().await,
            Self::Wireguard(handle) => handle.recv_error().await,
        }
    }

    pub async fn wait(self) -> Result<Vec<AsyncDevice>> {
        match self {
            Self::Mixnet(handle) => match handle.wait().await {
                Ok(Ok(device)) => Ok(vec![device]),
                Ok(Err(e)) => Err(Error::MixnetClient(e)),
                Err(e) => {
                    tracing::error!("Failed to join on mixnet tunnel handle: {}", e);
                    Ok(vec![])
                }
            },
            Self::Wireguard(handle) => Ok(vec![handle.wait().await]),
        }
    }
}
