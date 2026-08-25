// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{cmp, sync::Arc, time::Duration};

use geo::{Distance, Haversine, Point};
use nym_gateway_directory::{Gateway, Location};
use nym_vpn_api_client::{
    VpnApiClient, error::VpnApiClientError, response::NymUserGeoIpLocationResponse,
};
use tokio::sync::{
    RwLock,
    mpsc::{self, UnboundedReceiver},
    oneshot,
};
use tokio_util::sync::CancellationToken;

const GEO_IP_UPDATE_INTERVAL: Duration = Duration::from_hours(1);
const TIMEOUT_INITIAL_LOCATION: Duration = Duration::from_secs(5);

#[async_trait::async_trait]
pub trait GeoIpClient: Send + Sync + 'static {
    async fn latest_geo_ip(&self) -> Result<NymUserGeoIpLocationResponse, VpnApiClientError>;
}

#[async_trait::async_trait]
impl GeoIpClient for VpnApiClient {
    async fn latest_geo_ip(&self) -> Result<NymUserGeoIpLocationResponse, VpnApiClientError> {
        self.get_geo_ip().await
    }
}

fn geo_distance(x: &Location, y: &Location) -> f64 {
    let p1 = Point::new(x.longitude, x.latitude);
    let p2 = Point::new(y.longitude, y.latitude);
    Haversine.distance(p1, p2)
}

/// Groups of ISO country codes that are treated as a single jurisdiction for
/// gateway-safety purposes, beyond an exact country-code match. Membership is
/// symmetric: for any two codes in the same group, neither is offered as an
/// entry or exit relative to the other (or to a user located in the group).
const JURISDICTION_GROUPS: &[&[&str]] = &[
    // Greater China: mainland China, Taiwan, Macau.
    &["CN", "TW", "MO"],
];

/// Returns true if two ISO country codes belong to the same safety jurisdiction
/// group. Exact-match equality is handled separately by [`same_jurisdiction`].
fn same_jurisdiction_group(a: &str, b: &str) -> bool {
    JURISDICTION_GROUPS
        .iter()
        .any(|group| group.contains(&a) && group.contains(&b))
}

pub(crate) fn same_jurisdiction(x: &Location, y: &Location) -> bool {
    let (a, b) = (
        x.two_letter_iso_country_code.as_str(),
        y.two_letter_iso_country_code.as_str(),
    );
    if a == b {
        // US gateways are distinguished per-region rather than per-country.
        return a != "US" || x.region == y.region;
    }
    same_jurisdiction_group(a, b)
}

// Compare two gateways' distance to a given reference point
// cmp::Ordering::Greater means gw1 is farther than gw2
// cmp::Ordering::Less means gw1 is closest than gw2
pub(crate) fn closest_gateway(reference: &Location, gw1: &Gateway, gw2: &Gateway) -> cmp::Ordering {
    match (gw1.location.clone(), gw2.location.clone()) {
        (None, None) => cmp::Ordering::Equal,
        (None, Some(_)) => cmp::Ordering::Greater,
        (Some(_), None) => cmp::Ordering::Less,
        (Some(loc1), Some(loc2)) => {
            geo_distance(reference, &loc1).total_cmp(&geo_distance(reference, &loc2))
        }
    }
}

pub(crate) struct QueryControl {
    /// if geo location should ever be used by the algorithm, depending on user's preference
    enabled: bool,
    /// if geo location is active, depending on connection status, so that false locations
    /// don't get used
    active: bool,
}

impl Default for QueryControl {
    fn default() -> Self {
        Self {
            enabled: true,
            active: true,
        }
    }
}

