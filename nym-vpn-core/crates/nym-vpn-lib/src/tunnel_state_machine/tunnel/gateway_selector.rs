// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_crypto::asymmetric::x25519::KeyPair;
use nym_gateway_directory::{
    EntryPoint, ExitPoint, Gateway, GatewayCacheHandle, GatewayList, GatewayType, ScoreValue,
};
use nym_vpn_store::keys::wireguard::{WireguardKeyStore, WireguardKeysDb};

use crate::{
    GatewayDirectoryError,
    tunnel_state_machine::{TunnelSettings, TunnelType},
};

#[derive(Clone)]
pub struct GatewayWithKeys {
    gateway: Gateway,
    keys: Arc<KeyPair>,
}

impl std::fmt::Debug for GatewayWithKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewayWithKeys")
            .field("gateway", &self.gateway)
            .field("client_wireguard_public_key", &self.keys.public_key())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct SelectedGateways {
    entry: Box<GatewayWithKeys>,
    exit: Box<GatewayWithKeys>,
}

impl SelectedGateways {
    pub fn entry_gateway(&self) -> &Gateway {
        &self.entry.gateway
    }

    pub fn exit_gateway(&self) -> &Gateway {
        &self.exit.gateway
    }

    pub fn entry_keypair(&self) -> &Arc<KeyPair> {
        &self.entry.keys
    }

    pub fn exit_keypair(&self) -> &Arc<KeyPair> {
        &self.exit.keys
    }
}

