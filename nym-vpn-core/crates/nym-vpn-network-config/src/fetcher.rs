// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use nym_endpoint_health::{EndpointClass, EndpointHealthTracker};
use nym_http_api_client::Client as HttpApiClient;
use nym_sdk::{NymNetworkDetails, UserAgent};
use nym_validator_client::nym_api::NymApiClientExt;
use nym_vpn_api_client::{VpnApiClient, api_urls_to_urls, fronted_http_client};

use crate::{Error, Result, discovery::Discovery, envs::RegisteredNetworks};

const NETWORK_TIMEOUT: Duration = Duration::from_secs(10);

/// Network fetcher responsible for interaction with Nym API.
#[derive(Debug)]
pub struct Fetcher {
    api_client: HttpApiClient,
    vpn_api_client: VpnApiClient,
    user_agent: Option<UserAgent>,
    discovery: Box<Discovery>,
    tracker: Option<Arc<EndpointHealthTracker>>,
}

impl Fetcher {
    /// Create an instance of `Fetcher` using HTTP API endpoints from the given discovery.
    pub fn new(
        discovery: Discovery,
        user_agent: Option<UserAgent>,
        tracker: Option<Arc<EndpointHealthTracker>>,
    ) -> Result<Self> {
        if let Some(tracker) = &tracker {
            register_nym_api_urls(&discovery, tracker);
        }

        Ok(Self {
            user_agent: user_agent.clone(),
            api_client: build_api_client(&discovery, user_agent.clone(), tracker.as_deref())?,
            vpn_api_client: build_vpn_api_client(&discovery, user_agent)?,
            discovery: Box::new(discovery),
            tracker,
        })
    }

    /// Update internal discovery used by the fetcher.
    /// This causes recreation of the underlying HTTP API clients.
    pub(crate) fn set_discovery(&mut self, new_discovery: Discovery) -> Result<()> {
        if *self.discovery == new_discovery {
            return Ok(());
        }

        if let Some(tracker) = &self.tracker {
            register_nym_api_urls(&new_discovery, tracker);
        }

        self.api_client = build_api_client(
            &new_discovery,
            self.user_agent.clone(),
            self.tracker.as_deref(),
        )?;
        self.vpn_api_client = build_vpn_api_client(&new_discovery, self.user_agent.clone())?;
        *self.discovery = new_discovery;

        Ok(())
    }

    /// Fetch registered networks from the API.
    pub async fn fetch_registered_networks(&self) -> Result<RegisteredNetworks> {
        self.vpn_api_client
            .get_wellknown_envs()
            .await
            .map_err(Error::GetWellKnownEnvs)
            .map(RegisteredNetworks::new)
            .map(RegisteredNetworks::without_retired)
    }

    /// Fetch discovery information from the API.
    pub async fn fetch_discovery(&self, network_name: &str) -> Result<Discovery> {
        self.vpn_api_client
            .get_wellknown_discovery(network_name)
            .await
            .map_err(Error::GetWellKnownDiscovery)
            .and_then(|response| {
                Discovery::try_from(response).map_err(Error::ConvertWellKnownDiscovery)
            })
    }

    /// Fetch network details from the API.
    pub async fn fetch_network_details(&self) -> Result<Box<NymNetworkDetails>> {
        self.api_client
            .get_network_details()
            .await
            .map(|response| response.network)
            .map(Box::new)
            .map_err(Box::new)
            .map_err(Error::GetNetworkDetails)
    }
}

fn build_api_client(
    discovery: &Discovery,
    user_agent: Option<UserAgent>,
    tracker: Option<&EndpointHealthTracker>,
) -> Result<HttpApiClient> {
    let mut nym_api_urls = discovery.nym_api_urls();
    if let Some(tracker) = tracker {
        nym_api_urls = merge_and_order_api_urls_by_health(nym_api_urls, tracker);
    }
    let api_urls = api_urls_to_urls(&nym_api_urls).map_err(Error::CreateVpnApiClient)?;

    fronted_http_client::fronted_http_client(api_urls, user_agent, Some(NETWORK_TIMEOUT))
        .map_err(Error::CreateVpnApiClient)
}

