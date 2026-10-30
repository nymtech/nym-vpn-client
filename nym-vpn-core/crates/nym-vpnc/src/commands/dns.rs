// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, Ipv4Addr};

use anyhow::{Result, anyhow, bail};
use nym_vpn_lib_types::{CustomDnsSettings, DnsServer, PlainDnsProtocol};
use nym_vpn_proto::rpc_client::RpcClient;
use url::{Host, Url};

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get custom DNS servers
    Get,

    /// Enable custom DNS
    Enable,

    /// Disable custom DNS
    Disable,

    /// Set custom DNS servers (space separated)
    ///
    /// Each server is either a bare IP address (plain DNS over UDP and TCP) or a URL:
    ///
    ///   plain://IP[:PORT]                plain DNS over UDP and TCP
    ///   udp://IP[:PORT]                  plain DNS over UDP
    ///   tcp://IP[:PORT]                  plain DNS over TCP
    ///   tls://IP[:PORT]#NAME             DNS over TLS
    ///   https://IP[:PORT][/PATH]#NAME    DNS over HTTPS
    ///   quic://IP[:PORT]#NAME            DNS over QUIC
    ///   h3://IP[:PORT][/PATH]#NAME       DNS over HTTP/3
    ///
    /// NAME is the TLS server name. IPv6 addresses in URLs go in brackets, e.g. tls://[2606:4700:4700::1111]#cloudflare-dns.com
    #[command(verbatim_doc_comment)]
    Set {
        #[arg(value_parser = parse_dns_server)]
        dns_servers: Vec<DnsServer>,
    },

    /// Clear custom DNS servers
    Clear,

    /// Get default DNS servers
    GetDefault,
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let config = rpc_client.get_config().await?;
                println!(
                    "Custom DNS: {} [{}]",
                    if config.enable_custom_dns {
                        "Enabled"
                    } else {
                        "Disabled"
                    },
                    config
                        .custom_dns
                        .dns_servers
                        .iter()
                        .map(format_dns_server)
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                Ok(())
            }
            Command::Enable => {
                rpc_client.set_enable_custom_dns(true).await?;
                Ok(())
            }
            Command::Disable => {
                rpc_client.set_enable_custom_dns(false).await?;
                Ok(())
            }
            Command::Set { dns_servers } => {
                rpc_client
                    .set_custom_dns(CustomDnsSettings { dns_servers })
                    .await?;
                Ok(())
            }
            Command::Clear => {
                // You can also use `Set`, and specify no servers, but this is clearer.
                rpc_client
                    .set_custom_dns(CustomDnsSettings::default())
                    .await?;
                Ok(())
            }
            Command::GetDefault => {
                let default_dns = rpc_client.get_default_dns().await?;
                println!("Default DNS:");
                for dns_server in default_dns {
                    println!("{}", format_dns_server(&dns_server))
                }
                Ok(())
            }
        }
    }
}

