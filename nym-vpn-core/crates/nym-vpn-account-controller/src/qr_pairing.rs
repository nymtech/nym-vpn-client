// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use nym_sdk::mixnet::x25519::KeyPair;
use nym_vpn_api_client::{
    VpnApiClient,
    types::{PairingParams, VpnAccount},
};
use nym_vpn_lib_types::{PairingResult, TunnelEvent};
use rand08::rngs::OsRng;
use tokio::sync::broadcast;
use tokio_util::sync::{CancellationToken, DropGuard};

const RETRY_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Default)]
#[allow(unused)]
pub enum QrPairingProcessDropGuard {
    #[default]
    None,
    PairingCheckGuard(DropGuard),
    GetPairingSecretGuard(DropGuard),
}

impl QrPairingProcessDropGuard {
    pub fn drop_pairing_check(&mut self) {
        if matches!(self, QrPairingProcessDropGuard::PairingCheckGuard(_)) {
            *self = QrPairingProcessDropGuard::None;
        }
    }

    pub fn drop_get_pairing_secret(&mut self) {
        if matches!(self, QrPairingProcessDropGuard::GetPairingSecretGuard(_)) {
            *self = QrPairingProcessDropGuard::None;
        }
    }
}

pub struct QrPairingProcess {
    qr_keypair: KeyPair,
    vpn_api_client: VpnApiClient,
    event_sender: broadcast::Sender<TunnelEvent>,
    cancel_token: CancellationToken,
}

impl QrPairingProcess {
    pub fn new(
        vpn_api_client: VpnApiClient,
        event_sender: broadcast::Sender<TunnelEvent>,
        cancel_token: CancellationToken,
    ) -> Self {
        let qr_keypair = KeyPair::new(&mut OsRng);

        Self {
            qr_keypair,
            vpn_api_client,
            event_sender,
            cancel_token,
        }
    }

    pub fn qr_public_key(&self) -> String {
        self.qr_keypair.public_key().to_string()
    }

    pub fn spawn_pairing_check_task(
        self,
        account: Arc<VpnAccount>,
        ttl: Duration,
    ) -> QrPairingProcessDropGuard {
        let initial_instant = tokio::time::Instant::now();
        let mut interval = tokio::time::interval(RETRY_INTERVAL);
        let drop_guard = self.cancel_token.clone().drop_guard();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = self.cancel_token.cancelled() => {
                        tracing::debug!("QR pairing loop cancelled");
                        self.event_sender.send(TunnelEvent::Pairing(PairingResult::Cancelled)).ok();
                        break;
                    }
                    _ = interval.tick() => {
                        if initial_instant.elapsed() > ttl {
                            tracing::warn!("QR pairing loop timeout reached");
                            self.event_sender.send(TunnelEvent::Pairing(PairingResult::TimedOut)).ok();
                            break;
                        }
                        match self
                            .vpn_api_client
                            .check_pairing_account(&account)
                            .await
                        {
                            Ok(response) => {
                                if let Some(pin_code) = response.pin_code {
                                    tracing::debug!("QR code was scanned, checking PIN code");
                                    self.event_sender.send(TunnelEvent::Pairing(PairingResult::PinCode(pin_code))).ok();
                                    break;
                                } else {
                                    tracing::debug!("QR pairing not yet completed, retrying in {RETRY_INTERVAL:?}");
                                }
                            }
                            Err(e) => {
                                tracing::error!("Error checking QR pairing, stopping the checks: {e:?}");
                                self.event_sender.send(TunnelEvent::Pairing(PairingResult::Error(e.to_string()))).ok();
                                break;
                            }
                        }
                    }
                }
            }
        });

        QrPairingProcessDropGuard::PairingCheckGuard(drop_guard)
    }

    pub fn spawn_get_pairing_secret_task(
        self,
        remote_qr_pub_key: String,
        token: String,
        ttl: Duration,
    ) -> QrPairingProcessDropGuard {
        let initial_instant = tokio::time::Instant::now();
        let mut interval = tokio::time::interval(RETRY_INTERVAL);
        let pairing_params = PairingParams { token };
        let drop_guard = self.cancel_token.clone().drop_guard();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = self.cancel_token.cancelled() => {
                        tracing::debug!("QR pairing loop cancelled");
                        self.event_sender.send(TunnelEvent::Pairing(PairingResult::Cancelled)).ok();
                        break;
                    }
                    _ = interval.tick() => {
                        if initial_instant.elapsed() > ttl {
                            tracing::warn!("QR pairing loop timeout reached");
                            self.event_sender.send(TunnelEvent::Pairing(PairingResult::TimedOut)).ok();
                            break;
                        }
                        match self
                            .vpn_api_client
                            .try_to_get_secret_account(&remote_qr_pub_key, &pairing_params)
                            .await
                        {
                            Ok(response) => {
                                if let Some(mnemonic) = response.mnemonic {
                                    tracing::debug!("Received account secret mnemonic, pairing completed");
                                    self.event_sender.send(TunnelEvent::Pairing(PairingResult::Mnemonic(mnemonic))).ok();
                                    break;
                                } else {
                                    tracing::debug!("QR pairing not yet completed, retrying in {RETRY_INTERVAL:?}");
                                }
                            }
                            Err(e) => {
                                tracing::error!("Error getting the account secret, stopping the queries {e:?}");
                                self.event_sender.send(TunnelEvent::Pairing(PairingResult::Error(e.to_string()))).ok();
                                break;
                            }
                        }
                    }
                }
            }
        });

        QrPairingProcessDropGuard::GetPairingSecretGuard(drop_guard)
    }
}
