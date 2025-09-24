// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::SocketAddr;

/// Default websocket port used as a fallback
const DEFAULT_WS_PORT: u16 = 80;

pub trait GatewayExt {
    /// Returns a list of all endpoints with WSS port if available, otherwise WS port.
    fn endpoints(&self) -> Vec<SocketAddr>;
}

impl GatewayExt for nym_gateway_directory::Gateway {
    fn endpoints(&self) -> Vec<SocketAddr> {
        let ws_port = self
            .clients_wss_port
            .or(self.clients_ws_port)
            .unwrap_or(DEFAULT_WS_PORT);

        self.ips
            .iter()
            .map(|ip| SocketAddr::new(*ip, ws_port))
            .collect::<Vec<_>>()
    }
}
