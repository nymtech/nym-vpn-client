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
        let mut ports: Vec<u16> = self
            .clients_ws_port
            .into_iter()
            .chain(self.clients_wss_port)
            .collect();
        // if neither WS nor WSS is there, default port
        if ports.is_empty() {
            ports = vec![DEFAULT_WS_PORT];
        }

        itertools::iproduct!(self.ips.clone(), ports)
            .map(|(ip, port)| SocketAddr::new(ip, port))
            .collect::<Vec<_>>()
    }
}