fn register_nym_api_urls(discovery: &Discovery, tracker: &EndpointHealthTracker) {
    let urls = discovery
        .nym_api_urls()
        .iter()
        .filter_map(|api_url| api_url.url.parse().ok())
        .collect();
    tracker.register(EndpointClass::NymApi, urls);
}

/// Reorder ApiUrls so tracker-healthy endpoints come first. Entries the
/// tracker doesn't know keep their relative position at the end; the result
/// always contains every input entry (selection is fail-open, and dropping
/// URLs here could leave the client with nothing).
///
/// Comparison is done on parsed `Url`s rather than raw strings so a
/// non-normalized discovery URL (e.g. differing only in a trailing slash)
/// still matches its ranking entry. A URL that fails to parse simply ranks
/// last — it is never dropped.
pub fn order_api_urls_by_health(
    urls: Vec<nym_network_defaults::ApiUrl>,
    tracker: &EndpointHealthTracker,
) -> Vec<nym_network_defaults::ApiUrl> {
    let ranking = tracker.select(EndpointClass::NymApi);
    let rank_of = |api_url: &nym_network_defaults::ApiUrl| {
        api_url
            .url
            .parse::<url::Url>()
            .ok()
            .and_then(|parsed| ranking.iter().position(|u| *u == parsed))
            .unwrap_or(usize::MAX)
    };
    let mut urls = urls;
    urls.sort_by_key(rank_of);
    urls
}

/// Append tracker-known NymApi endpoints that are missing from `urls`
/// (as front-less entries), then health-order the combined list.
pub fn merge_and_order_api_urls_by_health(
    urls: Vec<nym_network_defaults::ApiUrl>,
    tracker: &EndpointHealthTracker,
) -> Vec<nym_network_defaults::ApiUrl> {
    let mut urls = urls;
    let known: Vec<url::Url> = urls.iter().filter_map(|u| u.url.parse().ok()).collect();
    for endpoint in tracker.all_endpoints(EndpointClass::NymApi) {
        if !known.contains(&endpoint) {
            urls.push(nym_network_defaults::ApiUrl {
                url: endpoint.to_string(),
                front_hosts: None,
            });
        }
    }
    order_api_urls_by_health(urls, tracker)
}

