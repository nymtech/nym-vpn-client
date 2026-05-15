// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{CompleteDnsReport, DiagnosticResult, DnsResolution};
use nym_vpn_network_config::Network;

use hickory_resolver::{
    Resolver, ResolverBuilder, TokioResolver,
    config::{CLOUDFLARE, NameServerConfig, QUAD9, ResolverConfig, ResolverOpts},
    net::runtime::TokioRuntimeProvider,
};
use std::{
    iter,
    net::IpAddr,
    time::{Duration, Instant},
};

#[derive(thiserror::Error, Debug)]
pub enum DnsDiagnosticError {
    #[error("resolve error: {0}")]
    ResolveError(#[from] hickory_resolver::net::NetError),
}

pub struct DnsDiagnostic {
    hostnames: Vec<String>,
    nameservers: Vec<NameServerConfig>,
}

impl DnsDiagnostic {
    #[cfg(not(target_os = "android"))]
    fn system() -> Result<TokioResolver, DnsDiagnosticError> {
        Self::build_resolver(Resolver::builder_tokio()?)
    }

    fn from_nameservers(
        nameservers: Vec<NameServerConfig>,
    ) -> Result<TokioResolver, DnsDiagnosticError> {
        let config = ResolverConfig::from_parts(None, Vec::new(), nameservers);
        Self::from_config(config)
    }

    fn from_config(config: ResolverConfig) -> Result<TokioResolver, DnsDiagnosticError> {
        Self::build_resolver(Resolver::builder_with_config(
            config,
            TokioRuntimeProvider::default(),
        ))
    }

    fn build_resolver(
        base: ResolverBuilder<TokioRuntimeProvider>,
    ) -> Result<TokioResolver, DnsDiagnosticError> {
        let mut options = ResolverOpts::default();
        options.attempts = 0;
        options.cache_size = 0;
        options.ip_strategy = hickory_resolver::config::LookupIpStrategy::Ipv4AndIpv6;
        options.timeout = Duration::from_secs(2);
        Ok(base.with_options(options).build()?)
    }

    pub async fn run_diagnostic(network: &Network) -> CompleteDnsReport {
        tracing::info!("Running DNS diagnostic");

        #[cfg(not(target_os = "android"))]
        let system_dns = hickory_resolver::system_conf::read_system_conf()
            .inspect_err(|err| {
                tracing::warn!("Failed to obtain system dns config: {err}");
            })
            .map(|(resolver_config, _opts)| resolver_config.name_servers)
            .unwrap_or_default();
        // Hickory fails due to missing JavaVM context
        // See: https://github.com/hickory-dns/hickory-dns/issues/3625
        // TODO: FIXME
        #[cfg(target_os = "android")]
        let system_dns = vec![];

        let many_diagnostic = DnsDiagnostic {
            hostnames: hostnames(network),
            nameservers: system_dns.clone(),
        };

        tracing::debug!(
            "Running system DNS diagnostic on: {:?}",
            many_diagnostic.hostnames
        );

        tracing::debug!("System DNS diagnostic");
        #[cfg(not(target_os = "android"))]
        let system = match DnsDiagnostic::system() {
            Ok(resolver) => Ok(many_diagnostic.resolve(&resolver).await),
            Err(e) => Err(e),
        }
        .into();
        // Hickory fails due to missing JavaVM context
        // See: https://github.com/hickory-dns/hickory-dns/issues/3625
        // TODO: FIXME
        #[cfg(target_os = "android")]
        let system = DiagnosticResult::from_err("hickory system resolver not working currently");

        let single_hostname = many_diagnostic.hostnames[0].clone();
        let ns_diagnostic = DnsDiagnostic {
            hostnames: vec![single_hostname],
            nameservers: system_dns,
        };

        tracing::debug!(
            "Running per ns DNS diagnostic on: {:?}",
            many_diagnostic.hostnames
        );

        let name_servers = QUAD9
            .tls()
            .chain(QUAD9.udp_and_tcp())
            .chain(QUAD9.https())
            .chain(CLOUDFLARE.tls())
            .chain(CLOUDFLARE.udp_and_tcp())
            .chain(CLOUDFLARE.https())
            .collect::<Vec<_>>();

        let mut results = Vec::new();
        for nameserver in name_servers.into_iter() {
            tracing::debug!("DNS diagnostic - {nameserver:?}");
            let res = match DnsDiagnostic::from_nameservers(vec![nameserver]) {
                Ok(resolver) => {
                    DiagnosticResult::from_value(ns_diagnostic.resolve(&resolver).await)
                }
                Err(e) => DiagnosticResult::from_err(e),
            };
            results.push(res);
        }

        CompleteDnsReport {
            system,
            by_nameserver: results,
        }
    }

    async fn resolve(&self, dns_resolver: &impl DnsResolver) -> Vec<DnsResolution> {
        futures::future::join_all(
            self.hostnames
                .iter()
                .map(|h| self.dns_resolution(h, dns_resolver)),
        )
        .await
    }

    async fn dns_resolution(
        &self,
        hostname: &str,
        dns_resolver: &impl DnsResolver,
    ) -> DnsResolution {
        let now = Instant::now();
        let resolution = dns_resolver.resolve(hostname).await;
        let resolution_duration_ms = now.elapsed().as_millis();

        DnsResolution {
            nameservers: format!("{:?}", self.nameservers),
            hostname: hostname.into(),
            resolution: resolution.into(),
            resolution_duration_ms,
        }
    }
}

pub fn hostnames(network: &Network) -> Vec<String> {
    let api_urls = network
        .nym_api_urls_as_urls()
        .into_iter()
        .chain(network.nym_vpn_api_urls_as_urls())
        .flatten()
        .chain(iter::once(network.nyxd_url.clone()));

    // Convert str urls to hostnames
    api_urls
        .filter_map(|url| match url.host_str() {
            Some(host) => Some(host.to_string()),
            None => {
                tracing::warn!("URL has no host component: {}", url);
                None
            }
        })
        .collect()
}

// QoL trait to accommodate both our custom resolver and hickory ones
#[async_trait::async_trait]
trait DnsResolver {
    async fn resolve(&self, hostname: &str) -> Result<Vec<IpAddr>, DnsDiagnosticError>;
}

#[async_trait::async_trait]
impl DnsResolver for TokioResolver {
    async fn resolve(&self, hostname: &str) -> Result<Vec<IpAddr>, DnsDiagnosticError> {
        Ok(self.lookup_ip(hostname).await?.iter().collect())
    }
}
