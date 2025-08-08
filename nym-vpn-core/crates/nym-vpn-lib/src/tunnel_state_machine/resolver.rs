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
//! See [start_resolver].
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::{Arc, LazyLock},
    time::{Duration, Instant},
};

use hickory_server::{
    ServerFuture,
    authority::{
        EmptyLookup, LookupObject, MessageRequest, MessageResponse, MessageResponseBuilder,
    },
    proto::{
        ProtoErrorKind,
        op::{Header, LowerQuery, ResponseCode, header::MessageType, op_code::OpCode},
        rr::{LowerName, Record, RecordType, domain::Name, rdata, record_data::RData},
    },
    resolver::{
        ResolveError, TokioResolver,
        config::{NameServerConfigGroup, ResolverConfig},
        lookup::Lookup,
        name_server::TokioConnectionProvider,
    },
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
};
use tokio::{
    net::UdpSocket,
    sync::{mpsc, oneshot},
};
use tokio_util::{either::Either, sync::CancellationToken};

/// If a local DNS resolver should be used at all times.
///
/// This setting does not affect the error or blocked state. In those states, we will want to use
/// the local DNS resoler to work around Apple's captive portals check. Exactly how this is done is
/// documented elsewhere.
pub static LOCAL_DNS_RESOLVER: LazyLock<bool> = LazyLock::new(|| {
    let disable_local_dns_resolver = std::env::var("NYM_DISABLE_LOCAL_DNS_RESOLVER")
        .map(|v| v != "0")
        // Use the local DNS resolver by default.
        .unwrap_or(false);
    if !disable_local_dns_resolver {
        tracing::info!("Using local DNS resolver");
    }
    !disable_local_dns_resolver
});

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

const TTL_SECONDS: u32 = 3;
/// An IP address to be used in the DNS response to the captive domain query. The address itself
/// belongs to the documentation range so should never be reachable.
const RESOLVED_ADDR: Ipv4Addr = Ipv4Addr::new(198, 51, 100, 1);

