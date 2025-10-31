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
    os::fd::AsRawFd,
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
        config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
        lookup::Lookup,
        name_server::TokioConnectionProvider,
    },
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
};
use nix::{
    fcntl,
    sys::socket::{self, AddressFamily, SockFlag, SockProtocol, SockType, SockaddrStorage},
};
use rand::Rng;
use tokio::{
    net::UdpSocket,
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::{
    either::Either,
    sync::{CancellationToken, DropGuard},
};

/// If a local DNS resolver should be used.
///
/// Local DNS resolver is used to work around Apple's captive portals check.
/// More info can be found at <https://github.com/mullvad/mullvadvpn-app/blob/main/docs/allow-macos-network-check.md>
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

/// Loopback interface name.
const LOOPBACK: &str = "lo0";

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
    UdpBind,

    /// Failed to get local address of a bound UDP socket
    #[error("failed to get local address of a bound UDP socket")]
    GetSocketAddr(#[source] io::Error),
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
        use_random_loopback: bool,
        shutdown_token: CancellationToken,
    ) -> Result<(ResolverHandle, tokio::task::JoinHandle<()>), Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let (resolver_socket, loopback_alias) =
            Self::new_random_socket(use_random_loopback).await?;
        let resolver_addr = resolver_socket.local_addr().map_err(Error::GetSocketAddr)?;

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
                                tracing::debug!("Attempting to restart server");

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

            if let Some(loopback_alias) = loopback_alias {
                loopback_alias.unassign().await;
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
                request = self.rx.recv() => {
                    match request {
                        Some(ResolverMessage::SetConfig {
                            new_config,
                            response_tx,
                        }) => {
                            tracing::info!("Updating config: {new_config:?}");

                            self.update_config(new_config);
                            flush_system_cache();
                            let _ = response_tx.send(());
                        }
                        Some(ResolverMessage::Query {
                            dns_query,
                            response_tx,
                        }) => {
                            self.inner_resolver.resolve(dns_query, response_tx);
                        }
                        None => {
                            // Channel closed, cancel server task
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

    /// Create a new [net::UdpSocket] bound to port 53 on loopback.
    ///
    /// This socket will try to bind to random ip in the range `127. 1-255. 0-255. 1-254 : 53`.
    /// After 3 failed attempts it will attempt to bind to `127.0.0.1 : 53`
    ///
    /// This is done this way to avoid collisions with other DNS servers running on the same system.
    ///
    /// If [use_random_loopback] is `false`, it will only try to bind to `127.0.0.1`.
    ///
    /// Returns `UdpSocket` and `Option<RandomLoopbackAlias>` upon success, otherwise an error.
    /// `RandomLoopbackAlias` removes the loopback alias when dropped.
    async fn new_random_socket(
        use_random_loopback: bool,
    ) -> Result<(UdpSocket, Option<RandomLoopbackAlias>), Error> {
        for attempt in 0.. {
            let (socket_addr, on_drop) = match attempt {
                ..3 if !use_random_loopback => continue,
                ..3 => match RandomLoopbackAlias::assign().await {
                    Ok(random) => (random.addr(), Some(random)),
                    Err(_) => continue,
                },
                3 => (IpAddr::from(Ipv4Addr::LOCALHOST), None),
                4.. => break,
            };

            let sock = match socket::socket(
                AddressFamily::Inet,
                SockType::Datagram,
                SockFlag::empty(),
                SockProtocol::Udp,
            ) {
                Ok(sock) => sock,
                Err(error) => {
                    tracing::error!("Failed to open IPv4/UDP socket: {error}");
                    continue;
                }
            };

            // SO_NONBLOCK is required for turning this into a tokio socket.
            if let Err(error) = fcntl::fcntl(&sock, fcntl::F_SETFL(fcntl::OFlag::O_NONBLOCK)) {
                tracing::warn!("Failed to set socket as nonblocking: {error}");
                continue;
            }

            // SO_REUSEADDR allows us to bind to `127.x.y.z` even if another socket is bound to
            // `0.0.0.0`. This can happen e.g. when macOS "Internet Sharing" is turned on.
            if let Err(error) = socket::setsockopt(&sock, socket::sockopt::ReuseAddr, &true) {
                tracing::warn!("Failed to set SO_REUSEADDR on resolver socket: {error}");
            }

            let sin = SockaddrStorage::from(SocketAddr::new(socket_addr, DNS_LISTEN_PORT));

            match socket::bind(sock.as_raw_fd(), &sin) {
                Ok(()) => {
                    let socket = UdpSocket::from_std(sock.into()).expect("socket is non-blocking");
                    return Ok((socket, on_drop));
                }
                Err(err) => tracing::warn!("Failed to bind DNS server to {socket_addr}: {err}"),
            }
        }
        Err(Error::UdpBind)
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
        let mut opts = ResolverOpts::default();
        // Set a 30 min minimum TTL for caching DNS responses.
        opts.positive_min_ttl = Some(Duration::from_secs(1800));
        let resolver =
            TokioResolver::builder_with_config(forward_config, TokioConnectionProvider::default())
                .with_options(opts)
                .build();

        self.inner_resolver = Resolver::Forwarding(Box::new(resolver));
    }
}

struct RandomLoopbackAlias {
    addr: IpAddr,
    drop_guard: DropGuard,
    unassign_task: JoinHandle<()>,
}

impl RandomLoopbackAlias {
    /// Assign a random IPv4 alias for the loopback interface.
    ///
    /// The alias is automatically removed when the struct is dropped.
    /// However it's recommended to call `unassign` to avoid race conditions.
    pub async fn assign() -> std::io::Result<Self> {
        let addr = IpAddr::from(Ipv4Addr::new(
            127,
            rand::thread_rng().gen_range(1..=255),
            rand::random(),
            // keep last octet in the range of 1-254 to avoid special addresses
            rand::thread_rng().gen_range(1..=254),
        ));

        // TODO: this command requires root privileges and will thus not work in `cargo test`.
        // This means that the tests will fall back to 127.0.0.1, and will not assert that the
        // ifconfig stuff actually works. We probably do want to test this, so what do?
        nym_macos::net::add_alias(LOOPBACK, addr)
            .await
            .inspect_err(|e| {
                tracing::warn!("Failed to add loopback {LOOPBACK} alias {addr}: {e}");
            })?;

        tracing::debug!("Created loopback address {addr}");

        let shutdown_token = CancellationToken::new();

        let child_token = shutdown_token.child_token();
        let unassign_task = tokio::task::spawn(async move {
            child_token.cancelled().await;

            tracing::debug!("Cleaning up loopback address {addr}");
            if let Err(e) = nym_macos::net::remove_alias(LOOPBACK, addr).await {
                tracing::warn!("Failed to clean up {LOOPBACK} alias {addr}: {e}");
            }
        });

        let drop_guard = shutdown_token.drop_guard();

        Ok(Self {
            addr,
            drop_guard,
            unassign_task,
        })
    }

    /// Unassign the loopback alias.
    pub async fn unassign(self) {
        // Dispose drop guard to trigger cancellation.
        drop(self.drop_guard);
        self.unassign_task.await.ok();
    }

    /// Returns loopback IP address alias.
    pub fn addr(&self) -> IpAddr {
        self.addr
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
                    response_handler.send_response(response).await
                } else {
                    let response = Self::build_response(message, &EmptyLookup);
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
    use std::{net::UdpSocket, time::Duration};

    use hickory_server::resolver::{
        TokioResolver,
        config::{NameServerConfigGroup, ResolverConfig},
        name_server::TokioConnectionProvider,
    };
    use nix::sys::socket::{
        self, AddressFamily, SockFlag, SockProtocol, SockType, SockaddrStorage, sockopt,
    };
    use tokio_util::sync::CancellationToken;

    use super::*;

    /// Test whether we can successfully bind the socket even if the address is already used to
    /// in different scenarios.
    ///
    /// # Note
    ///
    /// This test does not test aliases on lo0, as that requires root privileges.
    #[tokio::test]
    #[serial_test::serial]
    async fn test_bind() {
        // bind() succeeds if wildcard address is bound without REUSEADDR and REUSEPORT
        let _sock = bind_sock(
            BindParams::builder()
                .bind_addr(format!("0.0.0.0:{DNS_LISTEN_PORT}").parse().unwrap())
                .reuse_addr(false)
                .reuse_port(false)
                .build(),
        )
        .unwrap();

        let shutdown_token = CancellationToken::new();
        let (handle, join_handle) = LocalResolver::spawn(false, shutdown_token.child_token())
            .await
            .unwrap();
        let test_resolver = get_test_resolver(handle.listen_addr());
        test_resolver
            .lookup(&ALLOWED_DOMAINS[0], RecordType::A)
            .await
            .expect("lookup should succeed");
        drop(_sock);
        shutdown_token.cancel();
        join_handle.await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;

        // bind() succeeds if wildcard address is bound with REUSEADDR and REUSEPORT
        let _sock = bind_sock(
            BindParams::builder()
                .bind_addr(format!("0.0.0.0:{DNS_LISTEN_PORT}").parse().unwrap())
                .reuse_addr(true)
                .reuse_port(true)
                .build(),
        )
        .unwrap();

        let shutdown_token = CancellationToken::new();
        let (handle, join_handle) = LocalResolver::spawn(false, shutdown_token.child_token())
            .await
            .unwrap();
        let test_resolver = get_test_resolver(handle.listen_addr());
        test_resolver
            .lookup(&ALLOWED_DOMAINS[0], RecordType::A)
            .await
            .expect("lookup should succeed");
        drop(_sock);
        shutdown_token.cancel();
        join_handle.await.unwrap();

        // bind() should succeeds if 127.0.0.1 is already bound without REUSEADDR and REUSEPORT
        // NOTE: We cannot test this as creating an alias requires root privileges.
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_successful_lookup() {
        let shutdown_token = CancellationToken::new();
        let (handle, join_handle) = LocalResolver::spawn(false, shutdown_token.child_token())
            .await
            .unwrap();
        let test_resolver = get_test_resolver(handle.listen_addr());

        for domain in &*ALLOWED_DOMAINS {
            test_resolver
                .lookup(domain, RecordType::A)
                .await
                .expect("domain resolution failed");
        }

        shutdown_token.cancel();
        join_handle.await.unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_failed_lookup() {
        let shutdown_token = CancellationToken::new();
        let (handle, join_handle) = LocalResolver::spawn(false, shutdown_token.child_token())
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
        );
        shutdown_token.cancel();
        join_handle.await.unwrap();
    }

    /// Test that we close the socket when shutting down the local resolver.
    #[tokio::test]
    #[serial_test::serial]
    async fn test_unbind_socket_on_stop() {
        // Bind resolver to 127.0.0.1 so that we can easily bind to the same address here.
        let shutdown_token = CancellationToken::new();
        let (handle, join_handle) = LocalResolver::spawn(false, shutdown_token.child_token())
            .await
            .unwrap();
        let addr = handle.listen_addr();
        assert_eq!(
            addr,
            SocketAddr::from((Ipv4Addr::LOCALHOST, DNS_LISTEN_PORT))
        );
        shutdown_token.cancel();
        join_handle.await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        UdpSocket::bind(addr).expect("Failed to bind to a port that should have been removed");
    }

    fn get_test_resolver(listen_addr: SocketAddr) -> TokioResolver {
        let resolver_config = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(&[listen_addr.ip()], listen_addr.port(), true),
        );
        TokioResolver::builder_with_config(resolver_config, TokioConnectionProvider::default())
            .build()
    }

    #[derive(typed_builder::TypedBuilder)]
    struct BindParams {
        bind_addr: SocketAddr,
        reuse_addr: bool,
        reuse_port: bool,
        #[builder(default)]
        connect_addr: Option<SocketAddr>,
    }

    /// Helper function for creating and binding a UDP socket
    fn bind_sock(params: BindParams) -> io::Result<UdpSocket> {
        let sock = socket::socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            SockProtocol::Udp,
        )?;

        socket::setsockopt(&sock, sockopt::ReuseAddr, &params.reuse_addr)?;
        socket::setsockopt(&sock, sockopt::ReusePort, &params.reuse_port)?;

        socket::bind(sock.as_raw_fd(), &SockaddrStorage::from(params.bind_addr))?;

        if let Some(connect_addr) = params.connect_addr.map(SockaddrStorage::from) {
            socket::connect(sock.as_raw_fd(), &connect_addr)?;
        }

        println!(
            "Bound to {} (reuseport: {}, reuseaddr: {})",
            params.bind_addr, params.reuse_port, params.reuse_addr
        );
        Ok(UdpSocket::from(sock))
    }
}
