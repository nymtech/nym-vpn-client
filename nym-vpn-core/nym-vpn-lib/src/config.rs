// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::*;
use crate::wg_gateway_client::GatewayData;
use nym_crypto::asymmetric::x25519::KeyPair;
use std::net::Ipv4Addr;
use std::str::FromStr;
use talpid_types::net::wireguard::{
    ConnectionConfig, PeerConfig, PrivateKey, TunnelConfig, TunnelOptions,
};
use talpid_types::net::GenericTunnelOptions;

#[cfg(target_os = "linux")]
pub const TUNNEL_FWMARK: u32 = 0x6d6f6c65;
#[cfg(target_os = "linux")]
pub const TUNNEL_TABLE_ID: u32 = 0x6d6f6c65;

#[derive(Clone)]
pub struct WireguardConfig(pub talpid_wireguard::config::Config);

impl WireguardConfig {
    fn new(
        tunnel: TunnelConfig,
        peers: Vec<PeerConfig>,
        connection_config: &ConnectionConfig,
        wg_options: &TunnelOptions,
        generic_options: &GenericTunnelOptions,
    ) -> Result<Self> {
        Ok(Self(talpid_wireguard::config::Config::new(
            tunnel,
            peers,
            connection_config,
            wg_options,
            generic_options,
            None,
        )?))
    }

    pub fn init(keypair: &KeyPair, gateway_data: &GatewayData, mtu: u16) -> Result<Self> {
        let tunnel = TunnelConfig {
            private_key: PrivateKey::from(keypair.private_key().to_bytes()),
            addresses: vec![gateway_data.private_ip],
        };
        let peers = vec![PeerConfig {
            public_key: gateway_data.public_key.clone(),
            allowed_ips: vec![gateway_data.private_ip.into()],
            endpoint: gateway_data.endpoint,
            psk: None,
        }];
        let connection_config = ConnectionConfig {
            tunnel: tunnel.clone(),
            peer: peers[0].clone(),
            exit_peer: None,
            ipv4_gateway: Ipv4Addr::from_str(&gateway_data.private_ip.to_string())?,
            ipv6_gateway: None,
            #[cfg(target_os = "linux")]
            fwmark: Some(TUNNEL_FWMARK),
        };
        let generic_options = GenericTunnelOptions { enable_ipv6: false };
        let wg_options = TunnelOptions {
            mtu: Some(mtu),
            ..Default::default()
        };
        let config = Self::new(
            tunnel,
            peers,
            &connection_config,
            &wg_options,
            &generic_options,
        )?;
        Ok(config)
    }
}

impl std::fmt::Display for WireguardConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "tunnel:")?;
        writeln!(f, "  mtu: {}", self.0.mtu)?;
        #[cfg(target_os = "linux")]
        writeln!(f, "  enable_ipv6: {}", self.0.enable_ipv6)?;
        writeln!(f, "  addresses:")?;
        for address in &self.0.tunnel.addresses {
            writeln!(f, "    - {}", address)?;
        }
        writeln!(f, "peers:")?;
        for peer in &self.0.peers {
            writeln!(f, "  - public_key: {}", peer.public_key)?;
            writeln!(f, "    allowed_ips:")?;
            for allowed_ip in &peer.allowed_ips {
                writeln!(f, "      - {}", allowed_ip)?;
            }
            writeln!(f, "    endpoint: {}", peer.endpoint)?;
        }
        writeln!(f, "connection:")?;
        writeln!(f, "  ipv4_gateway: {}", self.0.ipv4_gateway)?;
        if let Some(ipv6_gateway) = &self.0.ipv6_gateway {
            writeln!(f, "  ipv6_gateway: {}", ipv6_gateway)?;
        }
        #[cfg(target_os = "linux")]
        if let Some(fwmark) = &self.0.fwmark {
            writeln!(f, "  fwmark: {}", fwmark)?;
        }
        Ok(())
    }
}
