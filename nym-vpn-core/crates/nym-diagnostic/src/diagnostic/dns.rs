// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_http_api_client::{HickoryDnsResolver, ResolveError};
use nym_vpn_lib_types::{CompleteDnsReport, DnsResolution};
use nym_vpn_network_config::Network;

use hickory_resolver::{
    Resolver, ResolverBuilder,
    config::{ResolverConfig, ResolverOpts},
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

        let dns_diagnostic = DnsDiagnostic {
            hostnames: hostnames(network),
        };

        tracing::debug!("Running DNS diagnostic on: {:?}", dns_diagnostic.hostnames);

        tracing::debug!("System DNS diagnostic");
        let system_resolver = DnsDiagnostic::system();
        let system = match system_resolver {
            Ok(resolver) => Ok(dns_diagnostic.resolve(&resolver).await),
            Err(e) => Err(e),
        }
        .into();

        tracing::debug!("Quad9 DNS diagnostic");
        let quad9_resolver = DnsDiagnostic::from_config(ResolverConfig::quad9());
        let quad9 = dns_diagnostic.resolve(&quad9_resolver).await;

        tracing::debug!("Quad9 DoH diagnostic");
        let quad9_doh_resolver = DnsDiagnostic::from_config(ResolverConfig::quad9_https());
        let quad9_doh = dns_diagnostic.resolve(&quad9_doh_resolver).await;

        tracing::debug!("Quad9 DoT diagnostic");
        let quad9_dot_resolver = DnsDiagnostic::from_config(ResolverConfig::quad9_tls());
        let quad9_dot = dns_diagnostic.resolve(&quad9_dot_resolver).await;

        tracing::debug!("CloudFlare DNS diagnostic");
        let cloudflare_resolver = DnsDiagnostic::from_config(ResolverConfig::cloudflare());
        let cloudflare = dns_diagnostic.resolve(&cloudflare_resolver).await;

        tracing::debug!("CloudFlare DoH diagnostic");
        let cloudflare_doh_resolver =
            DnsDiagnostic::from_config(ResolverConfig::cloudflare_https());
        let cloudflare_doh = dns_diagnostic.resolve(&cloudflare_doh_resolver).await;

        tracing::debug!("CloudFlare DoT diagnostic");
        let cloudflare_dot_resolver = DnsDiagnostic::from_config(ResolverConfig::cloudflare_tls());
        let cloudflare_dot = dns_diagnostic.resolve(&cloudflare_dot_resolver).await;

        tracing::debug!("Nym custom DNS diagnostic");
        let mut nym_resolver = HickoryDnsResolver::default();
        nym_resolver.disable_system_fallback();
        nym_resolver.set_static_fallbacks(Default::default());
        let nym = dns_diagnostic.resolve(&nym_resolver).await;

        CompleteDnsReport {
            system,
            quad9,
            quad9_doh,
            quad9_dot,
            cloudflare,
            cloudflare_doh,
            cloudflare_dot,
            nym,
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
}

#[async_trait::async_trait]
impl DnsResolver for HickoryDnsResolver {
    async fn resolve(&self, hostname: &str) -> Result<Vec<IpAddr>, ResolveError> {
        Ok(self.resolve_str(hostname).await?.collect())
    }
}

#[async_trait::async_trait]
impl DnsResolver for Resolver<TokioConnectionProvider> {
    async fn resolve(&self, hostname: &str) -> Result<Vec<IpAddr>, ResolveError> {
        Ok(self.lookup_ip(hostname).await?.iter().collect())
    }
}