pub async fn select_gateways(
    gateway_cache_handle: GatewayCacheHandle,
    tunnel_settings: &TunnelSettings,
    wg_keys_db: WireguardKeysDb,
) -> Result<SelectedGateways, GatewayDirectoryError> {
    // The set of exit gateways is smaller than the set of entry gateways, so we start by selecting
    // the exit gateway and then filter out the exit gateway from the set of entry gateways.

    let entry_point = EntryPoint::from(*tunnel_settings.entry_point.clone());
    let exit_point = ExitPoint::from(*tunnel_settings.exit_point.clone());

    if let (
        EntryPoint::Gateway {
            identity: entry_identity,
        },
        ExitPoint::Gateway {
            identity: exit_identity,
        },
    ) = (&entry_point, &exit_point)
        && entry_identity == exit_identity
    {
        return Err(GatewayDirectoryError::SameEntryAndExitGateway {
            identity: entry_identity.to_string(),
        });
    };

    let (mut entry_gateways, exit_gateways) = match tunnel_settings.tunnel_type {
        TunnelType::Wireguard => {
            let all_gateways = gateway_cache_handle
                .lookup_gateways(GatewayType::Wg)
                .await
                .map_err(GatewayDirectoryError::LookupGateways)?;

            let entry_gateways = if tunnel_settings.bridges_enabled() {
                GatewayList::new(
                    all_gateways.gw_type(),
                    all_gateways
                        .clone()
                        .into_iter()
                        .filter(|gw| gw.bridge_params.is_some())
                        .collect(),
                )
            } else {
                all_gateways.clone()
            };

            (entry_gateways, all_gateways)
        }
        TunnelType::Mixnet => {
            // Setup the gateway that we will use as the exit point
            let exit_gateways = gateway_cache_handle
                .lookup_gateways(GatewayType::MixnetExit)
                .await
                .map_err(GatewayDirectoryError::LookupGateways)?;
            // Setup the gateway that we will use as the entry point
            let entry_gateways = gateway_cache_handle
                .lookup_gateways(GatewayType::MixnetEntry)
                .await
                .map_err(GatewayDirectoryError::LookupGateways)?;
            (entry_gateways, exit_gateways)
        }
    };

    tracing::info!("Found {} entry gateways", entry_gateways.len());
    tracing::info!("Found {} exit gateways", exit_gateways.len());

    let exit_gateway = exit_point
        .lookup_gateway(&exit_gateways, Some(ScoreValue::High), tunnel_settings.residential_exit)
        .or_else(|err| {
            // When no gateways could be found, lower performance tier and try again
            if err.is_unmatched_non_specific_gateway() {
                tracing::debug!("Could not locate high quality exit gateway. Lowering performance filter to medium and trying again");

                exit_point.lookup_gateway(
                    &exit_gateways,
                    Some(ScoreValue::Medium),
                    tunnel_settings.residential_exit
                )
            } else {
                Err(err)
            }
        })
        .or_else(|err| {
            // When still no gateways found, try low performance as last resort
            if err.is_unmatched_non_specific_gateway() {
                tracing::debug!("Could not locate medium quality exit gateway. Lowering performance filter to low and trying again");

                exit_point.lookup_gateway(
                    &exit_gateways,
                    Some(ScoreValue::Low),
                    tunnel_settings.residential_exit
                )
            } else {
                Err(err)
            }
        })
        .map_err(GatewayDirectoryError::PerformantExitGatewayUnavailable)?;

    // Exclude the exit gateway from the list of entry gateways for privacy reasons
    entry_gateways.remove_gateway(&exit_gateway);

    let entry_gateway = entry_point
        .lookup_gateway(&entry_gateways, Some(ScoreValue::High))
        .or_else(|err| {
            // When no gateways could be found, lower performance tier and try again
            if err.is_unmatched_non_specific_gateway() {
                tracing::debug!("Could not locate high quality entry gateway. Lowering performance filter to medium and trying again");

                entry_point.lookup_gateway(
                    &entry_gateways,
                    Some(ScoreValue::Medium)
                )
            } else {
                Err(err)
            }
        })
        .or_else(|err| {
            // When still no gateways found, try low performance as last resort
            if err.is_unmatched_non_specific_gateway() {
                tracing::debug!("Could not locate medium quality entry gateway. Lowering performance filter to low and trying again");

                entry_point.lookup_gateway(
                    &entry_gateways,
                    Some(ScoreValue::Low)
                )
            } else {
                Err(err)
            }
        })
        .map_err(GatewayDirectoryError::PerformantEntryGatewayUnavailable)?;

    let entry_keys = wg_keys_db
        .load_or_create_keys(&entry_gateway.identity().to_string())
        .await
        .map_err(|source| GatewayDirectoryError::LoadKeypair {
            identity: entry_gateway.identity().to_string(),
            source,
        })?
        .entry_keypair()
        .clone();
    let exit_keys = wg_keys_db
        .load_or_create_keys(&exit_gateway.identity().to_string())
        .await
        .map_err(|source| GatewayDirectoryError::LoadKeypair {
            identity: exit_gateway.identity().to_string(),
            source,
        })?
        .exit_keypair()
        .clone();

    tracing::info!(
        "Using entry gateway: {}, location: {}, performance: {}",
        entry_gateway.identity(),
        entry_gateway
            .two_letter_iso_country_code()
            .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
        entry_gateway
            .mixnet_performance
            .map_or_else(|| "unknown".to_string(), |perf| perf.to_string()),
    );
    tracing::info!(
        "Using exit gateway: {}, location: {}, performance: {}",
        exit_gateway.identity(),
        exit_gateway
            .two_letter_iso_country_code()
            .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
        exit_gateway
            .mixnet_performance
            .map_or_else(|| "unknown".to_string(), |perf| perf.to_string()),
    );
    tracing::info!(
        "Using exit router address {}",
        exit_gateway
            .ipr_address
            .map_or_else(|| "none".to_string(), |ipr| ipr.to_string())
    );

    Ok(SelectedGateways {
        entry: Box::new(GatewayWithKeys {
            gateway: entry_gateway,
            keys: entry_keys,
        }),
        exit: Box::new(GatewayWithKeys {
            gateway: exit_gateway,
            keys: exit_keys,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nym_gateway_directory::{Gateway, Performance, ScoreValue};
    use nym_sdk::mixnet::NodeIdentity;

    /// Helper to create a test gateway with a given score
    fn create_test_gateway(
        identity: &str,
        country: &str,
        score: ScoreValue,
    ) -> Gateway {
        let identity_key = NodeIdentity::from_base58_string(identity)
            .expect("Valid base58 identity");
        
        Gateway::builder()
            .identity(identity_key)
            .location(nym_gateway_directory::Location {
                two_letter_iso_country_code: country.to_string(),
                ..Default::default()
            })
            .performance(Performance {
                last_updated_utc: "2025-10-22T00:00:00Z".to_string(),
                score,
                mixnet_score: ScoreValue::High,
                load: ScoreValue::Low,
                uptime_percentage_last_24_hours: 0.99,
            })
            .build()
    }

    #[test]
    fn test_score_value_ordering() {
        // Verify ScoreValue priority ordering
        assert!(ScoreValue::High > ScoreValue::Medium);
        assert!(ScoreValue::Medium > ScoreValue::Low);
        assert!(ScoreValue::Low > ScoreValue::Offline);
    }

    #[test]
    fn test_gateway_list_filtering_by_score() {
        let gateways = vec![
            create_test_gateway("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42", "US", ScoreValue::High),
            create_test_gateway("BayDqFCz1h7aShk54ewbKg4w8cT9ddqW9PDTMrvcRGgK", "DE", ScoreValue::Medium),
            create_test_gateway("ByxGq9hpDQu6Wc8augEh22w7CRWJHPNfDshB1b8nfWkh", "FR", ScoreValue::Low),
            create_test_gateway("Cz4oKe8rrr7tSGdXfGLfug2WNMfD5XZtYXBtvnjmHaaJ", "HK", ScoreValue::Offline),
        ];

        let gateway_list = GatewayList::new(Some(GatewayType::Wg), gateways);

        // Test filtering by different score thresholds
        let high_only = gateway_list.filter(&[
            nym_gateway_directory::GatewayFilter::MinScore(ScoreValue::High)
        ]);
        assert_eq!(high_only.len(), 1);
        assert_eq!(high_only[0].two_letter_iso_country_code(), Some("US"));

        let medium_and_above = gateway_list.filter(&[
            nym_gateway_directory::GatewayFilter::MinScore(ScoreValue::Medium)
        ]);
        assert_eq!(medium_and_above.len(), 2); // US (High) + DE (Medium)

        let low_and_above = gateway_list.filter(&[
            nym_gateway_directory::GatewayFilter::MinScore(ScoreValue::Low)
        ]);
        assert_eq!(low_and_above.len(), 3); // US + DE + FR
    }

    #[test]
    fn test_fallback_chain_high_available() {
        // When High performance gateways exist, select High
        let entry_point = EntryPoint::Country {
            two_letter_iso_country_code: "US".to_string(),
        };
        
        let gateways = GatewayList::new(
            Some(GatewayType::Wg),
            vec![
                create_test_gateway("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42", "US", ScoreValue::High),
                create_test_gateway("BayDqFCz1h7aShk54ewbKg4w8cT9ddqW9PDTMrvcRGgK", "US", ScoreValue::Medium),
            ],
        );

        // Should select High gateway
        let result = entry_point.lookup_gateway(&gateways, Some(ScoreValue::High));
        assert!(result.is_ok());
        let gateway = result.unwrap();
        assert_eq!(
            gateway.performance.as_ref().unwrap().score,
            ScoreValue::High
        );
    }

    #[test]
    fn test_fallback_chain_medium_when_no_high() {
        // When only Medium and Low exist, should accept Medium
        let entry_point = EntryPoint::Country {
            two_letter_iso_country_code: "DE".to_string(),
        };
        
        let gateways = GatewayList::new(
            Some(GatewayType::Wg),
            vec![
                create_test_gateway("BayDqFCz1h7aShk54ewbKg4w8cT9ddqW9PDTMrvcRGgK", "DE", ScoreValue::Medium),
                create_test_gateway("ByxGq9hpDQu6Wc8augEh22w7CRWJHPNfDshB1b8nfWkh", "DE", ScoreValue::Low),
            ],
        );

        // Try High first (should fail)
        let high_result = entry_point.lookup_gateway(&gateways, Some(ScoreValue::High));
        assert!(high_result.is_err());

        // Fallback to Medium (should succeed)
        let medium_result = entry_point.lookup_gateway(&gateways, Some(ScoreValue::Medium));
        assert!(medium_result.is_ok());
        let gateway = medium_result.unwrap();
        assert_eq!(
            gateway.performance.as_ref().unwrap().score,
            ScoreValue::Medium
        );
    }

    #[test]
    fn test_fallback_chain_low_when_no_high_or_medium() {
        // When only Low exists, should accept Low
        // Before, it tried High->Medium and failed
        // Now it tries High->Medium->Low and succeeds
        let entry_point = EntryPoint::Country {
            two_letter_iso_country_code: "VN".to_string(),
        };
        
        let gateways = GatewayList::new(
            Some(GatewayType::Wg),
            vec![
                create_test_gateway("ByxGq9hpDQu6Wc8augEh22w7CRWJHPNfDshB1b8nfWkh", "VN", ScoreValue::Low),
            ],
        );

        // Try High first (should fail)
        let high_result = entry_point.lookup_gateway(&gateways, Some(ScoreValue::High));
        assert!(high_result.is_err());

        // Try Medium (should fail)
        let medium_result = entry_point.lookup_gateway(&gateways, Some(ScoreValue::Medium));
        assert!(medium_result.is_err());

        // Fallback to Low (should succeed)
        let low_result = entry_point.lookup_gateway(&gateways, Some(ScoreValue::Low));
        assert!(low_result.is_ok());
        let gateway = low_result.unwrap();
        assert_eq!(
            gateway.performance.as_ref().unwrap().score,
            ScoreValue::Low
        );
    }

    #[test]
    fn test_fallback_chain_fails_when_only_offline() {
        // When only Offline gateways exist, all attempts should fail
        let entry_point = EntryPoint::Country {
            two_letter_iso_country_code: "XX".to_string(),
        };
        
        let gateways = GatewayList::new(
            Some(GatewayType::Wg),
            vec![
                create_test_gateway("Cz4oKe8rrr7tSGdXfGLfug2WNMfD5XZtYXBtvnjmHaaJ", "XX", ScoreValue::Offline),
            ],
        );

        // All score levels should fail for Offline gateways
        assert!(entry_point.lookup_gateway(&gateways, Some(ScoreValue::High)).is_err());
        assert!(entry_point.lookup_gateway(&gateways, Some(ScoreValue::Medium)).is_err());
        assert!(entry_point.lookup_gateway(&gateways, Some(ScoreValue::Low)).is_err());
    }

    #[test]
    fn test_specific_gateway_identity_ignores_score_filter() {
        // When selecting by specific gateway identity, score filter should not matter
        // Individually selected nodes work regardless of score
        let gw_identity = "ByxGq9hpDQu6Wc8augEh22w7CRWJHPNfDshB1b8nfWkh";
        let entry_point = EntryPoint::from_base58_string(gw_identity).unwrap();
        
        let gateways = GatewayList::new(
            Some(GatewayType::Wg),
            vec![
                create_test_gateway("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42", "US", ScoreValue::High),
                create_test_gateway(gw_identity, "FR", ScoreValue::Low),
            ],
        );

        let result = entry_point.lookup_gateway(&gateways, Some(ScoreValue::High));
        assert!(result.is_ok());
        let gateway = result.unwrap();
        assert_eq!(gateway.identity().to_base58_string(), gw_identity);
        assert_eq!(
            gateway.performance.as_ref().unwrap().score,
            ScoreValue::Low
        );
    }

    #[test]
    fn test_score_distribution_realistic_scenario() {
        let gateways = vec![
            create_test_gateway("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42", "US", ScoreValue::High),
            create_test_gateway("BayDqFCz1h7aShk54ewbKg4w8cT9ddqW9PDTMrvcRGgK", "DE", ScoreValue::Medium),
            create_test_gateway("ByxGq9hpDQu6Wc8augEh22w7CRWJHPNfDshB1b8nfWkh", "FR", ScoreValue::Medium),
            create_test_gateway("Cz4oKe8rrr7tSGdXfGLfug2WNMfD5XZtYXBtvnjmHaaJ", "GB", ScoreValue::Medium),
            create_test_gateway("DoezvC92kAVDhFpBbsRj52rErhikj2vtPi1Lup2EhbZ4", "NL", ScoreValue::Low),
            create_test_gateway("iifoNSQCXcbXptua6kteMX1X8EttPdw2BHbxjiRUCn4", "VN", ScoreValue::Low),
        ];

        let gateway_list = GatewayList::new(Some(GatewayType::Wg), gateways);

        // Verify distribution
        let high_count = gateway_list.filter(&[
            nym_gateway_directory::GatewayFilter::MinScore(ScoreValue::High)
        ]).len();
        assert_eq!(high_count, 1); // 1 High

        let medium_and_above = gateway_list.filter(&[
            nym_gateway_directory::GatewayFilter::MinScore(ScoreValue::Medium)
        ]);
        assert_eq!(medium_and_above.len(), 4); // 1 High + 3 Medium

        let low_and_above = gateway_list.filter(&[
            nym_gateway_directory::GatewayFilter::MinScore(ScoreValue::Low)
        ]);
        assert_eq!(low_and_above.len(), 6); // 1 High + 3 Medium + 2 Low

        // Without Low fallback node would be unreachable
        let vn_entry = EntryPoint::Country {
            two_letter_iso_country_code: "VN".to_string(),
        };
        
        // High and Medium fail
        assert!(vn_entry.lookup_gateway(&gateway_list, Some(ScoreValue::High)).is_err());
        assert!(vn_entry.lookup_gateway(&gateway_list, Some(ScoreValue::Medium)).is_err());
        
        let vn_gateway = vn_entry.lookup_gateway(&gateway_list, Some(ScoreValue::Low));
        assert!(vn_gateway.is_ok());
        assert_eq!(vn_gateway.unwrap().two_letter_iso_country_code(), Some("VN"));
    }
}
