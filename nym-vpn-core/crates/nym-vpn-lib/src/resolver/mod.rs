// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! This module implements a forwarding DNS resolver with two states:
//! * In the `Blocked` state, most queries receive an empty response, but certain captive portal
//!   domains receive a spoofed answer. This fools the OS into thinking that it has connectivity.
//! * In the `Forwarding` state, queries are forwarded to a set of configured DNS servers. This
//!   lets us use the routing table to determine where to send them, instead of them being forced
//!   out on the primary interface (in some cases).
//!
//! Platform-specific responsibilities (binding sockets, adding loopback aliases, flushing system
//! DNS caches) are delegated to `platform`.

#[cfg(any(target_os = "macos", target_os = "linux"))]
mod unix;

use nym_common::trace_err_chain;
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(crate) use unix::flush_system_cache;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub(crate) use windows::flush_system_cache;

#[cfg(test)]
mod tests;

mod tcp;
use tcp::new_tcp_listener;

mod udp;
use udp::new_random_socket;

#[cfg(target_os = "ios")]
mod apple_connection_provider;

use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
    sync::{Arc, LazyLock},
    time::{Duration, Instant},
};

use async_trait::async_trait;
use hickory_resolver::config::{LookupIpStrategy, NameServerConfig, ResolverOpts};
use hickory_server::{
    net::runtime::Time,
    proto::{
        op::{Header, HeaderCounts, LowerQuery, MessageType, Metadata, OpCode, ResponseCode},
        rr::{LowerName, RData, Record, RecordType, domain::Name, rdata},
    },
    resolver::{
        config::ResolverConfig,
        lookup::Lookup,
        net::{DnsError, NetError},
    },
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo, Server},
    zone_handler::{
        AuthLookup, LookupRecordsIter, MessageRequest, MessageResponse, MessageResponseBuilder,
    },
};
#[cfg(not(target_os = "ios"))]
use hickory_server::{net::runtime::TokioRuntimeProvider, resolver::TokioResolver};
use tokio::{
    net::{TcpListener, UdpSocket},
    sync::{mpsc, oneshot},
};
use tokio_util::{either::Either, sync::CancellationToken};

#[cfg(target_os = "ios")]
use apple_connection_provider::{AppleConnectionProvider, TokioResolver};

#[async_trait]
#[cfg_attr(target_os = "ios", allow(unused))]
pub trait LoopbackAlias: Send {
    fn addr(&self) -> IpAddr;

    async fn unassign(self: Box<Self>);
}

pub type BoxedLoopbackAlias = Box<dyn LoopbackAlias>;

pub use crate::dns_filter::{
    DnsFilter, DnsFilterDecision, DnsFilterStrategy, DnsFilterT, NullDnsFilter,
};

/// Local DNS resolver listen port.
const DNS_LISTEN_PORT: u16 = if cfg!(test) { 1053 } else { 53 };

/// Types of records that are spoofed for captive portal domains.
const ALLOWED_RECORD_TYPES: &[RecordType] = &[RecordType::A, RecordType::CNAME];

/// Fully-qualified captive portal domains.
const CAPTIVE_PORTAL_DOMAINS: &[&str] = &["captive.apple.com.", "netcts.cdn-apple.com."];

/// Fully-qualified captive portal domain names as consumed by hickory.
static ALLOWED_DOMAINS: LazyLock<Vec<LowerName>> = LazyLock::new(|| {
    CAPTIVE_PORTAL_DOMAINS
        .iter()
        .map(|domain| LowerName::from(Name::from_str(domain).expect("Failed to parse domain")))
        .collect()
});

/// [`nym_conflict::PROBE_DOMAIN`] as consumed by hickory. Answered directly
/// with [`nym_conflict::PROBE_ADDR`], independent of ad-block/filter state,
/// so `nym_conflict::detect` can tell whether DNS queries from other
/// applications are actually reaching this resolver unaltered.
static CONFLICT_PROBE_DOMAIN: LazyLock<LowerName> = LazyLock::new(|| {
    LowerName::from(Name::from_str(nym_conflict::PROBE_DOMAIN).expect("Failed to parse domain"))
});

const TTL_SECONDS: u32 = 3;