fn parse_dns_server(input: &str) -> Result<DnsServer> {
    if let Ok(address) = input.parse::<IpAddr>() {
        return Ok(DnsServer::Plain {
            address,
            port: None,
            ip_proto: PlainDnsProtocol::UdpAndTcp,
        });
    }

    let url = Url::parse(input).map_err(|e| anyhow!("Invalid DNS server '{input}': {e}"))?;

    if !url.username().is_empty() || url.password().is_some() {
        bail!("DNS server URL '{input}' must not contain credentials");
    }
    if url.query().is_some() {
        bail!("DNS server URL '{input}' must not contain a query");
    }

    let address = url_address(&url)
        .ok_or_else(|| anyhow!("DNS server URL '{input}' must use an IP address as the host"))?;
    let port = url.port();

    match url.scheme() {
        "plain" | "udp" | "tcp" => {
            if url.fragment().is_some() {
                bail!("Plain DNS server URL '{input}' must not contain a server name");
            }
            if url_query_path(&url).is_some() {
                bail!("Plain DNS server URL '{input}' must not contain a path");
            }
            let ip_proto = match url.scheme() {
                "udp" => PlainDnsProtocol::Udp,
                "tcp" => PlainDnsProtocol::Tcp,
                _ => PlainDnsProtocol::UdpAndTcp,
            };
            Ok(DnsServer::Plain {
                address,
                port,
                ip_proto,
            })
        }
        "tls" | "quic" => {
            if url_query_path(&url).is_some() {
                bail!("DNS server URL '{input}' must not contain a path");
            }
            let server_name = url_server_name(&url, input)?;
            Ok(if url.scheme() == "tls" {
                DnsServer::Tls {
                    address,
                    port,
                    server_name,
                }
            } else {
                DnsServer::Quic {
                    address,
                    port,
                    server_name,
                }
            })
        }
        "https" | "h3" => {
            let server_name = url_server_name(&url, input)?;
            let query_path = url_query_path(&url);
            Ok(if url.scheme() == "https" {
                DnsServer::Https {
                    address,
                    port,
                    server_name,
                    query_path,
                }
            } else {
                DnsServer::Http3 {
                    address,
                    port,
                    server_name,
                    query_path,
                }
            })
        }
        scheme => Err(anyhow!(
            "Unsupported DNS server scheme '{scheme}' in '{input}', expected one of: plain, udp, tcp, tls, https, quic, h3"
        )),
    }
}

fn format_dns_server(server: &DnsServer) -> String {
    match server {
        DnsServer::Plain {
            address,
            port: None,
            ip_proto: PlainDnsProtocol::UdpAndTcp,
        } => address.to_string(),
        DnsServer::Plain {
            address,
            port,
            ip_proto,
        } => {
            let scheme = match ip_proto {
                PlainDnsProtocol::Udp => "udp",
                PlainDnsProtocol::Tcp => "tcp",
                PlainDnsProtocol::UdpAndTcp => "plain",
            };
            format!("{scheme}://{}", format_host_port(*address, *port))
        }
        DnsServer::Tls {
            address,
            port,
            server_name,
        } => format!("tls://{}#{server_name}", format_host_port(*address, *port)),
        DnsServer::Quic {
            address,
            port,
            server_name,
        } => format!("quic://{}#{server_name}", format_host_port(*address, *port)),
        DnsServer::Https {
            address,
            port,
            server_name,
            query_path,
        } => format!(
            "https://{}{}#{server_name}",
            format_host_port(*address, *port),
            format_query_path(query_path.as_deref())
        ),
        DnsServer::Http3 {
            address,
            port,
            server_name,
            query_path,
        } => format!(
            "h3://{}{}#{server_name}",
            format_host_port(*address, *port),
            format_query_path(query_path.as_deref())
        ),
    }
}

fn url_address(url: &Url) -> Option<IpAddr> {
    match url.host()? {
        Host::Ipv4(address) => Some(IpAddr::V4(address)),
        Host::Ipv6(address) => Some(IpAddr::V6(address)),
        // Non-special schemes (tls, quic, ...) keep IPv4 hosts as opaque strings.
        Host::Domain(domain) => domain.parse::<Ipv4Addr>().ok().map(IpAddr::V4),
    }
}

fn url_server_name(url: &Url, input: &str) -> Result<String> {
    match url.fragment() {
        Some(name) if !name.is_empty() => Ok(name.to_owned()),
        _ => Err(anyhow!(
            "DNS server URL '{input}' must end with the TLS server name, e.g. #dns.example.com"
        )),
    }
}

fn url_query_path(url: &Url) -> Option<String> {
    match url.path() {
        "" | "/" => None,
        path => Some(path.to_owned()),
    }
}

fn format_host_port(address: IpAddr, port: Option<u16>) -> String {
    let host = match address {
        IpAddr::V4(address) => address.to_string(),
        IpAddr::V6(address) => format!("[{address}]"),
    };
    match port {
        Some(port) => format!("{host}:{port}"),
        None => host,
    }
}

