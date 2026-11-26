// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
use std::{collections::HashSet, net::IpAddr};

#[cfg(target_os = "android")]
use hickory_resolver::config::ProtocolConfig;
use nym_vpn_lib_types::DnsServer;

#[cfg(not(target_os = "android"))]
use crate::tunnel_state_machine::{ResolverConfig, custom_dns::DnsServerExt};
use crate::{default_dns_servers::DefaultDnsServers, tunnel_state_machine::DnsOptions};

/// Resolves `DnsOptions` into DNS servers, falling back to default DNS servers.
pub struct DnsOptionsResolver<'a> {
    options: &'a DnsOptions,
    enable_ipv6: bool,
    default_dns_servers: &'a DefaultDnsServers,
}

impl<'a> DnsOptionsResolver<'a> {
    pub fn new(
        options: &'a DnsOptions,
        enable_ipv6: bool,
        default_dns_servers: &'a DefaultDnsServers,
    ) -> Self {
        Self {
            options,
            enable_ipv6,
            default_dns_servers,
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn resolver_config(&self) -> ResolverConfig {
        let mut config = match self.custom_dns_servers() {
            Some(dns_servers) => dns_servers
                .iter()
                .map(|ns| ns.to_name_server_config())
                .collect(),
            None => self.default_dns_servers.get_dns_servers(),
        };
        config.retain(|ns| ns.ip.is_ipv4() || (ns.ip.is_ipv6() && self.enable_ipv6));
        ResolverConfig(config)
    }

    /// Returns IP addresses of the DNS servers suitable for Android DNS configuration.
    ///
    /// If Private DNS is enabled, these IPs will be probed for DoT before falling back to UDP/TCP.
    #[cfg(target_os = "android")]
    pub fn android_tunnel_dns(&self) -> Vec<IpAddr> {
        let mut seen = HashSet::new();
        let mut addrs: Vec<IpAddr> = match self.custom_dns_servers() {
            Some(dns_servers) => dns_servers
                .iter()
                .filter(|dns_server| {
                    // Android only supports TCP/UDP 53 and TLS (853)
                    matches!(dns_server, DnsServer::Tls { .. } | DnsServer::Plain { .. })
                })
                .map(|dns_server| dns_server.address())
                .filter(|ip| seen.insert(*ip))
                .collect(),
            None => self
                .default_dns_servers
                .get_dns_servers()
                .iter()
                .filter(|ns| {
                    // Android only supports TCP/UDP 53 and TLS (853)
                    ns.connections.iter().any(|conn| {
                        matches!(
                            conn.protocol,
                            ProtocolConfig::Tls { .. } | ProtocolConfig::Udp | ProtocolConfig::Tcp
                        )
                    })
                })
                .map(|ns| ns.ip)
                .filter(|ip| seen.insert(*ip))
                .collect(),
        };
        addrs.retain(|v| v.is_ipv4() || (v.is_ipv6() && self.enable_ipv6));
        addrs
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub fn allowed_dns_endpoints(&self) -> Vec<nym_firewall::Endpoint> {
        match self.custom_dns_servers() {
            Some(dns_servers) => dns_servers
                .iter()
                .filter(|dns_server| {
                    let ip = dns_server.address();
                    ip.is_ipv4() || (ip.is_ipv6() && self.enable_ipv6)
                })
                .flat_map(|dns_server| dns_server.firewall_endpoints())
                .collect(),
            None => self.allowed_default_dns_endpoints(),
        }
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub fn allowed_default_dns_endpoints(&self) -> Vec<nym_firewall::Endpoint> {
        self.default_dns_servers
            .get_dns_servers()
            .iter()
            .filter(|ns| ns.ip.is_ipv4() || (ns.ip.is_ipv6() && self.enable_ipv6))
            .flat_map(|ns| {
                ns.connections.iter().map(|conn| {
                    let proto = if conn.protocol.to_protocol().is_datagram() {
                        nym_firewall::TransportProtocol::Udp
                    } else {
                        nym_firewall::TransportProtocol::Tcp
                    };
                    nym_firewall::Endpoint::new(ns.ip, conn.port, proto)
                })
            })
            .collect::<Vec<_>>()
    }

    /// Returns custom DNS servers, or `None` if default DNS servers should be used.
    fn custom_dns_servers(&self) -> Option<&'a [DnsServer]> {
        match self.options {
            DnsOptions::Custom(dns_servers) if !dns_servers.is_empty() => Some(dns_servers),
            DnsOptions::Default | DnsOptions::Custom(_) => None,
        }
    }
}
