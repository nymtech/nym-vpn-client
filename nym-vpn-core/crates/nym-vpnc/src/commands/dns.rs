// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use anyhow::{Result, anyhow, bail};
use clap::{Arg, ArgAction, ArgMatches};
use nym_vpn_lib_types::{DnsServer, PlainDnsProtocol};
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

    /// Set custom DNS servers
    ///
    /// Servers are used in the order given. Each server is a transport option followed by
    /// HOST[:PORT], or --url with a URL:
    ///
    ///   --plain, --udp, --tcp              plain DNS over UDP and TCP, UDP only or TCP only
    ///   --tls, --https, --quic, --h3       DNS over TLS, HTTPS, QUIC or HTTP/3
    ///
    ///   udp://HOST[:PORT]                  plain DNS over UDP
    ///   tcp://HOST[:PORT]                  plain DNS over TCP
    ///   tls://HOST[:PORT][#NAME]           DNS over TLS
    ///   https://HOST[:PORT][/PATH][#NAME]  DNS over HTTPS
    ///   quic://HOST[:PORT][#NAME]          DNS over QUIC
    ///   h3://HOST[:PORT][/PATH][#NAME]     DNS over HTTP/3
    ///
    /// HOST is an IP address or a domain name. A domain name is resolved to its first IPv4
    /// address before the server is saved. IPv6 addresses go in brackets when followed by a port
    /// or used in a URL, e.g. [::1]:5353 or tls://[2606:4700:4700::1111]#cloudflare-dns.com
    ///
    /// NAME is the TLS server name. It defaults to HOST when HOST is a domain name, e.g.
    /// tls://dns.google
    ///
    /// --hostname and --query set NAME and PATH for the transport option right before them, e.g.
    ///
    ///   nym-vpnc dns set --tls 1.1.1.1 --hostname cloudflare-dns.com --plain 9.9.9.9
    ///   nym-vpnc dns set --https 8.8.8.8 --hostname dns.google --query /dns-query
    ///   nym-vpnc dns set --url tls://dns.google --url https://8.8.8.8/dns-query#dns.google
    #[command(verbatim_doc_comment)]
    Set {
        #[command(flatten)]
        servers: DnsServerArgs,
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
                    "Custom DNS: {}",
                    if config.custom_dns.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );

                for dns_server in config.custom_dns.dns_servers {
                    println!("{dns_server}");
                }
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
            Command::Set { servers } => {
                if servers.dns_servers.is_empty() {
                    bail!(
                        "Use `nym-vpnc dns clear` instead if you intend to remove all custom DNS."
                    );
                }
                let dns_servers = servers.resolve().await?;
                rpc_client.set_custom_dns(dns_servers).await?;
                Ok(())
            }
            Command::Clear => {
                rpc_client.set_custom_dns(Vec::new()).await?;
                Ok(())
            }
            Command::GetDefault => {
                let default_dns = rpc_client.get_default_dns().await?;
                println!("Default DNS:");
                for dns_server in default_dns {
                    println!("{}", dns_server)
                }
                Ok(())
            }
        }
    }
}

const URL_ARG: &str = "url";
const HOSTNAME_ARG: &str = "hostname";
const QUERY_ARG: &str = "query";

#[derive(Debug, Clone, Copy, PartialEq, Eq, enum_iterator::Sequence)]
enum Transport {
    Plain,
    Udp,
    Tcp,
    Tls,
    Https,
    Quic,
    H3,
}