/// An IP address to be used in the DNS response to the captive domain query. The address itself
/// belongs to the documentation range so should never be reachable.
const RESOLVED_ADDR: Ipv4Addr = Ipv4Addr::new(198, 51, 100, 1);

/// Timeout for TCP client connections.
/// Any client that does not send any DNS requests within the given timeout will be dropped.
pub const TCP_CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

/// Maximum number of DNS responses that can be queued for sending on a single TCP connection.
pub const TCP_RESPONSE_BUFFER_SIZE: usize = 32;

/// Resolver errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to bind UDP socket
    #[error("failed to bind UDP socket")]
    UdpBind,

    /// Failed to get local address of a bound UDP socket
    #[error("failed to get local address of a bound UDP socket")]
    GetSocketAddr(#[source] io::Error),

    /// Failed to create DNS resolver
    #[error("failed to create DNS resolver")]
    CreateResolver(#[source] hickory_resolver::net::NetError),
}

/// A DNS resolver that forwards queries to some other DNS server
///
/// Is controlled by commands sent through [ResolverHandle]s.
pub struct LocalResolver {
    rx: mpsc::UnboundedReceiver<ResolverMessage>,
    dns_server_task: tokio::task::JoinHandle<()>,
    bound_to: SocketAddr,
    inner_resolver: Resolver,
    dns_filter: DnsFilter,
    shutdown_token: CancellationToken,
}

/// A message to [LocalResolver]
enum ResolverMessage {
    /// Set resolver config
    SetConfig {
        /// New DNS config to use
        new_config: Config,
        /// Response channel when resolvers have been updated
        response_tx: oneshot::Sender<Result<(), Error>>,
    },

    /// Set the DNS-filter
    SetDnsFilter {
        /// New DNS filter to use
        dns_filter: DnsFilter,
        /// Response channel when resolvers have been updated
        response_tx: oneshot::Sender<()>,
    },

    /// Send a DNS query to the resolver
    Query {
        dns_query: LowerQuery,

        /// Channel for the query response
        response_tx: oneshot::Sender<Result<AuthLookup, NetError>>,
    },
}

/// Configuration for [Resolver]
#[derive(Debug, Default, Clone)]
enum Config {
    /// Drop DNS queries. For captive portal domains, return faux records.
    #[default]
    Blocking,

    /// Forward DNS queries to a configured server
    Forwarding {
        /// Remote DNS servers to use
        dns_servers: Vec<NameServerConfig>,

        /// Interface to bind client socket to.
        /// iOS only
        #[cfg(target_os = "ios")]
        bind_interface: Option<String>,
    },
}

enum Resolver {
    /// Drop DNS queries. For captive portal domains, return faux records
    Blocking,

    /// Forward DNS queries to a configured server
    Forwarding {
        resolver: Box<TokioResolver>,
        dns_filter: DnsFilter,
    },
}

impl Resolver {
    pub fn resolve(&self, query: LowerQuery, tx: oneshot::Sender<Result<AuthLookup, NetError>>) {
        tracing::trace!("resolve query: {}", query.to_string());
        let lookup = match self {
            Resolver::Blocking => Either::Left(async move { Self::resolve_blocked(query) }),
            Resolver::Forwarding {
                resolver,
                dns_filter,
            } => Either::Right(Self::resolve_forward(
                resolver.as_ref().clone(),
                query,
                dns_filter.clone(),
            )),
        };

        tokio::spawn(async move {
            let _ = tx.send(lookup.await);
        });
    }

    /// Resolution in blocked state will return spoofed records for captive portal domains.
    fn resolve_blocked(query: LowerQuery) -> Result<AuthLookup, NetError> {
        if !Self::is_captive_portal_domain(&query) {
            return Ok(AuthLookup::Empty);
        }

        let return_query = query.original().clone();
        let return_record = Record::from_rdata(
            return_query.name().clone(),
            TTL_SECONDS,
            RData::A(rdata::A(RESOLVED_ADDR)),
        );

        tracing::debug!(
            "Spoofing query for captive portal domain: {}",
            return_query.name()
        );

        let lookup = Lookup::new_with_deadline(
            return_query,
            [return_record],
            Instant::now() + Duration::from_secs(3),
        );
        Ok(AuthLookup::from(lookup))
    }

    /// Determines whether a DNS query is allowable. Currently, this implies that the query is
    /// either a `A` or a `CNAME` query for `captive.apple.com`.
    fn is_captive_portal_domain(query: &LowerQuery) -> bool {
        ALLOWED_RECORD_TYPES.contains(&query.query_type()) && ALLOWED_DOMAINS.contains(query.name())
    }

    /// Determines whether a DNS query is for the conflict-detection probe
    /// domain (see [`nym_conflict`]).
    fn is_conflict_probe_domain(query: &LowerQuery) -> bool {
        ALLOWED_RECORD_TYPES.contains(&query.query_type())
            && query.name() == &*CONFLICT_PROBE_DOMAIN
    }

    /// Always answers the conflict-detection probe domain with
    /// [`nym_conflict::PROBE_ADDR`], independent of ad-block/filter state.
    fn spoof_conflict_probe_response(
        return_query: &hickory_server::proto::op::Query,
    ) -> AuthLookup {
        let return_record = Record::from_rdata(
            return_query.name().clone(),
            TTL_SECONDS,
            RData::A(rdata::A(nym_conflict::PROBE_ADDR)),
        );

        let lookup = Lookup::new_with_deadline(
            return_query.clone(),
            [return_record],
            Instant::now() + Duration::from_secs(u64::from(TTL_SECONDS)),
        );
        AuthLookup::from(lookup)
    }

    async fn resolve_forward(
        resolver: TokioResolver,
        query: LowerQuery,
        dns_filter: DnsFilter,
    ) -> Result<AuthLookup, NetError> {
        let return_query = query.original().clone();

        if Self::is_conflict_probe_domain(&query) {
            tracing::trace!("Answering conflict-detection probe query");
            return Ok(Self::spoof_conflict_probe_response(&return_query));
        }

        let qname = return_query.name().to_ascii();
        let decision = dns_filter.should_block(&qname).await;
        if decision != DnsFilterDecision::Pass {
            tracing::trace!("Blocking DNS query for {qname} with strategy {decision:?}");
        }

        let result: AuthLookup = match decision {
            DnsFilterDecision::Pass => {
                let lookup = resolver
                    .lookup(return_query.name().clone(), return_query.query_type())
                    .await?;

                AuthLookup::from(lookup)
            }
            DnsFilterDecision::Block(DnsFilterStrategy::EmptyRecord) => AuthLookup::Empty,
            DnsFilterDecision::Block(DnsFilterStrategy::Localhost) => {
                let rdata = match return_query.query_type() {
                    RecordType::A => RData::A(rdata::A(Ipv4Addr::LOCALHOST)),
                    RecordType::AAAA => RData::AAAA(rdata::AAAA(Ipv6Addr::LOCALHOST)),
                    RecordType::CNAME => RData::CNAME(rdata::CNAME(Name::from_str("localhost.")?)),
                    other => {
                        tracing::trace!("Unsupported query type {other} for domain {qname}");
                        return Ok(AuthLookup::Empty);
                    }
                };

                let return_record =
                    Record::from_rdata(return_query.name().clone(), TTL_SECONDS, rdata);

                let lookup = Lookup::new_with_deadline(
                    return_query,
                    [return_record],
                    Instant::now() + Duration::from_secs(u64::from(TTL_SECONDS)),
                );
                AuthLookup::from(lookup)
            }
        };

        Ok(result)
    }
}

/// A handle to control a DNS resolver.
///
/// When all resolver handles are dropped, the resolver will stop.
#[derive(Clone)]
pub struct ResolverHandle {
    tx: mpsc::UnboundedSender<ResolverMessage>,
    listen_addr: SocketAddr,
}

impl ResolverHandle {
    fn new(tx: mpsc::UnboundedSender<ResolverMessage>, listen_addr: SocketAddr) -> Self {
        Self { tx, listen_addr }
    }

    /// Get listening address for resolver handle.
    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    /// Set the DNS servers to forward queries to `dns_servers`.
    pub async fn enable_forward(
        &self,
        dns_servers: Vec<NameServerConfig>,
        #[cfg(target_os = "ios")] bind_interface: Option<String>,
    ) -> Result<(), Error> {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(ResolverMessage::SetConfig {
                new_config: Config::Forwarding {
                    dns_servers,
                    #[cfg(target_os = "ios")]
                    bind_interface,
                },
                response_tx,
            })
            .is_ok()
        {
            response_rx.await.ok().unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    /// Disable forwarding.
    pub async fn disable_forward(&self) -> Result<(), Error> {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(ResolverMessage::SetConfig {
                new_config: Config::Blocking,
                response_tx,
            })
            .is_ok()
        {
            response_rx.await.ok().unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    /// Set the DNS filter.
    pub async fn set_dns_filter(&self, dns_filter: DnsFilter) {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(ResolverMessage::SetDnsFilter {
                dns_filter,
                response_tx,
            })
            .is_ok()
        {
            response_rx.await.ok();
        }
    }
}

impl LocalResolver {
    /// Spawn new filtering resolver and its handle.
    pub async fn spawn(
        use_random_loopback: bool,
        shutdown_token: CancellationToken,
    ) -> Result<(ResolverHandle, tokio::task::JoinHandle<()>), Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let (udp_socket, loopback_alias) =
            new_random_socket(DNS_LISTEN_PORT, use_random_loopback).await?;
        let resolver_addr = udp_socket.local_addr().map_err(Error::GetSocketAddr)?;

        // Attempt to bind TCP listener to the same port as UDP, but don't fail if it's not possible.
        let tcp_listener = new_tcp_listener(resolver_addr)
            .inspect_err(|_err| {
                tracing::warn!("Failed to bind TCP socket to {resolver_addr}");
            })
            .ok();
        let is_tcp_available = tcp_listener.is_some();

        let mut server = Self::new_server(udp_socket, tcp_listener, tx.clone()).await?;

        let cloned_shutdown_token = shutdown_token.child_token();
        let cloned_tx = tx.clone();
        let dns_server_task = tokio::spawn(async move {
            tracing::info!(
                "Running DNS resolver on {resolver_addr} ({})",
                if is_tcp_available { "udp, tcp" } else { "udp" }
            );

            loop {
                tokio::select! {
                    _ = cloned_shutdown_token.cancelled() => {
                        tracing::info!("Shutting down DNS server");
                        match server.shutdown_gracefully().await {
                            Ok(_) => {
                                tracing::info!("DNS server stopped gracefully");
                            },
                            Err(err) => {
                                tracing::error!("Failed to gracefully shutdown DNS server: {err}");
                            }
                        }
                        break;
                    }
                    result = server.block_until_done() => {
                        match result {
                            Ok(_) => {
                                tracing::info!("DNS server stopped gracefully");
                                break;
                            },
                            Err(err) => {
                                tracing::error!("DNS server unexpectedly stopped: {err}");
                                tracing::debug!("Attempting to restart server");

                                let udp_socket = match UdpSocket::bind(resolver_addr).await {
                                    Ok(socket) => socket,
                                    Err(e) => {
                                        tracing::error!("Failed to bind UDP socket to {resolver_addr}: {e}");
                                        break;
                                    }
                                };

                                let tcp_listener = TcpListener::bind(resolver_addr).await.inspect_err(|err| {
                                    tracing::warn!("Failed to bind TCP socket to {resolver_addr}: {err}");
                                }).ok();

                                match Self::new_server(udp_socket, tcp_listener, cloned_tx.clone()).await {
                                    Ok(new_server) => {
                                        server = new_server;
                                    }
                                    Err(error) => {
                                        tracing::error!("Failed to restart DNS server: {error}");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(loopback_alias) = loopback_alias {
                loopback_alias.unassign().await;
            }
        });

        let dns_filter: DnsFilter = Arc::new(NullDnsFilter);

        let resolver = Self {
            rx,
            dns_server_task,
            bound_to: resolver_addr,
            inner_resolver: Resolver::Blocking,
            dns_filter,
            shutdown_token,
        };

        // Spawn onto the multi-thread runtime (requires LocalResolver: Send)
        let join_handle = tokio::spawn(resolver.run());

        Ok((ResolverHandle::new(tx, resolver_addr), join_handle))
    }

    async fn new_server(
        server_socket: UdpSocket,
        tcp_listener: Option<TcpListener>,
        tx: mpsc::UnboundedSender<ResolverMessage>,
    ) -> Result<Server<ResolverImpl>, Error> {
        let mut server = Server::new(ResolverImpl { tx });
        server.register_socket(server_socket);
        if let Some(tcp_listener) = tcp_listener {
            server.register_listener(tcp_listener, TCP_CLIENT_TIMEOUT, TCP_RESPONSE_BUFFER_SIZE);
        }
        Ok(server)
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                request = self.rx.recv() => {
                    match request {
                        Some(ResolverMessage::SetConfig { new_config, response_tx }) => {
                            let res = self.update_config(new_config);
                            #[cfg(not(target_os = "ios"))]
                            if res.is_ok() {
                                flush_system_cache().await;
                            }
                            let _ = response_tx.send(res);
                        }
                        Some(ResolverMessage::SetDnsFilter { dns_filter, response_tx }) => {
                            // Store the new filter.
                            self.dns_filter = dns_filter;

                            // If we're currently forwarding, update the live resolver too.
                            if let Resolver::Forwarding { dns_filter, .. } = &mut self.inner_resolver {
                                *dns_filter = self.dns_filter.clone();
                            }

                            let _ = response_tx.send(());
                        }
                        Some(ResolverMessage::Query { dns_query, response_tx }) => {
                            self.inner_resolver.resolve(dns_query, response_tx);
                        }
                        None => {
                            self.shutdown_token.cancel();
                            break;
                        }
                    }
                },
                _ = self.shutdown_token.cancelled() => {
                    break;
                }
            }
        }

        tracing::debug!("Waiting for dns server task to finish");
        if let Err(e) = self.dns_server_task.await {
            tracing::error!("DNS server task failed: {e}");
        }
    }

    /// Update the current DNS config.
    fn update_config(&mut self, config: Config) -> Result<(), Error> {
        tracing::info!("Updating config: {config:?}");

        match config {
            Config::Blocking => {
                self.blocking();
                Ok(())
            }
            Config::Forwarding {
                mut dns_servers,
                #[cfg(target_os = "ios")]
                bind_interface,
            } => {
                // make sure not to accidentally forward queries to ourselves
                dns_servers.retain(|addr| addr.ip != self.bound_to.ip());
                self.forwarding(
                    dns_servers,
                    #[cfg(target_os = "ios")]
                    bind_interface,
                )
            }
        }
    }

    /// Turn into a blocking resolver.
    fn blocking(&mut self) {
        self.inner_resolver = Resolver::Blocking;
    }

    /// Turn into a forwarding resolver (forward DNS queries to [dns_servers]).
    fn forwarding(
        &mut self,
        dns_servers: Vec<NameServerConfig>,
        #[cfg(target_os = "ios")] bind_interface: Option<String>,
    ) -> Result<(), Error> {
        let forward_config = ResolverConfig::from_parts(None, vec![], dns_servers);

        #[cfg(target_os = "ios")]
        let connection_provider = AppleConnectionProvider::new(bind_interface);

        #[cfg(not(target_os = "ios"))]
        let connection_provider = TokioRuntimeProvider::default();

        let mut resolver_opts = ResolverOpts::default();
        resolver_opts.ip_strategy = LookupIpStrategy::Ipv4AndIpv6;

        let resolver = TokioResolver::builder_with_config(forward_config, connection_provider)
            .with_options(resolver_opts)
            .build()
            .map_err(Error::CreateResolver)?;

        self.inner_resolver = Resolver::Forwarding {
            resolver: Box::new(resolver),
            dns_filter: self.dns_filter.clone(),
        };

        Ok(())
    }
}

/// An implementation of [RequestHandler] that forwards queries.
struct ResolverImpl {
    tx: mpsc::UnboundedSender<ResolverMessage>,
}

impl ResolverImpl {
    fn build_response<'a>(
        message: &'a MessageRequest,
        lookup: &'a AuthLookup,
    ) -> MessageResponse<
        'a,
        'a,
        impl Iterator<Item = &'a Record> + Send + 'a,
        impl Iterator<Item = &'a Record> + Send + 'a,
        impl Iterator<Item = &'a Record> + Send + 'a,
        impl Iterator<Item = &'a Record> + Send + 'a,
    > {
        // Sets authoritative to false by default
        let mut response_meta = Metadata::response_from_request(&message.metadata);
        let builder = MessageResponseBuilder::from_message_request(message);

        if let AuthLookup::Resolved(resolved) = lookup {
            response_meta.recursion_available = resolved.message().metadata.recursion_available;
        }

        builder.build(
            response_meta,
            lookup.iter(),
            lookup.authorities().unwrap_or(LookupRecordsIter::Empty),
            std::iter::empty(),
            lookup.additionals().unwrap_or(LookupRecordsIter::Empty),
        )
    }

    /// Called when a DNS query is sent to the local resolver.
    async fn lookup<R: ResponseHandler>(
        &self,
        message: &Request,
        mut response_handler: R,
    ) -> Result<ResponseInfo, NetError> {
        tracing::trace!(
            "Lookup for: {}, client: {}/{}",
            &message
                .queries
                .queries()
                .iter()
                .map(|r| format!("{} {}", r.query_type(), r.name()))
                .collect::<Vec<_>>()
                .join(","),
            message.src(),
            message.protocol(),
        );

        let Some(query) = message.queries.queries().first() else {
            tracing::error!("Received a message without query");
            return Ok(make_response_info(message, ResponseCode::ServFail));
        };

        // BIND does not support multiple questions.
        if message.queries.queries().len() > 1 {
            tracing::error!("Received a message with multiple queries, using only the first one");
        }

        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(ResolverMessage::Query {
                dns_query: query.clone(),
                response_tx,
            })
            .is_err()
        {
            tracing::error!("Failed to send query to resolver");
            return Ok(make_response_info(message, ResponseCode::ServFail));
        };

        match response_rx.await {
            Ok(Ok(ref lookup)) => {
                let response = Self::build_response(message, lookup);
                response_handler
                    .send_response(response)
                    .await
                    .inspect_err(|err| {
                        trace_err_chain!(err, "failed to send response");
                    })
            }
            Ok(Err(resolve_err)) => {
                if let NetError::Dns(DnsError::NoRecordsFound(no_records)) = resolve_err {
                    let response_code = no_records.response_code;
                    let response = MessageResponseBuilder::from_message_request(message)
                        .error_msg(&message.metadata, response_code);
                    response_handler
                        .send_response(response)
                        .await
                        .inspect_err(|err| {
                            trace_err_chain!(err, "failed to send response");
                        })
                } else {
                    trace_err_chain!(resolve_err, "failed to resolve hostname");
                    Err(resolve_err)
                }
            }
            Err(_error) => Err(NetError::Message("channel is closed")),
        }
    }
}

fn make_response_info(message: &Request, response_code: ResponseCode) -> ResponseInfo {
    let mut metadata = Metadata::response_from_request(&message.metadata);
    metadata.response_code = response_code;
    let header = Header {
        metadata,
        counts: HeaderCounts::default(),
    };
    ResponseInfo::from(header)
}

#[async_trait::async_trait]
impl RequestHandler for ResolverImpl {
    async fn handle_request<R: ResponseHandler, T: Time>(
        &self,
        request: &Request,
        response_handle: R,
    ) -> ResponseInfo {
        if !request.src().ip().is_loopback() {
            tracing::error!("Dropping a stray request from outside: {}", request.src());
            make_response_info(request, ResponseCode::Refused)
        } else if request.metadata.message_type == MessageType::Query
            && request.metadata.op_code == OpCode::Query
        {
            self.lookup(request, response_handle)
                .await
                .unwrap_or_else(|_err| make_response_info(request, ResponseCode::ServFail))
        } else {
            tracing::trace!("Dropping non-query request: {:?}", request);
            make_response_info(request, ResponseCode::Refused)
        }
    }
}

#[cfg(not(target_os = "ios"))]
pub fn random_loopback_ipv4() -> IpAddr {
    use rand::Rng;

    IpAddr::from(Ipv4Addr::new(
        127,
        rand::thread_rng().gen_range(1..=255),
        rand::random(),
        // keep last octet in the range of 1-254 to avoid special addresses
        rand::thread_rng().gen_range(1..=254),
    ))
}
