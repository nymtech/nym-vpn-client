// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{cmp, sync::Arc};

use geo::{Distance, Haversine, Point};
use nym_gateway_directory::{Gateway, Location};
use nym_vpn_api_client::{
    VpnApiClient, error::VpnApiClientError, response::NymUserGeoIpLocationResponse,
};
use tokio::sync::Mutex;

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

pub(crate) fn same_jurisdiction(x: &Location, y: &Location) -> bool {
    if x.two_letter_iso_country_code == y.two_letter_iso_country_code
        && x.two_letter_iso_country_code == "US"
    {
        return x.region == y.region;
    }
    x.two_letter_iso_country_code == y.two_letter_iso_country_code
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

    fn can_query(&self) -> bool {
        self.enabled && self.active
    }

    fn should_clear_location(&self) -> bool {
        !self.enabled
    }
}

pub(crate) struct GeoIpProvider {
    client: Box<dyn GeoIpClient>,
    query_control: Arc<Mutex<QueryControl>>,
    latest_location: Option<Location>,
}

impl GeoIpProvider {
    pub(crate) async fn new(
        client: impl GeoIpClient,
        query_control: Arc<Mutex<QueryControl>>,
    ) -> Self {
        let can_query = query_control.lock().await.can_query();
        let latest_location = if can_query {
            client
                .latest_geo_ip()
                .await
                .inspect_err(|err| tracing::warn!("Failed to query VPN API: {err:?}"))
                .map(|ret| ret.into())
                .ok()
        } else {
            None
        };
        Self {
            client: Box::new(client),
            query_control,
            latest_location,
        }
    }

    pub(crate) async fn update(&mut self) -> Result<(), VpnApiClientError> {
        let (should_clear_location, can_query) = {
            let control = self.query_control.lock().await;
            (control.should_clear_location(), control.can_query())
        };
        if should_clear_location {
            self.latest_location = None;
        } else if can_query {
            self.latest_location = Some(self.client.latest_geo_ip().await?.into());
        }
        Ok(())
    }

    pub(crate) fn latest_location(&mut self) -> Option<Location> {
        self.latest_location.clone()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

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