fn format_query_path(query_path: Option<&str>) -> String {
    match query_path {
        Some(path) if path.starts_with('/') => path.to_owned(),
        Some(path) => format!("/{path}"),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> DnsServer {
        parse_dns_server(input).unwrap()
    }

    #[test]
    fn parses_bare_ip_as_plain_udp_and_tcp() {
        assert_eq!(
            parse("1.1.1.1"),
            DnsServer::Plain {
                address: "1.1.1.1".parse().unwrap(),
                port: None,
                ip_proto: PlainDnsProtocol::UdpAndTcp,
            }
        );
        assert_eq!(
            parse("2606:4700:4700::1111"),
            DnsServer::Plain {
                address: "2606:4700:4700::1111".parse().unwrap(),
                port: None,
                ip_proto: PlainDnsProtocol::UdpAndTcp,
            }
        );
    }

    #[test]
    fn parses_plain_schemes() {
        assert_eq!(
            parse("tcp://9.9.9.9:5353"),
            DnsServer::Plain {
                address: "9.9.9.9".parse().unwrap(),
                port: Some(5353),
                ip_proto: PlainDnsProtocol::Tcp,
            }
        );
        assert_eq!(
            parse("udp://[::1]"),
            DnsServer::Plain {
                address: "::1".parse().unwrap(),
                port: None,
                ip_proto: PlainDnsProtocol::Udp,
            }
        );
    }

    #[test]
    fn parses_encrypted_schemes() {
        assert_eq!(
            parse("tls://1.1.1.1:853#cloudflare-dns.com"),
            DnsServer::Tls {
                address: "1.1.1.1".parse().unwrap(),
                port: Some(853),
                server_name: "cloudflare-dns.com".to_owned(),
            }
        );
        assert_eq!(
            parse("https://8.8.8.8/dns-query#dns.google"),
            DnsServer::Https {
                address: "8.8.8.8".parse().unwrap(),
                port: None,
                server_name: "dns.google".to_owned(),
                query_path: Some("/dns-query".to_owned()),
            }
        );
        assert_eq!(
            parse("quic://94.140.14.14#dns.adguard-dns.com"),
            DnsServer::Quic {
                address: "94.140.14.14".parse().unwrap(),
                port: None,
                server_name: "dns.adguard-dns.com".to_owned(),
            }
        );
        assert_eq!(
            parse("h3://[2606:4700:4700::1111]:8443#cloudflare-dns.com"),
            DnsServer::Http3 {
                address: "2606:4700:4700::1111".parse().unwrap(),
                port: Some(8443),
                server_name: "cloudflare-dns.com".to_owned(),
                query_path: None,
            }
        );
    }

    #[test]
    fn rejects_invalid_input() {
        for input in [
            "not-an-ip",
            "tls://1.1.1.1",
            "tls://1.1.1.1#",
            "tls://cloudflare-dns.com#cloudflare-dns.com",
            "https://dns.google/dns-query#dns.google",
            "udp://1.1.1.1#name",
            "tcp://1.1.1.1/path",
            "quic://1.1.1.1/path#name",
            "https://8.8.8.8/dns-query?x=1#dns.google",
            "https://user@8.8.8.8#dns.google",
            "ftp://1.1.1.1",
            "tls://1.1.1.1:70000#name",
        ] {
            assert!(
                parse_dns_server(input).is_err(),
                "{input} should be rejected"
            );
        }
    }

    #[test]
    fn format_round_trips() {
        for input in [
            "1.1.1.1",
            "2606:4700:4700::1111",
            "plain://1.1.1.1:5353",
            "udp://9.9.9.9",
            "tcp://[::1]:53",
            "tls://1.1.1.1:853#cloudflare-dns.com",
            "https://8.8.8.8/dns-query#dns.google",
            "https://8.8.8.8:8443#dns.google",
            "quic://94.140.14.14#dns.adguard-dns.com",
            "h3://[2606:4700:4700::1111]/dns-query#cloudflare-dns.com",
        ] {
            assert_eq!(format_dns_server(&parse(input)), input);
        }
    }
}
