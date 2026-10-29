// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{BlacklistedGateways, Location};
use nym_vpn_store::keys::wireguard::WireguardKeysDb;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    TunnelSettings,
    tunnel::gateway_provider::{
        GatewayProviderEventSender, SELECTION_BUFFER_SIZE, SelectedGatewaysStream,
        SelectionResultSender, gateway_cache::GatewayCache, geo_ip::GeoIpProvider,
        selector::select_gateways,
    },
};

#[derive(Clone)]
pub struct SelectAndSend {
    pub tunnel_settings: TunnelSettings,
    /// Whether the entry gateway must support bridges; owned by the connecting state.
    pub use_bridges: bool,
    pub selection_tx: SelectionResultSender,
}

async fn continuous_select<C: GatewayCache>(
    select_and_send: SelectAndSend,
    gateway_provider_event_sender: Option<GatewayProviderEventSender>,
    gateway_cache: C,
    blacklisted_gateways: &BlacklistedGateways,
    device_location: Option<Location>,
    wg_keys_db: &WireguardKeysDb,
) {
    loop {
        // make sure the buffer is not full before deciding on other possible selections
        let Ok(selection_tx) = select_and_send.selection_tx.reserve().await else {
            tracing::debug!("GatewayProvider shut down during selection algorithm");
            return;
        };
        let selection = select_gateways(
            gateway_cache.clone(),
            blacklisted_gateways,
            &select_and_send.tunnel_settings,
            select_and_send.use_bridges,
            gateway_provider_event_sender.clone(),
            device_location.clone(),
            wg_keys_db,
        )
        .await;
        selection_tx.send(selection);
    }
}

pub struct SelectionAlgorithm<C: GatewayCache> {
    tunnel_settings_rx: mpsc::Receiver<SelectAndSend>,
    gateway_provider_event_sender_rx: mpsc::Receiver<GatewayProviderEventSender>,
    selected_gateways_stream: SelectedGatewaysStream,
    gateway_cache: C,
    geo_ip_provider: GeoIpProvider,
    blacklisted_gateways: BlacklistedGateways,
    wg_keys_db: WireguardKeysDb,
    shutdown_token: CancellationToken,
}