/// Resolver errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to bind UDP socket
    #[error("failed to bind UDP socket")]
    UdpBindError(#[source] io::Error),

    /// Failed to get local address of a bound UDP socket
    #[error("failed to get local address of a bound UDP socket")]
    GetSocketAddrError(#[source] io::Error),
}

/// A DNS resolver that forwards queries to some other DNS server
///
/// Is controlled by commands sent through [ResolverHandle]s.
pub struct LocalResolver {
    rx: mpsc::UnboundedReceiver<ResolverMessage>,
    dns_server_task: tokio::task::JoinHandle<()>,
    bound_to: SocketAddr,
    inner_resolver: Resolver,
    shutdown_token: CancellationToken,
}

/// A message to [LocalResolver]
enum ResolverMessage {
    /// Set resolver config
    SetConfig {
        /// New DNS config to use
        new_config: Config,
        /// Response channel when resolvers have been updated
        response_tx: oneshot::Sender<()>,
    },

    /// Send a DNS query to the resolver
    Query {
        dns_query: LowerQuery,

        /// Channel for the query response
        response_tx: oneshot::Sender<std::result::Result<Box<dyn LookupObject>, ResolveError>>,
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
        /// Remote DNS server to use
        dns_servers: Vec<IpAddr>,
    },
}

enum Resolver {
    /// Drop DNS queries. For captive portal domains, return faux records
    Blocking,

    /// Forward DNS queries to a configured server
    Forwarding(Box<TokioResolver>),
}

impl Resolver {
    pub fn resolve(
        &self,
        query: LowerQuery,
        tx: oneshot::Sender<std::result::Result<Box<dyn LookupObject>, ResolveError>>,
    ) {
        let lookup = match self {
            Resolver::Blocking => Either::Left(async move { Self::resolve_blocked(query) }),
            Resolver::Forwarding(resolver) => {
                Either::Right(Self::resolve_forward(resolver.as_ref().clone(), query))
            }
        };

        tokio::spawn(async move {
            let _ = tx.send(lookup.await);
        });
    }

    /// Resolution in blocked state will return spoofed records for captive portal domains.
    fn resolve_blocked(
        query: LowerQuery,
    ) -> std::result::Result<Box<dyn LookupObject>, ResolveError> {
        if !Self::is_captive_portal_domain(&query) {
            return Ok(Box::new(EmptyLookup));
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
            Arc::new([return_record]),
            Instant::now() + Duration::from_secs(3),
        );
        Ok(Box::new(ForwardLookup(lookup)) as Box<_>)
    }

    /// Determines whether a DNS query is allowable. Currently, this implies that the query is
    /// either a `A` or a `CNAME` query for `captive.apple.com`.
    fn is_captive_portal_domain(query: &LowerQuery) -> bool {
        ALLOWED_RECORD_TYPES.contains(&query.query_type()) && ALLOWED_DOMAINS.contains(query.name())
    }

    /// Forward DNS queries to the specified DNS resolver.
    async fn resolve_forward(
        resolver: TokioResolver,
        query: LowerQuery,
    ) -> std::result::Result<Box<dyn LookupObject>, ResolveError> {
        let return_query = query.original().clone();

        let lookup = resolver
            .lookup(return_query.name().clone(), return_query.query_type())
            .await;

        lookup.map(|lookup| Box::new(ForwardLookup(lookup)) as Box<_>)
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

    /// Get listening port for resolver handle
    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    /// Set the DNS server to forward queries to `dns_servers`
    pub async fn enable_forward(&self, dns_servers: Vec<IpAddr>) {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(ResolverMessage::SetConfig {
                new_config: Config::Forwarding { dns_servers },
                response_tx,
            })
            .is_ok()
        {
            response_rx.await.ok();
        };
    }

    // Disable forwarding
    pub async fn disable_forward(&self) {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(ResolverMessage::SetConfig {
                new_config: Config::Blocking,
                response_tx,
            })
            .is_ok()
        {
            response_rx.await.ok();
        }
    }
}

impl LocalResolver {
    /// Spawn new filtering resolver and it's handle.
    pub async fn spawn(
        listen_addr: SocketAddr,
        shutdown_token: CancellationToken,
    ) -> Result<(ResolverHandle, tokio::task::JoinHandle<()>), Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let resolver_socket = UdpSocket::bind(listen_addr)
            .await
            .map_err(Error::UdpBindError)?;
        let resolver_addr = resolver_socket
            .local_addr()
            .map_err(Error::GetSocketAddrError)?;

        let mut server = Self::new_server(resolver_socket, tx.clone()).await?;

        let cloned_shutdown_token = shutdown_token.child_token();
        let cloned_tx = tx.clone();
        let dns_server_task = tokio::spawn(async move {
            tracing::info!("Running DNS resolver on {resolver_addr}");

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
                                tracing::debug!("Attempting restart server");

                                let socket = match UdpSocket::bind(resolver_addr).await {
                                    Ok(socket) => socket,
                                    Err(e) => {
                                        tracing::error!("Failed to bind DNS server to {resolver_addr}: {e}");
                                        break;
                                    }
                                };

                                match Self::new_server(socket, cloned_tx.clone()).await {
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
        });

        let resolver = Self {
            rx,
            dns_server_task,
            bound_to: resolver_addr,
            inner_resolver: Resolver::Blocking,
            shutdown_token,
        };

        let join_handle = tokio::spawn(resolver.run());

        Ok((ResolverHandle::new(tx.clone(), resolver_addr), join_handle))
    }

    async fn new_server(
        server_socket: tokio::net::UdpSocket,
        tx: mpsc::UnboundedSender<ResolverMessage>,
    ) -> Result<ServerFuture<ResolverImpl>, Error> {
        let mut server = ServerFuture::new(ResolverImpl { tx });
        server.register_socket(server_socket);
        Ok(server)
    }

    /// Runs the filtering resolver as an actor, listening for new queries instances.  When all
    /// related [ResolverHandle] instances are dropped, this function will return, closing the DNS
    /// server.
    async fn run(mut self) {
        loop {
            tokio::select! {
               Some(request) = self.rx.recv() => {
                    match request {
                        ResolverMessage::SetConfig {
                            new_config,
                            response_tx,
                        } => {
                            tracing::info!("Updating config: {new_config:?}");

                            self.update_config(new_config);
                            flush_system_cache();
                            let _ = response_tx.send(());
                        }
                        ResolverMessage::Query {
                            dns_query,
                            response_tx,
                        } => {
                            self.inner_resolver.resolve(dns_query, response_tx);
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
    fn update_config(&mut self, config: Config) {
        match config {
            Config::Blocking => {
                self.blocking();
            }
            Config::Forwarding { mut dns_servers } => {
                // make sure not to accidentally forward queries to ourselves
                dns_servers.retain(|addr| *addr != self.bound_to.ip());
                self.forwarding(dns_servers);
            }
        }
    }

    /// Turn into a blocking resolver.
    fn blocking(&mut self) {
        self.inner_resolver = Resolver::Blocking;
    }

    /// Turn into a forwarding resolver (forward DNS queries to [dns_servers]).
    fn forwarding(&mut self, dns_servers: Vec<IpAddr>) {
        let forward_server_config =
            NameServerConfigGroup::from_ips_clear(&dns_servers, DNS_LISTEN_PORT, true);

        let forward_config = ResolverConfig::from_parts(None, vec![], forward_server_config);
        let resolver =
            TokioResolver::builder_with_config(forward_config, TokioConnectionProvider::default())
                .build();

        self.inner_resolver = Resolver::Forwarding(Box::new(resolver));
    }
}

/// Flush the DNS cache.
fn flush_system_cache() {
    if let Err(error) = kill_mdnsresponder() {
        tracing::error!("Failed to kill mDNSResponder: {error}");
    }
}

const MDNS_RESPONDER_PATH: &str = "/usr/sbin/mDNSResponder";

/// Find and kill mDNSResponder. The OS will restart the service.
fn kill_mdnsresponder() -> io::Result<()> {
    if let Some(mdns_pid) = nym_macos::process::pid_of_path(MDNS_RESPONDER_PATH) {
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(mdns_pid),
            nix::sys::signal::SIGHUP,
        )?;
    }
    Ok(())
}

type LookupResponse<'a> = MessageResponse<
    'a,
    'a,
    Box<dyn Iterator<Item = &'a Record> + Send + 'a>,
    std::iter::Empty<&'a Record>,
    std::iter::Empty<&'a Record>,
    std::iter::Empty<&'a Record>,
>;

/// An implementation of [hickory_server::server::RequestHandler] that forwards queries to
/// `FilteringResolver`.
struct ResolverImpl {
    tx: mpsc::UnboundedSender<ResolverMessage>,
}

impl ResolverImpl {
    fn build_response<'a>(
        message: &'a MessageRequest,
        lookup: &'a dyn LookupObject,
    ) -> LookupResponse<'a> {
        let mut response_header = Header::new();
        response_header.set_id(message.id());
        response_header.set_op_code(OpCode::Query);
        response_header.set_message_type(MessageType::Response);
        response_header.set_authoritative(false);

        MessageResponseBuilder::from_message_request(message).build(
            response_header,
            lookup.iter(),
            // forwarder responses only contain query answers, no ns,soa or additionals
            std::iter::empty(),
            std::iter::empty(),
            std::iter::empty(),
        )
    }

    /// This function is called when a DNS query is sent to the local resolver
    async fn lookup<R: ResponseHandler>(&self, message: &Request, mut response_handler: R) {
        tracing::trace!(
            "Lookup for: {}, client: {}/{}",
            message
                .queries()
                .iter()
                .map(|r| format!("{} {}", r.query_type(), r.name()))
                .collect::<Vec<_>>()
                .join(","),
            message.src(),
            message.protocol(),
        );

        let Some(query) = message.queries().first() else {
            tracing::error!("Received a message without query");
            return;
        };

        // BIND does not support multiple questions
        // See: https://stackoverflow.com/a/4083071/3042552
        if message.queries().len() > 1 {
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
            return;
        };

        let lookup_result = response_rx.await;
        let response_result = match lookup_result {
            Ok(Ok(ref lookup)) => {
                tracing::info!(
                    "Resolved to: {}",
                    lookup
                        .iter()
                        .map(|record| format!("{record:?}"))
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                let response = Self::build_response(message, lookup.as_ref());
                response_handler.send_response(response).await
            }
            Err(_error) => return,
            Ok(Err(resolve_err)) => {
                if resolve_err.is_no_records_found() {
                    let response_code = resolve_err
                        .proto()
                        .and_then(|proto_err| {
                            if let ProtoErrorKind::NoRecordsFound { response_code, .. } =
                                proto_err.kind()
                            {
                                Some(*response_code)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(ResponseCode::NoError);
                    let response = MessageResponseBuilder::from_message_request(message)
                        .error_msg(message.header(), response_code);
                    tracing::info!("Resolver error: no records found!");
                    response_handler.send_response(response).await
                } else {
                    let response = Self::build_response(message, &EmptyLookup);
                    tracing::info!("Resolver error: {resolve_err}");
                    response_handler.send_response(response).await
                }
            }
        };

        if let Err(err) = response_result {
            tracing::error!("Failed to send response: {err}");
        }
    }
}

#[async_trait::async_trait]
impl RequestHandler for ResolverImpl {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        response_handle: R,
    ) -> ResponseInfo {
        if !request.src().ip().is_loopback() {
            tracing::error!("Dropping a stray request from outside: {}", request.src());
            return Header::new().into();
        }
        if let MessageType::Query = request.message_type() {
            match request.op_code() {
                OpCode::Query => {
                    self.lookup(request, response_handle).await;
                }
                _ => {
                    tracing::trace!("Dropping non-query request: {:?}", request);
                }
            };
        }

        return Header::new().into();
    }
}

struct ForwardLookup(Lookup);

/// This trait has to be reimplemented for the Lookup so that it can be sent back to the
/// RequestHandler implementation.
impl LookupObject for ForwardLookup {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Record> + Send + 'a> {
        Box::new(self.0.record_iter())
    }

    fn take_additionals(&mut self) -> Option<Box<dyn LookupObject>> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hickory_server::resolver::{
        TokioResolver,
        config::{NameServerConfigGroup, ResolverConfig},
        name_server::TokioConnectionProvider,
    };
    use std::{
        mem,
        net::{SocketAddrV4, UdpSocket},
        time::Duration,
    };

    const LISTEN_ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));

    fn get_test_resolver(listen_addr: SocketAddr) -> TokioResolver {
        let resolver_config = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(&[listen_addr.ip()], listen_addr.port(), true),
        );
        TokioResolver::builder_with_config(resolver_config, TokioConnectionProvider::default())
            .build()
    }

    #[tokio::test]
    async fn test_successful_lookup() {
        let (handle, _join_handle) = LocalResolver::spawn(LISTEN_ADDR, CancellationToken::new())
            .await
            .unwrap();
        let test_resolver = get_test_resolver(handle.listen_addr());

        for domain in &*ALLOWED_DOMAINS {
            test_resolver
                .lookup(domain, RecordType::A)
                .await
                .expect("domain resolution failed");
        }
    }

    #[tokio::test]
    async fn test_failed_lookup() {
        let (handle, _join_handle) = LocalResolver::spawn(LISTEN_ADDR, CancellationToken::new())
            .await
            .unwrap();
        let test_resolver = get_test_resolver(handle.listen_addr());

        let captive_portal_domain = LowerName::from(Name::from_str("apple.com").unwrap());
        assert!(
            test_resolver
                .lookup(captive_portal_domain, RecordType::A)
                .await
                .is_err(),
            "Non-whitelisted DNS request should fail"
        )
    }

    #[tokio::test]
    async fn test_shutdown() {
        let (handle, _join_handle) = LocalResolver::spawn(LISTEN_ADDR, CancellationToken::new())
            .await
            .unwrap();
        let listen_addr = handle.listen_addr();
        mem::drop(handle);
        tokio::time::sleep(Duration::from_millis(300)).await;
        UdpSocket::bind(listen_addr)
            .expect("Failed to bind to a port that should have been removed");
    }
}
