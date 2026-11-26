// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use hickory_resolver::config::{CLOUDFLARE, NameServerConfig, QUAD9};
use nym_vpn_lib_types::DefaultDnsSettings;

const QUAD9_UNAVAILABLE_COUNTRY: &str = "FR";

/// Default DNS servers producer
#[derive(Debug, Clone)]
pub struct DefaultDnsServers {
    inner: Arc<Mutex<Inner>>,
}

impl DefaultDnsServers {
    pub fn new(is_quad9_available: bool) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new(is_quad9_available))),
        }
    }

    /// Update Quad9 availability based on default DNS settings, user and exit gateway countries
    /// (two-letter ISO codes). The last known user country is used when `user_country` is `None`.
    /// Returns `true` if the DNS servers changed.
    pub fn update(
        &self,
        settings: &DefaultDnsSettings,
        user_country: Option<&str>,
        exit_country: Option<&str>,
    ) -> bool {
        let mut inner = self.lock();
        // Cache last known user country, because it can be discarded by `GeoIpFetcher` in case of error.
        if let Some(user_country) = user_country {
            inner.last_known_user_country = Some(user_country.to_owned());
        }

        let is_quad9_available = settings.enable_quad9
            && inner.last_known_user_country.as_deref() != Some(QUAD9_UNAVAILABLE_COUNTRY)
            && exit_country != Some(QUAD9_UNAVAILABLE_COUNTRY);

        inner.set_quad9_available(is_quad9_available)
    }

    pub fn get_dns_servers(&self) -> Vec<NameServerConfig> {
        self.lock().name_servers.clone()
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[derive(Debug)]
struct Inner {
    is_quad9_available: bool,
    name_servers: Vec<NameServerConfig>,
    last_known_user_country: Option<String>,
}

impl Inner {
    fn new(is_quad9_available: bool) -> Self {
        let inner = Self {
            is_quad9_available,
            name_servers: Self::make_name_servers(is_quad9_available),
            last_known_user_country: None,
        };
        inner.log_quad9_enabled();
        inner
    }

    fn set_quad9_available(&mut self, is_quad9_available: bool) -> bool {
        if self.is_quad9_available == is_quad9_available {
            false
        } else {
            self.is_quad9_available = is_quad9_available;
            self.name_servers = Self::make_name_servers(is_quad9_available);
            self.log_quad9_enabled();
            true
        }
    }

    fn make_name_servers(is_quad9_available: bool) -> Vec<NameServerConfig> {
        // Quad9 is not available in France
        let servers = is_quad9_available
            .then(|| QUAD9.tls().chain(QUAD9.https()))
            .into_iter()
            .flatten();

        servers
            .chain(CLOUDFLARE.tls())
            .chain(CLOUDFLARE.https())
            .filter(|ns| {
                // Exclude IPv6 addresses due to reliability issues
                ns.ip.is_ipv4()
            })
            .collect::<Vec<_>>()
    }

    fn log_quad9_enabled(&self) {
        tracing::info!("Enable quad9 servers: {}", self.is_quad9_available);
    }
}
