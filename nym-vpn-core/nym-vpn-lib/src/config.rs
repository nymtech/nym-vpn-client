// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    error::*,
    wg_gateway_client::{GatewayData, WgGatewayClient},
};
use nym_crypto::asymmetric::x25519::KeyPair;
use nym_gateway_directory::{GatewayClient, NodeIdentity};
use std::{
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};
use talpid_types::net::{
    wireguard::{ConnectionConfig, PeerConfig, PrivateKey, TunnelConfig, TunnelOptions},
    GenericTunnelOptions,
};

#[cfg(target_os = "linux")]
pub const TUNNEL_FWMARK: u32 = 0x6d6f6c65;

#[derive(Clone)]
pub struct WireguardConfig {
    pub talpid_config: talpid_wireguard::config::Config,
    pub gateway_data: GatewayData,
    pub gateway_id: NodeIdentity,
}

impl WireguardConfig {
    fn new(
        tunnel: TunnelConfig,
        peers: Vec<PeerConfig>,
        connection_config: &ConnectionConfig,
        wg_options: &TunnelOptions,
        generic_options: &GenericTunnelOptions,
        gateway_data: GatewayData,
        gateway_id: NodeIdentity,
    ) -> Result<Self> {
        Ok(Self {
            talpid_config: talpid_wireguard::config::Config::new(
                tunnel,
                peers,
                connection_config,
                wg_options,
                generic_options,
                None,
            )?,
            gateway_data,
            gateway_id,
        })
    }

    pub fn init(
        keypair: &KeyPair,
        gateway_data: GatewayData,
        wg_gateway: Option<IpAddr>,
        gateway_id: NodeIdentity,
        mtu: u16,
    ) -> Result<Self> {
        let tunnel = TunnelConfig {
            private_key: PrivateKey::from(keypair.private_key().to_bytes()),
            addresses: vec![gateway_data.private_ipv4.into()],
        };
        let peers = vec![PeerConfig {
            public_key: gateway_data.public_key.clone(),
            allowed_ips: vec![IpAddr::from(gateway_data.private_ipv4).into()],
            endpoint: gateway_data.endpoint,
            psk: None,
        }];
        let default_ipv4_gateway = Ipv4Addr::from_str(&gateway_data.private_ipv4.to_string())?;
        let (ipv4_gateway, ipv6_gateway) = match wg_gateway {
            Some(IpAddr::V4(ipv4_gateway)) => (ipv4_gateway, None),
            Some(IpAddr::V6(ipv6_gateway)) => (default_ipv4_gateway, Some(ipv6_gateway)),
            None => (default_ipv4_gateway, None),
        };
        let connection_config = ConnectionConfig {
            tunnel: tunnel.clone(),
            peer: peers[0].clone(),
            exit_peer: None,
            ipv4_gateway,
            ipv6_gateway,
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
            gateway_data,
            gateway_id,
        )?;
        Ok(config)
    }
}

impl std::fmt::Display for WireguardConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "tunnel:")?;
        writeln!(f, "  mtu: {}", self.talpid_config.mtu)?;
        #[cfg(target_os = "linux")]
        writeln!(f, "  enable_ipv6: {}", self.talpid_config.enable_ipv6)?;
        writeln!(f, "  addresses:")?;
        for address in &self.talpid_config.tunnel.addresses {
            writeln!(f, "    - {}", address)?;
        }
        writeln!(f, "peers:")?;
        for peer in &self.talpid_config.peers {
            writeln!(f, "  - public_key: {}", peer.public_key)?;
            writeln!(f, "    allowed_ips:")?;
            for allowed_ip in &peer.allowed_ips {
                writeln!(f, "      - {}", allowed_ip)?;
            }
            writeln!(f, "    endpoint: {}", peer.endpoint)?;
        }
        writeln!(f, "connection:")?;
        writeln!(f, "  ipv4_gateway: {}", self.talpid_config.ipv4_gateway)?;
        if let Some(ipv6_gateway) = &self.talpid_config.ipv6_gateway {
            writeln!(f, "  ipv6_gateway: {}", ipv6_gateway)?;
        }
        #[cfg(target_os = "linux")]
        if let Some(fwmark) = &self.talpid_config.fwmark {
            writeln!(f, "  fwmark: {}", fwmark)?;
        }
        Ok(())
    }
}

pub(crate) async fn init_wireguard_config(
    gateway_client: &GatewayClient,
    wg_gateway_client: &mut WgGatewayClient,
    wg_gateway: Option<IpAddr>,
    mtu: u16,
) -> Result<(WireguardConfig, IpAddr)> {
    // First we need to register with the gateway to setup keys and IP assignment
    tracing::info!("Registering with wireguard gateway");
    let gateway_id = wg_gateway_client
        .auth_recipient()
        .gateway()
        .to_base58_string();
    let gateway_host = gateway_client
        .lookup_gateway_ip(&gateway_id)
        .await
        .map_err(|source| GatewayDirectoryError::FailedToLookupGatewayIp { gateway_id, source })?;
    let wg_gateway_data = wg_gateway_client.register_wireguard(gateway_host).await?;
    tracing::debug!("Received wireguard gateway data: {wg_gateway_data:?}");

    let wireguard_config = WireguardConfig::init(
        wg_gateway_client.keypair(),
        wg_gateway_data,
        wg_gateway,
        *wg_gateway_client.auth_recipient().gateway(),
        mtu,
    )?;
    Ok((wireguard_config, gateway_host))
}
