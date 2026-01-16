// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_http_api_client::ResolveError;
use nym_vpn_lib_types::{CompleteDnsReport, DnsResolution};
use nym_vpn_network_config::Network;

use hickory_resolver::{
    Resolver, ResolverBuilder,
    config::{NameServerConfig, NameServerConfigGroup, ResolverConfig, ResolverOpts},
    name_server::TokioConnectionProvider,
};
use std::{
    iter,
    net::IpAddr,
    time::{Duration, Instant},
};

pub struct DnsDiagnostic {
    hostnames: Vec<String>,
}

impl DnsDiagnostic {
    fn system() -> Result<Resolver<TokioConnectionProvider>, ResolveError> {
        Ok(Self::build_resolver(Resolver::builder_tokio()?))
    }

    fn from_nameservers<G: Into<NameServerConfigGroup>>(
        nameservers: G,
    ) -> Resolver<TokioConnectionProvider> {
        let nameservers: NameServerConfigGroup = nameservers.into();
        let config = ResolverConfig::from_parts(None, Vec::new(), nameservers);
        Self::from_config(config)
    }

    fn from_config(config: ResolverConfig) -> Resolver<TokioConnectionProvider> {
        Self::build_resolver(Resolver::builder_with_config(
            config,
            TokioConnectionProvider::default(),
        ))
    }

    fn build_resolver(
        base: ResolverBuilder<TokioConnectionProvider>,
    ) -> Resolver<TokioConnectionProvider> {
        let mut options = ResolverOpts::default();
        options.attempts = 0;
        options.cache_size = 0;
        options.ip_strategy = hickory_resolver::config::LookupIpStrategy::Ipv4AndIpv6;
        options.timeout = Duration::from_secs(2);
        base.with_options(options).build()
    }

    pub async fn run_diagnostic(network: &Network) -> CompleteDnsReport {
        tracing::info!("Running DNS diagnostic");

        let many_diagnostic = DnsDiagnostic {
            hostnames: hostnames(network),
        };

        tracing::debug!(
            "Running system DNS diagnostic on: {:?}",
            many_diagnostic.hostnames
        );

        tracing::debug!("System DNS diagnostic");
        let system_resolver = DnsDiagnostic::system();
        let system = match system_resolver {
            Ok(resolver) => Ok(many_diagnostic.resolve(&resolver).await),
            Err(e) => Err(e),
        }
        .into();

        let single_hostname = many_diagnostic.hostnames[0].clone();
        let ns_diagnostic = DnsDiagnostic {
            hostnames: vec![single_hostname],
        };

        tracing::debug!(
            "Running per ns DNS diagnostic on: {:?}",
            many_diagnostic.hostnames
        );

        let mut name_servers = NameServerConfigGroup::quad9_tls();
        name_servers.merge(NameServerConfigGroup::quad9());
        name_servers.merge(NameServerConfigGroup::quad9_https());
        name_servers.merge(NameServerConfigGroup::cloudflare_tls());
        name_servers.merge(NameServerConfigGroup::cloudflare());
        name_servers.merge(NameServerConfigGroup::cloudflare_https());

        let mut results = Vec::new();
        for nameserver in name_servers.into_inner().into_iter() {
            tracing::debug!("DNs diagnostic - {nameserver:?}");
            let resolver = DnsDiagnostic::from_nameservers(vec![nameserver]);
            results.append(&mut ns_diagnostic.resolve(&resolver).await);
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
                .map(|h| Self::dns_resolution(h, dns_resolver)),
        )
        .await
    }

    async fn dns_resolution(hostname: &str, dns_resolver: &impl DnsResolver) -> DnsResolution {
        let now = Instant::now();
        let resolution = dns_resolver.resolve(hostname).await;
        let resolution_duration_ms = now.elapsed().as_millis();

        DnsResolution {
            nameservers: format!("{:?}", dns_resolver.nameservers()),
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
    async fn resolve(&self, hostname: &str) -> Result<Vec<IpAddr>, ResolveError>;

    fn nameservers(&self) -> Vec<NameServerConfig>;
}

#[async_trait::async_trait]
impl DnsResolver for Resolver<TokioConnectionProvider> {
    async fn resolve(&self, hostname: &str) -> Result<Vec<IpAddr>, ResolveError> {
        Ok(self.lookup_ip(hostname).await?.iter().collect())
    }

    fn nameservers(&self) -> Vec<NameServerConfig> {
        self.config().name_servers().to_vec()
    }
}