impl Transport {
    fn flag(self) -> &'static str {
        match self {
            Transport::Plain => "plain",
            Transport::Udp => "udp",
            Transport::Tcp => "tcp",
            Transport::Tls => "tls",
            Transport::Https => "https",
            Transport::Quic => "quic",
            Transport::H3 => "h3",
        }
    }

    fn help(self) -> &'static str {
        match self {
            Transport::Plain => "Plain DNS server over UDP and TCP",
            Transport::Udp => "Plain DNS server over UDP",
            Transport::Tcp => "Plain DNS server over TCP",
            Transport::Tls => "DNS over TLS server, optionally followed by --hostname",
            Transport::Https => {
                "DNS over HTTPS server, optionally followed by --hostname and --query"
            }
            Transport::Quic => "DNS over QUIC server, optionally followed by --hostname",
            Transport::H3 => {
                "DNS over HTTP/3 server, optionally followed by --hostname and --query"
            }
        }
    }

    fn has_hostname(self) -> bool {
        matches!(
            self,
            Transport::Tls | Transport::Https | Transport::Quic | Transport::H3
        )
    }

    fn has_query(self) -> bool {
        matches!(self, Transport::Https | Transport::H3)
    }
}

/// DNS servers for `dns set`, from --url and transport options in command line order.
#[derive(Debug, Clone)]
pub struct DnsServerArgs {
    dns_servers: Vec<DnsServerSpec>,
}

