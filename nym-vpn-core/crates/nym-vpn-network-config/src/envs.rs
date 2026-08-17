// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashSet, fmt};

use itertools::Itertools;

use crate::Result;

static DEFAULT_ENVS_JSON: &[u8] = include_bytes!("../default/envs.json");

/// Retired network names that may still appear in wellknown envs until the VPN API deploys.
const RETIRED_NETWORKS: &[&str] = &["evil"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredNetworks {
    inner: HashSet<String>,
}

impl RegisteredNetworks {
    pub(crate) fn new(networks: HashSet<String>) -> Self {
        RegisteredNetworks { inner: networks }
    }

    pub fn names(&self) -> &HashSet<String> {
        &self.inner
    }

    pub(crate) fn without_retired(mut self) -> Self {
        for name in RETIRED_NETWORKS {
            self.inner.remove(*name);
        }
        self
    }
}

impl<'de> serde::de::Deserialize<'de> for RegisteredNetworks {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let registered: HashSet<String> = serde::de::Deserialize::deserialize(deserializer)?;
        Ok(RegisteredNetworks { inner: registered })
    }
}

impl serde::ser::Serialize for RegisteredNetworks {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl fmt::Display for RegisteredNetworks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner.iter().format(", "))
    }
}

impl Default for RegisteredNetworks {
    fn default() -> Self {
        #[allow(clippy::expect_used)]
        serde_json::from_slice(DEFAULT_ENVS_JSON).expect("Failed to parse default envs JSON")
    }
}

#[cfg(test)]
mod tests {
    use crate::{discovery::Discovery, fetcher::Fetcher};

    use super::*;

    #[test]
    fn test_registered_networks_serialization() {
        let registered_networks = RegisteredNetworks {
            inner: vec!["mainnet".to_string(), "testnet".to_string()]
                .into_iter()
                .collect(),
        };

        let serialized = serde_json::to_string(&registered_networks).unwrap();
        let deserialized: RegisteredNetworks = serde_json::from_str(&serialized).unwrap();

        assert_eq!(registered_networks, deserialized);
    }

    #[test]
    fn test_registered_networks_default() {
        let registered_networks = RegisteredNetworks::default();
        assert!(registered_networks.inner.contains("mainnet"));
        assert!(!registered_networks.inner.contains("evil"));
    }

    #[test]
    fn test_without_retired_drops_evil() {
        let networks =
            RegisteredNetworks::new(HashSet::from(["mainnet".to_string(), "evil".to_string()]));
        let filtered = networks.without_retired();
        assert_eq!(filtered.inner, HashSet::from(["mainnet".to_string()]));
    }

    #[tokio::test]
    async fn test_envs_default_same_as_fetched() {
        let fetcher = Fetcher::new(Discovery::default_mainnet(), None, None).unwrap();
        let default_envs = RegisteredNetworks::default();
        let fetched_envs = fetcher.fetch_registered_networks().await.unwrap();

        assert_eq!(default_envs, fetched_envs);
    }
}