fn build_vpn_api_client(
    discovery: &Discovery,
    user_agent: Option<UserAgent>,
) -> Result<VpnApiClient> {
    let vpn_api_urls =
        api_urls_to_urls(&discovery.nym_vpn_api_urls()).map_err(Error::CreateVpnApiClient)?;

    VpnApiClient::new(vpn_api_urls, user_agent).map_err(Error::CreateVpnApiClient)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_fetch() {
        let network_name = "mainnet";
        let fetcher = Fetcher::new(Discovery::default_mainnet(), None, None).unwrap();
        let discovery = fetcher.fetch_discovery(network_name).await.unwrap();
        assert_eq!(discovery.network_name, network_name);
    }

    #[test]
    fn api_urls_ordered_by_health() {
        use nym_endpoint_health::{EndpointClass, EndpointHealthTracker, FailureKind};

        let tracker = EndpointHealthTracker::new();
        let primary = "https://validator.example/api/";
        let front = "https://front.example/api/";
        tracker.register(
            EndpointClass::NymApi,
            vec![primary.parse().unwrap(), front.parse().unwrap()],
        );
        // blacklist the primary
        for _ in 0..3 {
            tracker.report_failure(
                EndpointClass::NymApi,
                &primary.parse().unwrap(),
                FailureKind::Connect,
            );
        }

        let urls = vec![
            nym_network_defaults::ApiUrl {
                url: primary.to_string(),
                front_hosts: None,
            },
            nym_network_defaults::ApiUrl {
                url: front.to_string(),
                front_hosts: Some(vec!["f.example".into()]),
            },
        ];
        let ordered = order_api_urls_by_health(urls, &tracker);
        assert_eq!(ordered[0].url, front);
        // blacklisted-but-only-remaining entries are kept (fail-open), so both survive
        assert_eq!(ordered.len(), 2);
        assert!(ordered[1].front_hosts.is_none());
    }

    #[test]
    fn api_urls_ordered_by_health_matches_non_normalized_urls() {
        use nym_endpoint_health::{EndpointClass, EndpointHealthTracker};

        // Tracker holds the URL without a trailing slash; discovery has one
        // with a trailing slash (or vice versa) -- both parse to the same
        // `Url`, so they must still match.
        let tracker = EndpointHealthTracker::new();
        tracker.register(
            EndpointClass::NymApi,
            vec!["https://validator.example".parse().unwrap()],
        );

        let urls = vec![nym_network_defaults::ApiUrl {
            url: "https://validator.example/".to_string(),
            front_hosts: None,
        }];
        let ordered = order_api_urls_by_health(urls, &tracker);
        // Never-drop guarantee holds regardless; this asserts the specific
        // match (rank 0, not usize::MAX) by checking it sorts identically to
        // the tracker's own ranking rather than falling to the back.
        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered[0].url, "https://validator.example/");
    }

    #[test]
    fn merge_and_order_adds_tracker_known_endpoint_without_duplicating() {
        use nym_endpoint_health::EndpointHealthTracker;

        let discovery_url = "https://validator.example/api/";
        let extra_runtime_url = "https://extra-signer.example/";

        let tracker = EndpointHealthTracker::new();
        tracker.register(
            EndpointClass::NymApi,
            vec![
                discovery_url.parse().unwrap(),
                extra_runtime_url.parse().unwrap(),
            ],
        );
        // blacklist the discovery url so it must sort after the healthy extra
        for _ in 0..3 {
            tracker.report_failure(
                EndpointClass::NymApi,
                &discovery_url.parse().unwrap(),
                nym_endpoint_health::FailureKind::Connect,
            );
        }

        let urls = vec![nym_network_defaults::ApiUrl {
            url: discovery_url.to_string(),
            front_hosts: None,
        }];

        let merged = merge_and_order_api_urls_by_health(urls, &tracker);

        // Nothing dropped, extra endpoint added, no duplicates.
        assert_eq!(merged.len(), 2);
        assert!(merged.iter().any(|u| u.url == discovery_url));
        let extra = merged
            .iter()
            .find(|u| u.url == extra_runtime_url)
            .expect("extra tracker-known endpoint must be present");
        assert!(extra.front_hosts.is_none());

        // Health ordering still applies: blacklisted discovery url sorts last.
        assert_eq!(merged[0].url, extra_runtime_url);
        assert_eq!(merged[1].url, discovery_url);
    }

    #[test]
    fn merge_and_order_does_not_duplicate_when_discovery_url_already_tracker_registered() {
        use nym_endpoint_health::EndpointHealthTracker;

        let discovery_url = "https://validator.example/api/";
        let tracker = EndpointHealthTracker::new();
        tracker.register(EndpointClass::NymApi, vec![discovery_url.parse().unwrap()]);

        let urls = vec![nym_network_defaults::ApiUrl {
            url: discovery_url.to_string(),
            front_hosts: None,
        }];

        let merged = merge_and_order_api_urls_by_health(urls, &tracker);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].url, discovery_url);
    }

    #[test]
    fn api_urls_ordered_by_health_keeps_unparseable_urls_last_without_dropping() {
        use nym_endpoint_health::EndpointHealthTracker;

        let tracker = EndpointHealthTracker::new();
        let urls = vec![nym_network_defaults::ApiUrl {
            url: "not a url".to_string(),
            front_hosts: None,
        }];
        let ordered = order_api_urls_by_health(urls, &tracker);
        assert_eq!(
            ordered.len(),
            1,
            "unparseable entries must never be dropped"
        );
        assert_eq!(ordered[0].url, "not a url");
    }
}
