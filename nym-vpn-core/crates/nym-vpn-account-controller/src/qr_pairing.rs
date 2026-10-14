// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{str::FromStr, sync::Arc, time::Duration};

use nym_crypto::{
    Aes256GcmSiv, blake3,
    generic_array::typenum::Unsigned,
    hkdf,
    symmetric::aead::{self, AeadKey, KeySizeUser, Nonce, random_nonce},
};
use nym_sdk::mixnet::x25519::{KeyPair, PublicKey};
use nym_vpn_api_client::{
    VpnApiClient,
    types::{PairingParams, VpnAccount},
};
use nym_vpn_lib_types::{PairingResult, TunnelEvent};
use rand08::rngs::OsRng;
use tokio::sync::{Mutex, broadcast};
use tokio_util::sync::CancellationToken;

const RETRY_INTERVAL: Duration = Duration::from_secs(5);

type SharedKeySize = <Aes256GcmSiv as KeySizeUser>::KeySize;

pub struct SharedSymmetricKey(AeadKey<Aes256GcmSiv>);

impl SharedSymmetricKey {
    pub fn random_nonce(&self) -> Nonce<Aes256GcmSiv> {
        random_nonce::<Aes256GcmSiv, _>(&mut OsRng)
    }

    pub fn validate_aead_nonce(raw: &[u8]) -> Nonce<Aes256GcmSiv> {
        Nonce::<Aes256GcmSiv>::clone_from_slice(raw)
    }

    pub fn encrypt(&self, plaintext: &[u8], nonce: &Nonce<Aes256GcmSiv>) -> Option<Vec<u8>> {
        aead::encrypt::<Aes256GcmSiv>(&self.0, nonce, plaintext)
            .inspect_err(|err| tracing::warn!("Could not encrypt plaintext: {err:?}"))
            .ok()
    }

    pub fn decrypt(&self, ciphertext: &[u8], nonce: &Nonce<Aes256GcmSiv>) -> Option<Vec<u8>> {
        aead::decrypt::<Aes256GcmSiv>(&self.0, nonce, ciphertext)
            .inspect_err(|err| tracing::warn!("Could not decrypt ciphertext: {err:?}"))
            .ok()
    }
}

pub enum QrPairingProcessHandle {
    None,
    PairingCheck {
        shared_key: Arc<Mutex<Option<SharedSymmetricKey>>>,
        cancel_token: CancellationToken,
    },
    GetPairingSecret {
        cancel_token: CancellationToken,
    },
}

impl QrPairingProcessHandle {
    pub fn cancel_pairing_check(&mut self) {
        if let QrPairingProcessHandle::PairingCheck { cancel_token, .. } = self {
            cancel_token.cancel();
            *self = QrPairingProcessHandle::None;
        }
    }

    pub fn cancel_get_pairing_secret(&mut self) {
        if let QrPairingProcessHandle::GetPairingSecret { cancel_token } = self {
            cancel_token.cancel();
            *self = QrPairingProcessHandle::None;
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

    fn derive_shared_key(&self, remote_public: &PublicKey) -> SharedSymmetricKey {
        let dh_result = self.qr_keypair.private_key().diffie_hellman(remote_public);
        let key_size = SharedKeySize::to_usize();

        let okm = hkdf::extract_then_expand::<blake3::Hasher>(None, &dh_result, None, key_size)
            .expect("somehow too long okm was provided");

        SharedSymmetricKey(AeadKey::<Aes256GcmSiv>::clone_from_slice(&okm))
    }

    pub fn spawn_pairing_check_task(
        self,
        account: Arc<VpnAccount>,
        ttl: Duration,
    ) -> QrPairingProcessHandle {
        let initial_instant = tokio::time::Instant::now();
        let mut interval = tokio::time::interval(RETRY_INTERVAL);
        let cancel_token = self.cancel_token.clone();
        let shared_key = Arc::new(Mutex::new(None));
        let shared_key_setter = shared_key.clone();

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
                                if let Some(response) = response.inner {
                                    let Ok(remote_pub_key) = PublicKey::from_str(&response.remote_pub_key) else {
                                        tracing::error!("Invalid public key received in QR pairing reponse");
                                        break;
                                    };
                                    *shared_key_setter.lock().await = Some(self.derive_shared_key(&remote_pub_key));
                                    tracing::debug!("QR code was scanned, checking PIN code");
                                    self.event_sender.send(TunnelEvent::Pairing(PairingResult::PinCode(response.pin_code))).ok();
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

        QrPairingProcessHandle::PairingCheck {
            cancel_token,
            shared_key,
        }
    }

    pub fn spawn_get_pairing_secret_task(
        self,
        remote_qr_pub_key: PublicKey,
        token: String,
        ttl: Duration,
    ) -> QrPairingProcessHandle {
        let initial_instant = tokio::time::Instant::now();
        let mut interval = tokio::time::interval(RETRY_INTERVAL);
        let pairing_params = PairingParams { token };
        let cancel_token = self.cancel_token.clone();
        let shared_key = self.derive_shared_key(&remote_qr_pub_key);

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
                            .try_to_get_secret_account(&remote_qr_pub_key.to_string(), &pairing_params)
                            .await
                        {
                            Ok(response) => {
                                if let Some(response) = response.inner {
                                    let Some(mnemonic_bytes) = shared_key.decrypt(
                                        &response.enc_mnemonic,
                                        &SharedSymmetricKey::validate_aead_nonce(&response.nonce),
                                    ) else {
                                        tracing::error!("decrypt mnemonic");
                                        break;
                                    };
                                    let Ok(mnemonic) = String::from_utf8(mnemonic_bytes) else {
                                        tracing::error!("decode mnemonic bytes");
                                        break;
                                    };
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

        QrPairingProcessHandle::GetPairingSecret { cancel_token }
    }
}