impl DnsServerArgs {
    async fn resolve(self) -> Result<Vec<DnsServer>> {
        let mut dns_servers = Vec::with_capacity(self.dns_servers.len());
        for spec in self.dns_servers {
            dns_servers.push(spec.resolve().await?);
        }
        Ok(dns_servers)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ServerHost {
    Ip(IpAddr),
    Domain(String),
}

impl ServerHost {
    fn domain(&self) -> Option<&str> {
        match self {
            ServerHost::Ip(_) => None,
            ServerHost::Domain(domain) => Some(domain),
        }
    }
}

/// A DNS server as given on the command line.
///
/// When `domain` is set, `server` holds an unspecified address until `resolve` replaces it with
/// the domain's first IPv4 address.
#[derive(Debug, Clone, PartialEq)]
struct DnsServerSpec {
    domain: Option<String>,
    server: DnsServer,
}

impl DnsServerSpec {
    fn new(host: ServerHost, server: impl FnOnce(IpAddr) -> DnsServer) -> Self {
        match host {
            ServerHost::Ip(address) => Self {
                domain: None,
                server: server(address),
            },
            ServerHost::Domain(domain) => Self {
                domain: Some(domain),
                server: server(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            },
        }
    }

    async fn resolve(self) -> Result<DnsServer> {
        let Self { domain, mut server } = self;
        let Some(domain) = domain else {
            return Ok(server);
        };
        let resolved = tokio::net::lookup_host((domain.as_str(), 0))
            .await
            .map_err(|e| anyhow!("Failed to resolve '{domain}': {e}"))?
            .map(|addr| addr.ip())
            .find(IpAddr::is_ipv4)
            .ok_or_else(|| anyhow!("'{domain}' has no IPv4 address"))?;
        println!("Resolved {domain} to {resolved}");
        match &mut server {
            DnsServer::Plain { address, .. }
            | DnsServer::Tls { address, .. }
            | DnsServer::Https { address, .. }
            | DnsServer::Quic { address, .. }
            | DnsServer::Http3 { address, .. } => *address = resolved,
        }
        Ok(server)
    }
}

impl clap::Args for DnsServerArgs {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        let cmd = cmd.arg(
            Arg::new(URL_ARG)
                .long(URL_ARG)
                .value_name("URL")
                .help("DNS server URL, e.g. tls://dns.google")
                .action(ArgAction::Append)
                .value_parser(parse_url_server),
        );
        enum_iterator::all::<Transport>()
            .fold(cmd, |cmd, transport| {
                cmd.arg(
                    Arg::new(transport.flag())
                        .long(transport.flag())
                        .value_name("HOST[:PORT]")
                        .help(transport.help())
                        .action(ArgAction::Append)
                        .value_parser(parse_host_port),
                )
            })
            .arg(
                Arg::new(HOSTNAME_ARG)
                    .long(HOSTNAME_ARG)
                    .value_name("HOSTNAME")
                    .help(
                        "TLS server name for the preceding --tls, --https, --quic or --h3 (default: its HOST when that is a domain name)",
                    )
                    .action(ArgAction::Append),
            )
            .arg(
                Arg::new(QUERY_ARG)
                    .long(QUERY_ARG)
                    .value_name("PATH")
                    .help("Query path for the preceding --https or --h3 (default: /dns-query)")
                    .action(ArgAction::Append),
            )
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args(cmd)
    }
}

impl clap::FromArgMatches for DnsServerArgs {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let dns_servers = dns_servers_from_matches(matches).map_err(|e| {
            clap::Error::raw(clap::error::ErrorKind::ValueValidation, e.to_string())
        })?;
        Ok(Self { dns_servers })
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
}

enum DnsServerArg {
    Url(DnsServerSpec),
    Transport(Transport, (ServerHost, Option<u16>)),
    Hostname(String),
    Query(String),
}

struct PendingServer {
    transport: Transport,
    host: ServerHost,
    port: Option<u16>,
    hostname: Option<String>,
    query: Option<String>,
}

impl PendingServer {
    fn describe(&self) -> String {
        format!(
            "--{} {}",
            self.transport.flag(),
            format_server_host_port(&self.host, self.port)
        )
    }

    fn into_spec(self) -> Result<DnsServerSpec> {
        let description = self.describe();
        let Self {
            transport,
            host,
            port,
            hostname,
            query,
        } = self;
        let server_name = match hostname.or_else(|| host.domain().map(str::to_owned)) {
            Some(server_name) => server_name,
            None if transport.has_hostname() => bail!("{description} requires --hostname"),
            None => String::new(),
        };
        let query_path = query.map(|query| match query.starts_with('/') {
            true => query,
            false => format!("/{query}"),
        });
        Ok(DnsServerSpec::new(host, |address| match transport {
            Transport::Plain => DnsServer::Plain {
                address,
                port,
                ip_proto: PlainDnsProtocol::UdpAndTcp,
            },
            Transport::Udp => DnsServer::Plain {
                address,
                port,
                ip_proto: PlainDnsProtocol::Udp,
            },
            Transport::Tcp => DnsServer::Plain {
                address,
                port,
                ip_proto: PlainDnsProtocol::Tcp,
            },
            Transport::Tls => DnsServer::Tls {
                address,
                port,
                server_name,
            },
            Transport::Quic => DnsServer::Quic {
                address,
                port,
                server_name,
            },
            Transport::Https => DnsServer::Https {
                address,
                port,
                server_name,
                query_path,
            },
            Transport::H3 => DnsServer::Http3 {
                address,
                port,
                server_name,
                query_path,
            },
        }))
    }
}

enum Entry {
    Done(DnsServerSpec),
    Pending(PendingServer),
}

fn indexed<'a, T: Clone + Send + Sync + 'static>(
    matches: &'a ArgMatches,
    id: &str,
    wrap: impl Fn(T) -> DnsServerArg + 'a,
) -> impl Iterator<Item = (usize, DnsServerArg)> + 'a {
    let indices = matches.indices_of(id).into_iter().flatten();
    let values = matches.get_many::<T>(id).into_iter().flatten().cloned();
    indices.zip(values.map(wrap))
}

fn dns_servers_from_matches(matches: &ArgMatches) -> Result<Vec<DnsServerSpec>> {
    let mut args: Vec<(usize, DnsServerArg)> =
        indexed(matches, URL_ARG, DnsServerArg::Url).collect();
    for transport in enum_iterator::all::<Transport>() {
        args.extend(indexed(matches, transport.flag(), move |value| {
            DnsServerArg::Transport(transport, value)
        }));
    }
    args.extend(indexed(matches, HOSTNAME_ARG, DnsServerArg::Hostname));
    args.extend(indexed(matches, QUERY_ARG, DnsServerArg::Query));
    args.sort_by_key(|(index, _)| *index);

    collect_dns_servers(args.into_iter().map(|(_, arg)| arg))
}

fn collect_dns_servers(args: impl IntoIterator<Item = DnsServerArg>) -> Result<Vec<DnsServerSpec>> {
    let mut entries: Vec<Entry> = Vec::new();
    for arg in args {
        match arg {
            DnsServerArg::Url(server) => entries.push(Entry::Done(server)),
            DnsServerArg::Transport(transport, (host, port)) => {
                entries.push(Entry::Pending(PendingServer {
                    transport,
                    host,
                    port,
                    hostname: None,
                    query: None,
                }))
            }
            DnsServerArg::Hostname(hostname) => match entries.last_mut() {
                Some(Entry::Pending(pending)) if pending.transport.has_hostname() => {
                    if pending.hostname.is_some() {
                        bail!("--hostname is given twice for {}", pending.describe());
                    }
                    if hostname.is_empty() {
                        bail!("--hostname for {} must not be empty", pending.describe());
                    }
                    pending.hostname = Some(hostname);
                }
                _ => bail!("--hostname {hostname} must follow --tls, --https, --quic or --h3"),
            },
            DnsServerArg::Query(query) => match entries.last_mut() {
                Some(Entry::Pending(pending)) if pending.transport.has_query() => {
                    if pending.query.is_some() {
                        bail!("--query is given twice for {}", pending.describe());
                    }
                    pending.query = Some(query);
                }
                _ => bail!("--query {query} must follow --https or --h3"),
            },
        }
    }
    entries
        .into_iter()
        .map(|entry| match entry {
            Entry::Done(server) => Ok(server),
            Entry::Pending(pending) => pending.into_spec(),
        })
        .collect()
}

fn parse_ip_port(input: &str) -> Option<(IpAddr, Option<u16>)> {
    let bracketed_ipv6 = input
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .and_then(|inner| inner.parse::<Ipv6Addr>().ok())
        .map(IpAddr::V6);
    match bracketed_ipv6.or_else(|| input.parse::<IpAddr>().ok()) {
        Some(address) => Some((address, None)),
        None => input
            .parse::<SocketAddr>()
            .ok()
            .map(|addr| (addr.ip(), Some(addr.port()))),
    }
}

fn parse_domain(input: &str) -> Option<String> {
    match Host::parse(input) {
        Ok(Host::Domain(domain)) => Some(domain),
        _ => None,
    }
}

fn parse_host_port(input: &str) -> Result<(ServerHost, Option<u16>)> {
    if let Some((address, port)) = parse_ip_port(input) {
        return Ok((ServerHost::Ip(address), port));
    }
    let invalid = || {
        anyhow!(
            "Invalid address '{input}', expected IP[:PORT] or DOMAIN[:PORT] with IPv6 in brackets when a port is given"
        )
    };
    let (host, port) = match input.rsplit_once(':') {
        Some((host, port)) => (host, Some(port.parse::<u16>().map_err(|_| invalid())?)),
        None => (input, None),
    };
    let domain = parse_domain(host).ok_or_else(invalid)?;
    Ok((ServerHost::Domain(domain), port))
}

fn parse_url_server(input: &str) -> Result<DnsServerSpec> {
    if !input.contains("://") {
        bail!("Invalid DNS server URL '{input}', expected SCHEME://HOST, or use --plain {input}");
    }

    let url = Url::parse(input).map_err(|e| anyhow!("Invalid DNS server '{input}': {e}"))?;

    if !url.username().is_empty() || url.password().is_some() {
        bail!("DNS server URL '{input}' must not contain credentials");
    }
    if url.query().is_some() {
        bail!("DNS server URL '{input}' must not contain a query");
    }

    let host = url_host(&url).ok_or_else(|| {
        anyhow!("DNS server URL '{input}' must use an IP address or a domain name as the host")
    })?;
    let port = url.port();

    Ok(match url.scheme() {
        "udp" => {
            reject_server_name(&url, input)?;
            reject_path(&url, input)?;
            DnsServerSpec::new(host, |address| DnsServer::Plain {
                address,
                port,
                ip_proto: PlainDnsProtocol::Udp,
            })
        }
        "tcp" => {
            reject_server_name(&url, input)?;
            reject_path(&url, input)?;
            DnsServerSpec::new(host, |address| DnsServer::Plain {
                address,
                port,
                ip_proto: PlainDnsProtocol::Tcp,
            })
        }
        "tls" => {
            reject_path(&url, input)?;
            let server_name = url_server_name(&url, &host, input)?;
            DnsServerSpec::new(host, |address| DnsServer::Tls {
                address,
                port,
                server_name,
            })
        }
        "quic" => {
            reject_path(&url, input)?;
            let server_name = url_server_name(&url, &host, input)?;
            DnsServerSpec::new(host, |address| DnsServer::Quic {
                address,
                port,
                server_name,
            })
        }
        "https" => {
            let server_name = url_server_name(&url, &host, input)?;
            let query_path = Some(url_query_path(&url));
            DnsServerSpec::new(host, |address| DnsServer::Https {
                address,
                port,
                server_name,
                query_path,
            })
        }
        "h3" => {
            let server_name = url_server_name(&url, &host, input)?;
            let query_path = Some(url_query_path(&url));
            DnsServerSpec::new(host, |address| DnsServer::Http3 {
                address,
                port,
                server_name,
                query_path,
            })
        }
        scheme => bail!(
            "Unsupported DNS server scheme '{scheme}' in '{input}', expected one of: udp, tcp, tls, https, quic, h3"
        ),
    })
}

fn url_host(url: &Url) -> Option<ServerHost> {
    match url.host()? {
        Host::Ipv4(address) => Some(ServerHost::Ip(IpAddr::V4(address))),
        Host::Ipv6(address) => Some(ServerHost::Ip(IpAddr::V6(address))),
        // Non-special schemes (tls, quic, ...) keep hosts as opaque strings, including IPv4.
        Host::Domain(host) => match host.parse::<Ipv4Addr>() {
            Ok(address) => Some(ServerHost::Ip(IpAddr::V4(address))),
            Err(_) => parse_domain(host).map(ServerHost::Domain),
        },
    }
}

fn url_server_name(url: &Url, host: &ServerHost, input: &str) -> Result<String> {
    match (url.fragment(), host.domain()) {
        (Some(name), _) if !name.is_empty() => Ok(name.to_owned()),
        (None, Some(domain)) => Ok(domain.to_owned()),
        _ => Err(anyhow!(
            "DNS server URL '{input}' must end with the TLS server name, e.g. #dns.example.com, or use a domain name as the host"
        )),
    }
}

fn reject_server_name(url: &Url, input: &str) -> Result<()> {
    match url.fragment() {
        Some(_) => bail!("Plain DNS server URL '{input}' must not contain a server name"),
        None => Ok(()),
    }
}

fn reject_path(url: &Url, input: &str) -> Result<()> {
    match url.path() {
        "" | "/" => Ok(()),
        _ => bail!("DNS server URL '{input}' must not contain a path"),
    }
}

/// The URL path as given. Non-special schemes like h3 have an empty path when none is given,
/// which HTTP treats as `/`.
fn url_query_path(url: &Url) -> String {
    match url.path() {
        "" => "/".to_owned(),
        path => path.to_owned(),
    }
}

fn format_server_host_port(host: &ServerHost, port: Option<u16>) -> String {
    match host {
        ServerHost::Ip(address) => format_host_port(*address, port),
        ServerHost::Domain(domain) => match port {
            Some(port) => format!("{domain}:{port}"),
            None => domain.clone(),
        },
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> DnsServer {
        let spec = parse_url_server(input).unwrap();
        assert_eq!(spec.domain, None, "{input}");
        spec.server
    }

    fn plain(input: &str) -> DnsServer {
        let [server] = parse_set(&["--plain", input]).unwrap().try_into().unwrap();
        server
    }

    fn parse_domain_spec(input: &str) -> DnsServerSpec {
        parse_url_server(input).unwrap()
    }

    fn plain_spec(input: &str) -> DnsServerSpec {
        let [spec] = parse_set_specs(&["--plain", input])
            .unwrap()
            .try_into()
            .unwrap();
        spec
    }

    fn unresolved(domain: &str, server: DnsServer) -> DnsServerSpec {
        DnsServerSpec {
            domain: Some(domain.to_owned()),
            server,
        }
    }

    const UNSPECIFIED: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);

    #[derive(Debug, clap::Parser)]
    struct SetCli {
        #[command(flatten)]
        servers: DnsServerArgs,
    }

    fn parse_set_specs(args: &[&str]) -> Result<Vec<DnsServerSpec>, clap::Error> {
        use clap::Parser;
        SetCli::try_parse_from(std::iter::once("set").chain(args.iter().copied()))
            .map(|cli| cli.servers.dns_servers)
    }

    fn parse_set(args: &[&str]) -> Result<Vec<DnsServer>, clap::Error> {
        parse_set_specs(args).map(|specs| {
            specs
                .into_iter()
                .map(|spec| {
                    assert_eq!(spec.domain, None, "{args:?}");
                    spec.server
                })
                .collect()
        })
    }

    #[test]
    fn parses_domain_hosts() {
        assert_eq!(
            plain_spec("DNS.Google:5353"),
            unresolved(
                "dns.google",
                DnsServer::Plain {
                    address: UNSPECIFIED,
                    port: Some(5353),
                    ip_proto: PlainDnsProtocol::UdpAndTcp,
                }
            )
        );
        assert_eq!(
            parse_domain_spec("udp://dns.google"),
            unresolved(
                "dns.google",
                DnsServer::Plain {
                    address: UNSPECIFIED,
                    port: None,
                    ip_proto: PlainDnsProtocol::Udp,
                }
            )
        );
        assert_eq!(
            parse_domain_spec("tls://one.one.one.one:853"),
            unresolved(
                "one.one.one.one",
                DnsServer::Tls {
                    address: UNSPECIFIED,
                    port: Some(853),
                    server_name: "one.one.one.one".to_owned(),
                }
            )
        );
        assert_eq!(
            parse_domain_spec("https://dns.google/dns-query"),
            unresolved(
                "dns.google",
                DnsServer::Https {
                    address: UNSPECIFIED,
                    port: None,
                    server_name: "dns.google".to_owned(),
                    query_path: Some("/dns-query".to_owned()),
                }
            )
        );
        assert_eq!(
            parse_domain_spec("quic://dns.adguard-dns.com#unfiltered.adguard-dns.com"),
            unresolved(
                "dns.adguard-dns.com",
                DnsServer::Quic {
                    address: UNSPECIFIED,
                    port: None,
                    server_name: "unfiltered.adguard-dns.com".to_owned(),
                }
            )
        );
    }

    #[test]
    fn parses_domain_hosts_in_transport_options() {
        assert_eq!(
            parse_set_specs(&[
                "--tls",
                "dns.google",
                "--h3",
                "cloudflare-dns.com:443",
                "--hostname",
                "one.one.one.one",
                "--query",
                "/dns-query",
                "--tcp",
                "dns.quad9.net",
            ])
            .unwrap(),
            vec![
                unresolved(
                    "dns.google",
                    DnsServer::Tls {
                        address: UNSPECIFIED,
                        port: None,
                        server_name: "dns.google".to_owned(),
                    }
                ),
                unresolved(
                    "cloudflare-dns.com",
                    DnsServer::Http3 {
                        address: UNSPECIFIED,
                        port: Some(443),
                        server_name: "one.one.one.one".to_owned(),
                        query_path: Some("/dns-query".to_owned()),
                    }
                ),
                unresolved(
                    "dns.quad9.net",
                    DnsServer::Plain {
                        address: UNSPECIFIED,
                        port: None,
                        ip_proto: PlainDnsProtocol::Tcp,
                    }
                ),
            ]
        );
    }

    #[tokio::test]
    async fn resolves_domain_to_first_ipv4() {
        assert_eq!(
            plain_spec("localhost:5353").resolve().await.unwrap(),
            DnsServer::Plain {
                address: "127.0.0.1".parse().unwrap(),
                port: Some(5353),
                ip_proto: PlainDnsProtocol::UdpAndTcp,
            }
        );
        assert_eq!(
            parse_domain_spec("tls://localhost")
                .resolve()
                .await
                .unwrap(),
            DnsServer::Tls {
                address: "127.0.0.1".parse().unwrap(),
                port: None,
                server_name: "localhost".to_owned(),
            }
        );
        assert_eq!(
            plain_spec("9.9.9.9").resolve().await.unwrap(),
            plain("9.9.9.9")
        );
    }

    #[tokio::test]
    async fn fails_to_resolve_unknown_domain() {
        assert!(
            plain_spec("does-not-exist.invalid")
                .resolve()
                .await
                .is_err()
        );
    }

    #[test]
    fn parses_transport_options_in_command_line_order() {
        assert_eq!(
            parse_set(&[
                "--tls",
                "1.1.1.1:853",
                "--hostname",
                "cloudflare-dns.com",
                "--plain",
                "9.9.9.9",
                "--udp",
                "[::1]:5353",
                "--https",
                "8.8.8.8",
                "--query",
                "dns-query",
                "--hostname",
                "dns.google",
                "--url",
                "quic://94.140.14.14#dns.adguard-dns.com",
                "--plain",
                "2606:4700:4700::1111",
            ])
            .unwrap(),
            vec![
                DnsServer::Tls {
                    address: "1.1.1.1".parse().unwrap(),
                    port: Some(853),
                    server_name: "cloudflare-dns.com".to_owned(),
                },
                plain("9.9.9.9"),
                DnsServer::Plain {
                    address: "::1".parse().unwrap(),
                    port: Some(5353),
                    ip_proto: PlainDnsProtocol::Udp,
                },
                DnsServer::Https {
                    address: "8.8.8.8".parse().unwrap(),
                    port: None,
                    server_name: "dns.google".to_owned(),
                    query_path: Some("/dns-query".to_owned()),
                },
                parse("quic://94.140.14.14#dns.adguard-dns.com"),
                DnsServer::Plain {
                    address: "2606:4700:4700::1111".parse().unwrap(),
                    port: None,
                    ip_proto: PlainDnsProtocol::UdpAndTcp,
                },
            ]
        );
    }

    #[test]
    fn parses_each_transport_option() {
        assert_eq!(
            parse_set(&[
                "--tcp",
                "9.9.9.9:53",
                "--quic",
                "94.140.14.14",
                "--hostname",
                "dns.adguard-dns.com",
                "--h3",
                "[2606:4700:4700::1111]:443",
                "--hostname",
                "cloudflare-dns.com",
                "--query",
                "/dns-query",
            ])
            .unwrap(),
            vec![
                parse("tcp://9.9.9.9:53"),
                parse("quic://94.140.14.14#dns.adguard-dns.com"),
                parse("h3://[2606:4700:4700::1111]:443/dns-query#cloudflare-dns.com"),
            ]
        );
    }

    #[test]
    fn parses_bracketed_ipv6_without_port() {
        assert_eq!(
            parse_set(&["--udp", "[::1]"]).unwrap(),
            vec![parse("udp://[::1]")]
        );
    }

    #[test]
    fn no_servers_is_empty() {
        assert_eq!(parse_set(&[]).unwrap(), vec![]);
    }

    #[test]
    fn rejects_invalid_transport_options() {
        for args in [
            &["--tls", "1.1.1.1"][..],
            &["--https", "8.8.8.8", "--query", "/dns-query"],
            &["--hostname", "dns.google", "--tls", "8.8.8.8"],
            &["--udp", "1.1.1.1", "--hostname", "one.one"],
            &["--tls", "1.1.1.1", "--hostname", "a", "--hostname", "b"],
            &["--tls", "1.1.1.1", "--hostname", ""],
            &["--tls", "1.1.1.1", "--hostname", "a", "--query", "/q"],
            &[
                "--https",
                "8.8.8.8",
                "--hostname",
                "a",
                "--query",
                "/q",
                "--query",
                "/r",
            ],
            &["--url", "tls://1.1.1.1#a", "--hostname", "b"],
            &["9.9.9.9"],
            &["--url", "9.9.9.9"],
            &["--url", "dns.google:53"],
            &["--plain", "1.1.1.1:"],
            &["--plain", "[1.1.1.1]"],
            &["--plain", "dns.google:"],
            &["--plain", "dns.google:70000"],
            &["--plain", "not a domain"],
            &["--plain", "1.1.1.1:70000"],
            &["--plain", "udp://1.1.1.1"],
            &["--tcp", "1.1.1.1:70000"],
        ] {
            assert!(parse_set(args).is_err(), "{args:?} should be rejected");
        }
    }

    #[test]
    fn parses_plain_ip() {
        assert_eq!(
            plain("1.1.1.1"),
            DnsServer::Plain {
                address: "1.1.1.1".parse().unwrap(),
                port: None,
                ip_proto: PlainDnsProtocol::UdpAndTcp,
            }
        );
        assert_eq!(
            plain("2606:4700:4700::1111"),
            DnsServer::Plain {
                address: "2606:4700:4700::1111".parse().unwrap(),
                port: None,
                ip_proto: PlainDnsProtocol::UdpAndTcp,
            }
        );
    }

    #[test]
    fn parses_plain_ip_with_port() {
        assert_eq!(
            plain("1.1.1.1:5353"),
            DnsServer::Plain {
                address: "1.1.1.1".parse().unwrap(),
                port: Some(5353),
                ip_proto: PlainDnsProtocol::UdpAndTcp,
            }
        );
        assert_eq!(
            plain("[::1]:53"),
            DnsServer::Plain {
                address: "::1".parse().unwrap(),
                port: Some(53),
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
                query_path: Some("/".to_owned()),
            }
        );
    }

    #[test]
    fn keeps_url_path_as_given() {
        for input in ["https://8.8.8.8#dns.google", "https://8.8.8.8/#dns.google"] {
            assert_eq!(
                parse(input),
                DnsServer::Https {
                    address: "8.8.8.8".parse().unwrap(),
                    port: None,
                    server_name: "dns.google".to_owned(),
                    query_path: Some("/".to_owned()),
                },
                "{input}"
            );
        }
        assert_eq!(
            parse("h3://1.1.1.1/resolve#cloudflare-dns.com"),
            DnsServer::Http3 {
                address: "1.1.1.1".parse().unwrap(),
                port: None,
                server_name: "cloudflare-dns.com".to_owned(),
                query_path: Some("/resolve".to_owned()),
            }
        );
    }

    #[test]
    fn rejects_invalid_urls() {
        for input in [
            "1.1.1.1",
            "dns.google:53",
            "tls://1.1.1.1",
            "tls://1.1.1.1#",
            "tls://999.1.1.1#name",
            "udp://dns.google#name",
            "udp://1.1.1.1#name",
            "tcp://1.1.1.1/path",
            "quic://1.1.1.1/path#name",
            "https://8.8.8.8/dns-query?x=1#dns.google",
            "https://user@8.8.8.8#dns.google",
            "ftp://1.1.1.1",
            "plain://1.1.1.1",
            "tls://1.1.1.1:70000#name",
        ] {
            assert!(
                parse_url_server(input).is_err(),
                "{input} should be rejected"
            );
        }
    }
}