impl<C: GatewayCache> SelectionAlgorithm<C> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tunnel_settings_rx: mpsc::Receiver<SelectAndSend>,
        gateway_provider_event_sender_rx: mpsc::Receiver<GatewayProviderEventSender>,
        selected_gateways_stream: SelectedGatewaysStream,
        gateway_cache: C,
        geo_ip_provider: GeoIpProvider,
        blacklisted_gateways: BlacklistedGateways,
        wg_keys_db: WireguardKeysDb,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            tunnel_settings_rx,
            gateway_provider_event_sender_rx,
            selected_gateways_stream,
            gateway_cache,
            geo_ip_provider,
            blacklisted_gateways,
            wg_keys_db,
            shutdown_token,
        }
    }

    pub async fn run(
        mut self,
        mut latest_tunnel_settings: SelectAndSend,
        mut latest_location: Option<Location>,
    ) {
        let mut gateway_provider_event_sender = None;
        loop {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    tracing::info!("SelectionAlgorithm shut down");
                    break;
                }
                Some(new_settings) = self.tunnel_settings_rx.recv() => {
                    latest_tunnel_settings = new_settings;
                }
                Some(sender) = self.gateway_provider_event_sender_rx.recv() => {
                    gateway_provider_event_sender = Some(sender);
                }
                new_location = self.geo_ip_provider.new_location() => {
                    let location_changed = new_location.is_some() && new_location != latest_location;
                    latest_location = new_location;
                    if location_changed {
                        self.reset_selection_stream(&mut latest_tunnel_settings).await;
                    }
                }
                _ = continuous_select(
                        latest_tunnel_settings.clone(),
                        gateway_provider_event_sender.clone(),
                        self.gateway_cache.clone(),
                        &self.blacklisted_gateways,
                        latest_location.clone(),
                        &self.wg_keys_db,
                    ) => {},
            }
        }
        self.wg_keys_db.close().await;
    }

    /// Replace the selection stream with a fresh one, so that selections which
    /// were already buffered for the previous location are dropped and the next
    /// connect uses the new location.
    ///
    /// `GatewayProvider::set_tunnel_settings` holds the stream lock while it
    /// sends new settings to us, so keep receiving settings while waiting for
    /// the lock. New settings come with a fresh stream of their own, which
    /// makes the replacement unnecessary.
    async fn reset_selection_stream(&mut self, latest_tunnel_settings: &mut SelectAndSend) {
        tokio::select! {
            mut selected_gateways_stream = self.selected_gateways_stream.lock() => {
                let (selection_tx, selection_rx) = mpsc::channel(SELECTION_BUFFER_SIZE);
                *selected_gateways_stream =
                    tokio_stream::StreamExt::peekable(ReceiverStream::new(selection_rx));
                latest_tunnel_settings.selection_tx = selection_tx;
                tracing::debug!(
                    "Location changed, dropped gateway selections computed for the previous one"
                );
            }
            Some(new_settings) = self.tunnel_settings_rx.recv() => {
                *latest_tunnel_settings = new_settings;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use futures::StreamExt as _;
    use nym_gateway_directory::{Gateway, Performance, ScoreValue};
    use nym_vpn_lib_types::{EntryPoint, ExitPoint};
    use tokio::sync::{Mutex, RwLock};

    use crate::tunnel_state_machine::tunnel::gateway_provider::{
        SelectionResult,
        gateway_cache::tests::MockGatewayCache,
        tests::{default_tunnel_settings, gateway_id_to_gateway},
    };

    use super::*;

    async fn reset_default_settings(
        tunnel_settings_tx: &mpsc::Sender<SelectAndSend>,
    ) -> mpsc::Receiver<SelectionResult> {
        let (selection_tx, selection_rx) = mpsc::channel(1);
        tunnel_settings_tx
            .send(SelectAndSend {
                tunnel_settings: default_tunnel_settings(),
                use_bridges: false,
                selection_tx,
            })
            .await
            .unwrap();
        selection_rx
    }

    #[tokio::test]
    async fn run_algo() {
        let (tunnel_settings_tx, tunnel_settings_rx) = mpsc::channel(1);
        let (_tunnel_monitor_event_sender_tx, tunnel_monitor_event_sender_rx) = mpsc::channel(1);
        let (_update_location_tx, update_location_rx) = mpsc::unbounded_channel();
        let (selection_tx, selection_rx) = mpsc::channel(10);
        let shutdown_token = CancellationToken::new();
        let possible_gateways_ids = [
            "2zHiExNRKiCXVKS35SNKtK4apGfZELMpA1jJ2gVevJoz",
            "38zcSsvjXsAX7C28ko2H3Lt55X4TYxfZYkPADxKXZHUj",
        ];
        let possible_gateways = possible_gateways_ids.map(gateway_id_to_gateway);

        let gateways = Arc::new(RwLock::new(None));
        let algo = SelectionAlgorithm::new(
            tunnel_settings_rx,
            tunnel_monitor_event_sender_rx,
            Arc::new(Mutex::new(tokio_stream::StreamExt::peekable(
                ReceiverStream::new(selection_rx),
            ))),
            MockGatewayCache::new(gateways.clone()),
            GeoIpProvider::new(update_location_rx),
            BlacklistedGateways::new(),
            WireguardKeysDb::Ephemeral(Default::default()),
            shutdown_token.clone(),
        );
        let handle = tokio::spawn(algo.run(
            SelectAndSend {
                tunnel_settings: default_tunnel_settings(),
                use_bridges: false,
                selection_tx,
            },
            None,
        ));

        // set some default settings
        let mut selection_rx = reset_default_settings(&tunnel_settings_tx).await;
        // error when no gateway available
        *gateways.write().await = Some(vec![]);
        assert!(selection_rx.recv().await.unwrap().is_err());

        // re-set some default settings because there are continuous retries with errors in the stream
        let mut selection_rx = reset_default_settings(&tunnel_settings_tx).await;
        // error when not enough gateways
        *gateways.write().await = Some(vec![possible_gateways[0].clone()]);
        assert!(selection_rx.recv().await.unwrap().is_err());

        // re-set some default settings because there are continuous retries with errors in the stream
        let mut selection_rx = reset_default_settings(&tunnel_settings_tx).await;
        // 2 gateways should be minimally functional
        *gateways.write().await = Some(possible_gateways.to_vec());
        // check that the stream is infinite, as long as there are valid available choices
        for _ in 0..100 {
            let selected_gateways = selection_rx.recv().await.unwrap().unwrap();
            assert!(
                possible_gateways_ids.contains(
                    &selected_gateways
                        .entry_gateway()
                        .identity()
                        .to_base58_string()
                        .as_str()
                )
            );
            assert!(
                possible_gateways_ids.contains(
                    &selected_gateways
                        .exit_gateway()
                        .identity()
                        .to_base58_string()
                        .as_str()
                )
            );
        }

        shutdown_token.cancel();
        handle.await.unwrap();
    }

    const CH_GATEWAYS: [&str; 3] = [
        "24h2yanCFU5iy7xNQmW6RowFa6EzmAYQdM1bs8Y1X6iH",
        "26ZmTxTVBKHZg8MTKwypHkXZVJhDC7QHuv3BdsyRyTuk",
        "27GwHdmXLULVieyXmxZ6v9DHzRJtTEjfode1dzbptEAK",
    ];
    const US_GATEWAYS: [&str; 3] = [
        "28tXg9mEW4mifgU1TdetVVAN5PvmhtLpHzFRMfJBT6ND",
        "29U3LythwEaqigL5YajXALw1c7DE7YNcRW7Vn7KcYMQL",
        "2aZj5UjC4N3SMjfJNjFiaHPqg1sgKDBYwaLozxGePQxW",
    ];

    fn zurich() -> Location {
        Location {
            two_letter_iso_country_code: "CH".to_string(),
            latitude: 47.37,
            longitude: 8.54,
            ..Default::default()
        }
    }

    fn new_york() -> Location {
        Location {
            two_letter_iso_country_code: "US".to_string(),
            latitude: 40.71,
            longitude: -74.01,
            ..Default::default()
        }
    }

    fn gateway_at(id: &str, location: Location) -> Gateway {
        Gateway::builder()
            .identity(id.parse().unwrap())
            .performance(Performance {
                last_updated_utc: Default::default(),
                score: ScoreValue::High,
                mixnet_score: ScoreValue::High,
                load: ScoreValue::Low,
                uptime_percentage_last_24_hours: Default::default(),
            })
            .location(location)
            .build()
    }

    /// Regression test for #6655: selections buffered before a location change
    /// must not be handed out afterwards.
    #[tokio::test]
    async fn location_change_drops_buffered_selections() {
        let (_tunnel_settings_tx, tunnel_settings_rx) = mpsc::channel(1);
        let (_tunnel_monitor_event_sender_tx, tunnel_monitor_event_sender_rx) = mpsc::channel(1);
        let (update_location_tx, update_location_rx) = mpsc::unbounded_channel();
        let (selection_tx, selection_rx) = mpsc::channel(SELECTION_BUFFER_SIZE);
        let selected_gateways_stream = Arc::new(Mutex::new(tokio_stream::StreamExt::peekable(
            ReceiverStream::new(selection_rx),
        )));
        let shutdown_token = CancellationToken::new();

        let gateways = CH_GATEWAYS
            .iter()
            .map(|id| gateway_at(id, zurich()))
            .chain(US_GATEWAYS.iter().map(|id| gateway_at(id, new_york())))
            .collect();
        let mut tunnel_settings = default_tunnel_settings();
        tunnel_settings.entry_point = Box::new(EntryPoint::Auto {
            exclude_user_country: false,
        });
        tunnel_settings.exit_point = Box::new(ExitPoint::Auto {
            exclude_entry_point_country: false,
            exclude_user_country: false,
        });

        let algo = SelectionAlgorithm::new(
            tunnel_settings_rx,
            tunnel_monitor_event_sender_rx,
            selected_gateways_stream.clone(),
            MockGatewayCache::new(Arc::new(RwLock::new(Some(gateways)))),
            GeoIpProvider::new(update_location_rx),
            BlacklistedGateways::new(),
            WireguardKeysDb::Ephemeral(Default::default()),
            shutdown_token.clone(),
        );
        let handle = tokio::spawn(algo.run(
            SelectAndSend {
                tunnel_settings,
                use_bridges: false,
                selection_tx,
            },
            None,
        ));

        let next_entry = || async {
            let selection = selected_gateways_stream
                .lock()
                .await
                .next()
                .await
                .unwrap()
                .unwrap();
            selection.entry_gateway().identity().to_base58_string()
        };

        // The device starts out in Switzerland, and the buffer fills up with Swiss picks.
        update_location_tx.send(Some(zurich())).unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        let entry = next_entry().await;
        assert!(CH_GATEWAYS.contains(&entry.as_str()), "got {entry}");
        tokio::time::sleep(Duration::from_millis(200)).await;

        // The device moves to the US. Every selection from now on must be based on that.
        update_location_tx.send(Some(new_york())).unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        for _ in 0..SELECTION_BUFFER_SIZE + 1 {
            let entry = next_entry().await;
            assert!(
                US_GATEWAYS.contains(&entry.as_str()),
                "selection computed for the previous location was handed out: {entry}"
            );
        }

        shutdown_token.cancel();
        handle.await.unwrap();
    }
}
