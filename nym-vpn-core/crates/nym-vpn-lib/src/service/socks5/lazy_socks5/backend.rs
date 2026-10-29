// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Lifecycle of the Nym mixnet client behind the lazy SOCKS5 wrapper.

use super::{LazySocks5, LazySocks5Error, SharedState};
use nym_gateway_directory::ScoreValue;
use nym_sdk::mixnet::{MixnetClientBuilder, Socks5, StoragePaths};
use nym_vpn_lib_types::{TunnelState, TunnelType};
use rand::seq::IndexedRandom;
use std::{net::SocketAddr, sync::atomic::Ordering, time::Duration};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

impl LazySocks5 {
    /// Ensure backend is started with retry logic
    /// Retries backend initialization in case of transient failures (e.g., DNS timeouts)
    pub async fn ensure_backend_started_with_retry(
        shared: &SharedState,
        client_addr: SocketAddr,
    ) -> Result<(), LazySocks5Error> {
        const MAX_RETRIES: u32 = 3;
        const INITIAL_RETRY_DELAY: Duration = Duration::from_millis(500);

        let mut last_error = None;
        for attempt in 1..=MAX_RETRIES {
            match Box::pin(Self::ensure_backend_started(shared)).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    error!(
                        "Failed to start backend for {} (attempt {}/{}): {}",
                        client_addr, attempt, MAX_RETRIES, e
                    );
                    last_error = Some(e);

                    if attempt < MAX_RETRIES {
                        let delay = INITIAL_RETRY_DELAY * 2_u32.pow(attempt - 1);
                        info!("Retrying backend start in {:?}...", delay);
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.expect("last_error should always be set after MAX_RETRIES attempts"))
    }

    /// Shut down the backend
    /// Note: Callers should check for active connections before calling this
    pub async fn shutdown_backend(shared: &SharedState) {
        let mut client_guard = shared.mixnet_client.lock().await;
        if let Some(mixnet_client) = client_guard.take() {
            info!("Shutting down Nym mixnet client");
            shared.is_mixnet_running.store(false, Ordering::Relaxed);
            mixnet_client.disconnect().await;
        }
    }

    /// Rotate to a new Network Requester by shutting down current mixnet client
    /// Only rotates if there are no active connections to avoid disrupting ongoing transactions
    pub async fn rotate_network_requester(shared: &SharedState) {
        // Check if there are active connections
        let active_count = shared.active_connections();

        if active_count > 0 {
            info!(
                "Skipping Network Requester rotation: {} active connection(s) in progress",
                active_count
            );
            info!("Rotation will be attempted at next interval when connections are idle");
            return;
        }

        info!("Shutting down current mixnet client to rotate Network Requester");
        Self::shutdown_backend(shared).await;
        info!("Next SOCKS5 connection will use a new random Network Requester");
    }

    /// Build a mixnet client with the given gateway (or None for random selection)
    async fn build_mixnet_client(
        shared: &SharedState,
        socks5_storage_paths: &StoragePaths,
        socks5_config: &Socks5,
        gateway_id: Option<&String>,
    ) -> Result<
        nym_sdk::mixnet::DisconnectedMixnetClient<nym_sdk::mixnet::OnDiskPersistent>,
        LazySocks5Error,
    > {
        let mut builder =
            MixnetClientBuilder::new_with_default_storage(socks5_storage_paths.clone())
                .await
                .map_err(|e| {
                    error!("Failed to create mixnet client builder: {}", e);
                    LazySocks5Error::Internal(e.to_string())
                })?;

        // Configure network environment if provided
        if let Some(ref network_details) = shared.config.network_details {
            builder = builder.network_details(network_details.clone());
            debug!(
                "Using network environment: {}",
                network_details.network_name
            );
        }

        // Configure gateway if specified
        if let Some(gateway_id) = gateway_id {
            builder = builder.request_gateway(gateway_id.clone());
        }

        let mixnet_client = builder
            .socks5_config(socks5_config.clone())
            .with_custom_bandwidth_provider(Box::new(shared.config.bandwidth_command_tx.clone()))
            .build()
            .map_err(|e| {
                error!("Failed to build mixnet client: {}", e);
                LazySocks5Error::Internal(e.to_string())
            })?;

        Ok(mixnet_client)
    }

    /// Ensure the backend is started (lazy initialization)
    async fn ensure_backend_started(shared: &SharedState) -> Result<(), LazySocks5Error> {
        // Hold the lock until the client is stored to prevent concurrent initialization
        let mut mixnet_client_slot = shared.mixnet_client.lock().await;
        if mixnet_client_slot.is_some() {
            return Ok(());
        }

        if shared.cancel_token.is_cancelled() {
            return Err(LazySocks5Error::Internal(
                "SOCKS5 service is shutting down".to_owned(),
            ));
        }

        debug!("First connection detected, initializing Nym mixnet backend...");

        // Determine Network Requester address (fixed or random)
        let network_requester_address = match &shared.config.network_requester_address {
            Some(fixed_address) => {
                debug!("Using fixed Network Requester: {}", fixed_address);
                fixed_address.clone()
            }
            None => {
                info!("Selecting random Network Requester from gateway directory...");
                // If random selection fails, retry with exponential backoff
                let mut last_error = None;
                let mut selected_nr = None;
                for attempt in 1..=3 {
                    match Self::select_random_network_requester(shared).await {
                        Ok(random_nr) => {
                            if attempt > 1 {
                                info!(
                                    "Successfully selected random Network Requester after {} attempt(s)",
                                    attempt
                                );
                            } else {
                                // Log the selected Network Requester on first successful attempt
                                info!("Selected random Network Requester: {}", random_nr);
                            }
                            selected_nr = Some(random_nr);
                            break;
                        }
                        Err(e) => {
                            warn!(
                                "Random Network Requester selection failed (attempt {}/3): {}",
                                attempt, e
                            );
                            last_error = Some(e);
                            if attempt < 3 {
                                let delay = Duration::from_millis(500 * 2_u64.pow(attempt - 1));
                                debug!("Retrying random selection in {:?}...", delay);
                                sleep(delay).await;
                            }
                        }
                    }
                }
                // If all retries failed, return the last error
                selected_nr
                    .ok_or_else(|| last_error.unwrap_or(LazySocks5Error::NoNetworkRequesters))?
            }
        };

        info!("Using Network Requester: {}", network_requester_address);

        let mut socks5_config = Socks5::new(network_requester_address);
        socks5_config.send_anonymously = true;
        socks5_config.bind_address = shared.config.internal_listen_address;

        // Create a custom StoragePaths that shares the credential database with the main VPN
        // but uses a separate identity by storing keys in a sibling "_socks5" directory
        let mixnet_folder_name = shared
            .config
            .mixnet_data_path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("mixnet_data_path must have a valid file name");
        let socks5_data_path = shared
            .config
            .mixnet_data_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(format!("{}_socks5", mixnet_folder_name));

        // Ensure parent directory exists (permissions will be checked when we try to write)
        if let Some(parent) = socks5_data_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                error!(
                    "Failed to create parent directory {}: {e}",
                    parent.display()
                );
                LazySocks5Error::Internal(format!(
                    "Failed to create parent directory {}: {e}. Check directory permissions.",
                    parent.display()
                ))
            })?;
        }

        // Remove old socks5 directory if it exists
        // - to get fresh identity each time
        // - to not worry about version migrations
        let create_dir = if socks5_data_path.exists() {
            match tokio::fs::remove_dir_all(&socks5_data_path).await {
                Ok(_) => {
                    info!(
                        "Removed old socks5 directory {}",
                        socks5_data_path.display()
                    );
                    true
                }
                Err(e) => {
                    warn!(
                        "Failed to remove old socks5 directory {}: {e}.  We will use the old directory.",
                        socks5_data_path.display()
                    );
                    false
                }
            }
        } else {
            true
        };

        if create_dir {
            tokio::fs::create_dir_all(&socks5_data_path)
                .await
                .map_err(|e| {
                    error!(
                        "Failed to create socks5 data directory {}: {e}",
                        socks5_data_path.display()
                    );
                    LazySocks5Error::Internal(format!(
                        "Failed to create socks5 data directory {}: {e}",
                        socks5_data_path.display()
                    ))
                })?;

            debug!("Created fresh socks5 directory for new identity");
        }

        // Create storage paths for SOCKS5 identity
        let socks5_storage_paths = StoragePaths::new_from_dir(&socks5_data_path).map_err(|e| {
            error!("Failed to create socks5 storage paths: {}", e);
            LazySocks5Error::Internal(format!("Failed to create socks5 storage paths: {}", e))
        })?;

        // BC note : We are NOT overriding the credential store, because it's not supposed to be shared anymore
        // Credential requests go through the channel for it now
        debug!(
            "Using separate identity keys in: {}",
            socks5_data_path.display()
        );

        // Build the mixnet client with shared credentials but different identity
        // When dVPN is connected (WireGuard mode), use VPN's entry gateway for firewall compatibility.
        // The entry gateway is fixed, but we can route to any Network Requester (exit) for privacy.
        let tunnel_state = shared.tunnel_state.borrow().clone();

        // Always use VPN's entry gateway if VPN is connected (for firewall compatibility)
        // The Network Requester (exit) is independent and can be any available Network Requester
        let requested_gateway_id = if let TunnelState::Connected { connection_data } = &tunnel_state
        {
            Some(connection_data.entry_gateway.id.clone())
        } else {
            None
        };

        // Build and connect with VPN's entry gateway (if VPN is connected)
        // The Network Requester address in socks5_config determines the exit point
        let mixnet_client = match Self::build_mixnet_client(
            shared,
            &socks5_storage_paths,
            &socks5_config,
            requested_gateway_id.as_ref(),
        )
        .await
        {
            Ok(client) => {
                match Box::pin(client.connect_to_mixnet_via_socks5()).await {
                    Ok(connected_client) => connected_client,
                    Err(e) => {
                        let error_msg = e.to_string();
                        // Check if error is about gateway not found and we have a requested gateway
                        if let Some(gateway_id) = requested_gateway_id.as_ref()
                            && error_msg.contains("no gateway with id")
                        {
                            let is_wireguard_mode = matches!(
                                tunnel_state,
                                TunnelState::Connected { ref connection_data }
                                if connection_data.tunnel.tunnel_type() == TunnelType::Wireguard
                            );

                            if is_wireguard_mode {
                                // WireGuard mode: cannot change entry gateway (firewall rules)
                                error!(
                                    "VPN's entry gateway {} unavailable. Cannot use SOCKS5 in WireGuard mode: firewall rules require VPN's entry gateway.",
                                    gateway_id
                                );
                                return Err(LazySocks5Error::Internal(format!(
                                    "Cannot use SOCKS5 in WireGuard mode: VPN's entry gateway {} is not available. \
                                    Firewall rules only allow the VPN's entry gateway.",
                                    gateway_id
                                )));
                            } else {
                                // Not WireGuard: fallback to random gateway
                                warn!(
                                    "Gateway {} unavailable, falling back to random selection",
                                    gateway_id
                                );

                                let fallback_client = Self::build_mixnet_client(
                                    shared,
                                    &socks5_storage_paths,
                                    &socks5_config,
                                    None,
                                )
                                .await
                                .map_err(|e| {
                                    error!("Failed to build fallback client: {}", e);
                                    e
                                })?;

                                Box::pin(fallback_client.connect_to_mixnet_via_socks5())
                                    .await
                                    .map_err(|fallback_error| {
                                        LazySocks5Error::Internal(format!(
                                            "Failed to connect: gateway {} failed ({}), fallback also failed ({})",
                                            gateway_id,
                                            error_msg,
                                            fallback_error
                                        ))
                                    })?
                            }
                        } else {
                            return Err(LazySocks5Error::Internal(error_msg));
                        }
                    }
                }
            }
            Err(e) => return Err(e),
        };

        info!(
            "SOCKS5 mixnet backend connected (address: {})",
            mixnet_client.nym_address()
        );

        *mixnet_client_slot = Some(mixnet_client);
        shared.is_mixnet_running.store(true, Ordering::Relaxed);
        drop(mixnet_client_slot);

        // Give the internal SOCKS5 server a moment to fully bind
        sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Select a random Network Requester from the gateway directory.
    /// # Errors
    /// - `LazySocks5Error::GatewayDirectory` if gateway cache is not configured
    /// - `LazySocks5Error::NoNetworkRequesters` if no nodes have NR addresses
    async fn select_random_network_requester(
        shared: &SharedState,
    ) -> Result<String, LazySocks5Error> {
        let Some(ref gateway_cache_handle) = shared.config.gateway_cache_handle else {
            error!("Gateway cache handle not configured for random NR selection");
            error!("This is a configuration error - rotation requires gateway_cache_handle");
            return Err(LazySocks5Error::GatewayDirectory(
                "Gateway cache handle not available. Required for random Network Requester selection. \
                 Ensure gateway_cache_handle is provided when enabling rotation.".to_string(),
            ));
        };

        // Fetch NymNodes with SOCKS5 probe data from VPN API
        // This uses a separate method to avoid breaking existing code that depends on skimmed nodes
        debug!("Fetching NymNodes with SOCKS5 probe data for Network Requester selection");
        let nymnodes = gateway_cache_handle
            .lookup_nymnodes_for_socks5()
            .await
            .map_err(|e| {
                error!("Failed to fetch NymNodes with SOCKS5 data: {}", e);
                LazySocks5Error::GatewayDirectory(format!(
                    "Failed to lookup NymNodes with SOCKS5 probe data from gateway directory: {}. \
                     Ensure gateway directory is accessible and properly configured.",
                    e
                ))
            })?;

        let total_nodes = nymnodes.len();
        debug!("Fetched {} nodes from directory", total_nodes);

        // Filter nodes that have a network requester address
        // Exclude VPN exit gateway for privacy (avoid correlation between VPN and SOCKS5 traffic)
        // Filter by SOCKS5 score: prefer High, fallback to Medium (exclude Low/Offline)
        let vpn_exit_identity = shared.config.vpn_exit_gateway_identity.as_ref();
        let mut high_score_nodes = Vec::new();
        let mut medium_score_nodes = Vec::new();
        let mut excluded_by_score = 0;

        for (node, nr_address, socks5_score) in nymnodes
            .into_iter()
            .filter_map(|gw| {
                // Check if node has Network Requester address first
                // No NR address, skip
                gw.nr_address.clone().map(|nr_address| (gw, nr_address))
            })
            .filter(|(gw, _)| {
                // only consider gateways that are NOT the VPN exit gateway
                // unwrap_or_default works because if there's no vpn exit
                // identity, it resolves to empty string, which will be NOT EQUAL
                // to any gateway ID, thus preserving those gateways IN the set
                gw.identity().to_string() != vpn_exit_identity.cloned().unwrap_or_default()
            })
            .filter_map(|(gw, nr_address)| {
                // Only consider exit-capable gateways (SOCKS5 Network Requesters must be exit gateways)
                let score = gw
                    .last_probe
                    .as_ref()
                    .and_then(|probe| probe.outcome.socks5.as_ref())
                    .map(|socks5| socks5.score);
                score.map(|socks5_score| (gw, nr_address, socks5_score))
            })
        {
            // Filter by SOCKS5 score: prefer High, fallback to Medium (exclude Low/Offline/None)
            match socks5_score {
                Some(ScoreValue::High) => {
                    // High score - preferred
                    high_score_nodes.push((node.clone(), nr_address.clone()));
                }
                Some(ScoreValue::Medium) => {
                    // Medium score - fallback option
                    medium_score_nodes.push((node.clone(), nr_address.clone()));
                }
                Some(score) => {
                    // Low or Offline - exclude
                    excluded_by_score += 1;
                    debug!(
                        "Excluding node {} with low SOCKS5 score: {:?}",
                        node.identity(),
                        score
                    );
                }
                None => {
                    // No score data - exclude
                    excluded_by_score += 1;
                    debug!(
                        "Excluding node {} with no SOCKS5 score data",
                        node.identity()
                    );
                }
            }
        }

        // Prefer High score nodes, fallback to Medium if no High available
        let nodes_with_nr = if !high_score_nodes.is_empty() {
            info!(
                "Found {} High score Network Requesters (excluding {} low/no-score nodes)",
                high_score_nodes.len(),
                excluded_by_score
            );
            high_score_nodes
        } else if !medium_score_nodes.is_empty() {
            warn!(
                "No High score Network Requesters available, falling back to {} Medium score nodes (excluding {} low/no-score nodes)",
                medium_score_nodes.len(),
                excluded_by_score
            );
            medium_score_nodes
        } else {
            error!("No Network Requesters available with High/Medium SOCKS5 scores");
            error!(
                "Filtered out {} nodes (low score or no score data)",
                excluded_by_score
            );
            error!(
                "This may indicate a network issue, outdated gateway cache, or all available proxies have low scores"
            );
            error!("Will retry selection to find better options");
            return Err(LazySocks5Error::NoNetworkRequesters);
        };

        let nr_count = nodes_with_nr.len();

        // Select a random one
        let mut rng = rand::rng();
        let (selected_node, nr_address) = nodes_with_nr.choose(&mut rng).ok_or_else(|| {
            error!(
                "Random selection failed despite having {} candidates",
                nr_count
            );
            LazySocks5Error::NoNetworkRequesters
        })?;

        info!(
            "Selected random Network Requester: {} (gateway: {}, {} total available)",
            nr_address, selected_node.identity, nr_count
        );

        Ok(nr_address.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tests::create_test_config, *};
    use tokio::sync::watch;
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn test_gateway_cache_handle_not_provided() {
        // Test that random NR selection fails gracefully without gateway cache
        let mut config = create_test_config();
        config.network_requester_address = None; // Request random
        config.gateway_cache_handle = None; // But no cache handle provided

        let (_tunnel_state_tx, tunnel_state) = watch::channel(TunnelState::Disconnected);
        let cancel_token = CancellationToken::new();

        let shared = SharedState::new(config, tunnel_state, cancel_token);

        // Attempt random selection should fail with informative error
        let result = LazySocks5::select_random_network_requester(&shared).await;
        assert!(result.is_err(), "Should fail without gateway cache handle");

        match result {
            Err(LazySocks5Error::GatewayDirectory(msg)) => {
                assert!(
                    msg.contains("Gateway cache handle not available"),
                    "Error should mention missing cache handle"
                );
                assert!(
                    msg.contains("Required for random Network Requester selection"),
                    "Error should explain why it's needed"
                );
            }
            _ => panic!("Expected GatewayDirectory error"),
        }
    }
}