impl QueryControl {
    pub(crate) fn new(enabled: bool) -> Self {
        Self {
            enabled,
            active: enabled,
        }
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub(crate) fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub(crate) fn do_not_query(&self) -> bool {
        !self.enabled || !self.active
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum State {
    Nothing,
    FetchInProgress,
}

pub(crate) enum FetcherCommand {
    Fetch,
    Abort(oneshot::Sender<()>),
}

pub(crate) struct GeoIpFetcher {
    state: State,
    query_control: Arc<RwLock<QueryControl>>,
    client: Box<dyn GeoIpClient>,
    command_rx: mpsc::UnboundedReceiver<FetcherCommand>,
    update_location_tx: mpsc::UnboundedSender<Option<Location>>,
    shutdown_token: CancellationToken,
}

impl GeoIpFetcher {
    pub(crate) fn new(
        enable_geo_location: bool,
        client: Box<dyn GeoIpClient>,
        command_rx: mpsc::UnboundedReceiver<FetcherCommand>,
        update_location_tx: mpsc::UnboundedSender<Option<Location>>,
        shutdown_token: CancellationToken,
    ) -> Self {
        let state = if enable_geo_location {
            State::FetchInProgress
        } else {
            State::Nothing
        };
        let query_control = Arc::new(RwLock::new(QueryControl::new(enable_geo_location)));

        Self {
            state,
            query_control,
            client,
            command_rx,
            update_location_tx,
            shutdown_token,
        }
    }

    pub(crate) fn query_control(&self) -> Arc<RwLock<QueryControl>> {
        self.query_control.clone()
    }

    async fn maybe_start_fetching(&mut self) {
        if !self.query_control.read().await.do_not_query() {
            self.state = State::FetchInProgress;
        }
    }

    pub(crate) async fn run(mut self) {
        let update_timer = tokio::time::sleep(GEO_IP_UPDATE_INTERVAL);
        tokio::pin!(update_timer);
        loop {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    tracing::debug!("GeoIpFetcher shut down");
                    return;
                }
                _ = &mut update_timer => {
                    self.maybe_start_fetching().await;
                    update_timer.set(tokio::time::sleep(GEO_IP_UPDATE_INTERVAL));
                }
                ret = self.client.latest_geo_ip(), if self.state == State::FetchInProgress => {
                    self.state = State::Nothing;
                    match ret {
                        Ok(geo_ip_location) => {
                            let Ok(location) = geo_ip_location.location.try_into() else {
                                tracing::warn!("Failed to convert geo ip location response into location");
                                let _ = self.update_location_tx.send(None);
                                continue;
                            };
                            let _ = self.update_location_tx.send(Some(location));
                        }
                        Err(err) => {
                            let _ = self.update_location_tx.send(None);
                            tracing::warn!("Failed to query VPN API: {err:?}");
                        }
                    }
                }
                Some(command) = self.command_rx.recv() => {
                    match command {
                        FetcherCommand::Fetch => self.maybe_start_fetching().await,
                        FetcherCommand::Abort(done) => {
                            self.state = State::Nothing;
                            let _ = done.send(());
                        }
                    }
                }
            }
        }
    }
}

pub(crate) struct GeoIpProvider {
    update_location_rx: UnboundedReceiver<Option<Location>>,
    latest_known_location: Option<Location>,
}

impl GeoIpProvider {
    pub(crate) fn new(update_location_rx: UnboundedReceiver<Option<Location>>) -> Self {
        Self {
            update_location_rx,
            latest_known_location: None,
        }
    }

    /// Get the initial location, or timeout early to not disrupt too much the connecting phase.
    pub(crate) async fn initial_location(&mut self) -> Option<Location> {
        self.latest_known_location =
            tokio::time::timeout(TIMEOUT_INITIAL_LOCATION, self.update_location_rx.recv())
                .await
                .inspect_err(|_| {
                    tracing::warn!(
                        "No location for {} seconds, considering random location",
                        TIMEOUT_INITIAL_LOCATION.as_secs()
                    )
                })
                .ok()??;
        self.latest_known_location.clone()
    }

    /// Return whenever there is a new location available, different to what we've already returned
    /// previously.
    pub(crate) async fn new_location(&mut self) -> Option<Location> {
        loop {
            // if recv() returns None, there will never be a new location because the fetcher is gone
            // So we should return from the loop
            let Some(latest_location) = self.update_location_rx.recv().await? else {
                tracing::debug!(
                    "Received empty location, because of an API error, not updating it as new location"
                );
                continue;
            };
            if self.latest_known_location.as_ref() != Some(&latest_location) {
                self.latest_known_location = Some(latest_location);
                return self.latest_known_location.clone();
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn greater_china_shares_jurisdiction() {
        let group = ["CN", "TW", "MO"];
        // Every pair within the group shares a jurisdiction (symmetric),
        // including each code with itself.
        for a in group {
            for b in group {
                assert!(
                    same_jurisdiction_group(a, b),
                    "{a}/{b} should share a jurisdiction",
                );
            }
        }
        // Codes outside the group are not affected (Hong Kong is intentionally excluded).
        assert!(!same_jurisdiction_group("HK", "CN"));
        assert!(!same_jurisdiction_group("HK", "TW"));
        assert!(!same_jurisdiction_group("CN", "US"));
        assert!(!same_jurisdiction_group("TW", "JP"));
        assert!(!same_jurisdiction_group("US", "GB"));
        assert!(!same_jurisdiction_group("DE", "FR"));
    }

    #[derive(Clone)]
    pub struct MockGeoIpClient {}

    impl MockGeoIpClient {
        pub fn new() -> Self {
            Self {}
        }
    }

    #[async_trait::async_trait]
    impl GeoIpClient for MockGeoIpClient {
        async fn latest_geo_ip(&self) -> Result<NymUserGeoIpLocationResponse, VpnApiClientError> {
            Ok(NymUserGeoIpLocationResponse {
                ip: "127.0.0.1".to_string(),
                location: nym_vpn_api_client::response::Location {
                    two_letter_iso_country_code: "XX".to_string(),
                    latitude: 0f64,
                    longitude: 0f64,
                    city: "Mixnode".to_string(),
                    region: "Mixnet".to_string(),
                    asn: None,
                },
            })
        }
    }
}
