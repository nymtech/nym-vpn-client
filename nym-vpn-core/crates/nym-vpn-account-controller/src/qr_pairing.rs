// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use nym_sdk::mixnet::x25519::KeyPair;
use nym_vpn_api_client::{VpnApiClient, types::VpnAccount};
use nym_vpn_lib_types::{PairingCheckResult, TunnelEvent};
use rand08::rngs::OsRng;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

const CHECK_PAIRING_RETRY_INTERVAL: Duration = Duration::from_secs(5);

pub struct QrPairingProcess {
    qr_keypair: KeyPair,
    vpn_api_client: VpnApiClient,
    account: Arc<VpnAccount>,
    event_sender: broadcast::Sender<TunnelEvent>,
    cancel_token: CancellationToken,
}

impl QrPairingProcess {
    pub fn new(
        vpn_api_client: VpnApiClient,
        account: Arc<VpnAccount>,
        event_sender: broadcast::Sender<TunnelEvent>,
        cancel_token: CancellationToken,
    ) -> Self {
        let qr_keypair = KeyPair::new(&mut OsRng);

        Self {
            qr_keypair,
            vpn_api_client,
            account,
            event_sender,
            cancel_token,
        }
    }

    pub fn qr_public_key(&self) -> String {
        self.qr_keypair.public_key().to_string()
    }

    pub fn spawn_pairing_check_task(self, ttl: Duration) {
        let initial_instant = tokio::time::Instant::now();
        let mut interval = tokio::time::interval(CHECK_PAIRING_RETRY_INTERVAL);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = self.cancel_token.cancelled() => {
                        tracing::debug!("QR pairing loop cancelled");
                        self.event_sender.send(TunnelEvent::PairingCheck(PairingCheckResult::Cancelled)).ok();
                        break;
                    }
                    _ = interval.tick() => {
                        if initial_instant.elapsed() > ttl {
                            tracing::warn!("QR pairing loop timeout reached");
                            self.event_sender.send(TunnelEvent::PairingCheck(PairingCheckResult::TimedOut)).ok();
                            break;
                        }
                        match self
                            .vpn_api_client
                            .check_pairing_account(&self.account)
                            .await
                        {
                            Ok(response) => {
                                if let Some(pin_code) = response.pin_code {
                                    tracing::debug!("QR code was scanned, checking PIN code");
                                    self.event_sender.send(TunnelEvent::PairingCheck(PairingCheckResult::PinCode(pin_code))).ok();
                                    break;
                                } else {
                                    tracing::debug!("QR pairing not yet completed, retrying in {CHECK_PAIRING_RETRY_INTERVAL:?}");
                                }
                            }
                            Err(e) => {
                                tracing::error!("Error checking QR pairing, stopping the checks: {e:?}");
                                self.event_sender.send(TunnelEvent::PairingCheck(PairingCheckResult::Error(e.to_string()))).ok();
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}
