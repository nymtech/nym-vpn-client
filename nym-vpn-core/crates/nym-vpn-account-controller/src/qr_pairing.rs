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
    error::VpnApiClientError,
    response::{NymVpnGetPairingSecretResponse, NymVpnPairingCheckResponse, NymVpnPairingResponse},
    types::{PairingParams, VpnAccount},
};
use nym_vpn_lib_types::{
    AccountCommandError, PairingResult, QrPairingConfirmation, QrUrl, TunnelEvent,
};
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

enum QrPairingProcessHandle {
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

pub struct QrPairingProcess<C: PairingApiClient> {
    qr_keypair: KeyPair,
    vpn_api_client: C,
    event_sender: broadcast::Sender<TunnelEvent>,
    cancel_token: CancellationToken,
}

impl<C: PairingApiClient + Send + Sync + 'static> QrPairingProcess<C> {
    pub fn new(
        vpn_api_client: C,
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

    fn spawn_pairing_check_task(
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

    fn spawn_get_pairing_secret_task(
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

#[async_trait::async_trait]
pub trait PairingApiClient: Clone {
    async fn pair_account(
        &self,
        account: &VpnAccount,
        qr_pub_key: &str,
    ) -> Result<NymVpnPairingResponse, VpnApiClientError>;
    async fn check_pairing_account(
        &self,
        account: &VpnAccount,
    ) -> Result<NymVpnPairingCheckResponse, VpnApiClientError>;
    async fn try_to_pair_to_account(
        &self,
        remote_qr_pub_key: &str,
        local_qr_pub_key: &str,
        pin_code: &str,
        token: &str,
    ) -> Result<NymVpnPairingResponse, VpnApiClientError>;
    async fn try_to_get_secret_account(
        &self,
        remote_qr_pub_key: &str,
        pairing_params: &PairingParams,
    ) -> Result<NymVpnGetPairingSecretResponse, VpnApiClientError>;
    async fn share_secret_account(
        &self,
        account: &VpnAccount,
        enc_mnemonic: Vec<u8>,
        nonce: Vec<u8>,
    ) -> Result<(), VpnApiClientError>;
}

#[async_trait::async_trait]
impl PairingApiClient for VpnApiClient {
    async fn pair_account(
        &self,
        account: &VpnAccount,
        qr_pub_key: &str,
    ) -> Result<NymVpnPairingResponse, VpnApiClientError> {
        self.pair_account(account, qr_pub_key).await
    }

    async fn check_pairing_account(
        &self,
        account: &VpnAccount,
    ) -> Result<NymVpnPairingCheckResponse, VpnApiClientError> {
        self.check_pairing_account(account).await
    }

    async fn try_to_pair_to_account(
        &self,
        remote_qr_pub_key: &str,
        local_qr_pub_key: &str,
        pin_code: &str,
        token: &str,
    ) -> Result<NymVpnPairingResponse, VpnApiClientError> {
        self.try_to_pair_to_account(remote_qr_pub_key, local_qr_pub_key, pin_code, token)
            .await
    }

    async fn try_to_get_secret_account(
        &self,
        remote_qr_pub_key: &str,
        pairing_params: &PairingParams,
    ) -> Result<NymVpnGetPairingSecretResponse, VpnApiClientError> {
        self.try_to_get_secret_account(remote_qr_pub_key, pairing_params)
            .await
    }

    async fn share_secret_account(
        &self,
        account: &VpnAccount,
        enc_mnemonic: Vec<u8>,
        nonce: Vec<u8>,
    ) -> Result<(), VpnApiClientError> {
        self.share_secret_account(account, enc_mnemonic, nonce)
            .await
    }
}

pub struct QrPairingHandler<C: PairingApiClient> {
    vpn_api_client: C,
    process_handle: QrPairingProcessHandle,
}

impl<C: PairingApiClient + Send + Sync + 'static> QrPairingHandler<C> {
    pub fn new(vpn_api_client: C) -> Self {
        Self {
            vpn_api_client,
            process_handle: QrPairingProcessHandle::None,
        }
    }

    pub(crate) async fn pair_account(
        &mut self,
        account: &Arc<VpnAccount>,
        event_sender: broadcast::Sender<TunnelEvent>,
        cancel_token: CancellationToken,
    ) -> Result<QrUrl, AccountCommandError> {
        let qr_pairing_process =
            QrPairingProcess::new(self.vpn_api_client.clone(), event_sender, cancel_token);

        let response = self
            .vpn_api_client
            .pair_account(account, &qr_pairing_process.qr_public_key())
            .await?;

        let ret = QrUrl {
            url: format!(
                "https://nym.com/qr-pairing?pubkey={}",
                qr_pairing_process.qr_public_key()
            ),
        };

        self.process_handle = qr_pairing_process
            .spawn_pairing_check_task(account.clone(), Duration::from_secs(response.ttl_seconds));

        Ok(ret)
    }

    pub(crate) async fn scanned_key(
        &mut self,
        remote_qr_pub_key_str: &str,
        event_sender: broadcast::Sender<TunnelEvent>,
        cancel_token: CancellationToken,
    ) -> Result<QrPairingConfirmation, AccountCommandError> {
        let qr_pairing_process =
            QrPairingProcess::new(self.vpn_api_client.clone(), event_sender, cancel_token);

        let pin_code = String::new();
        let token = String::new();
        let remote_qr_pub_key = PublicKey::from_str(remote_qr_pub_key_str)
            .map_err(|err| AccountCommandError::InvalidPublicKey(err.to_string()))?;

        let response = self
            .vpn_api_client
            .try_to_pair_to_account(
                remote_qr_pub_key_str,
                &qr_pairing_process.qr_public_key(),
                &pin_code,
                &token,
            )
            .await?;

        self.process_handle = qr_pairing_process.spawn_get_pairing_secret_task(
            remote_qr_pub_key,
            token,
            Duration::from_secs(response.ttl_seconds),
        );

        Ok(QrPairingConfirmation { pin_code })
    }

    pub(crate) async fn verified_pairing_pin(
        &mut self,
        account: &Arc<VpnAccount>,
    ) -> Result<(), AccountCommandError> {
        let QrPairingProcessHandle::PairingCheck { ref shared_key, .. } = self.process_handle
        else {
            tracing::warn!("Pairing needs to be started first");
            return Err(AccountCommandError::PairingNotStarted);
        };
        let (enc_mnemonic, nonce) = {
            let shared_key_lock = shared_key.lock().await;
            let Some(shared_key) = shared_key_lock.as_ref() else {
                tracing::warn!("Cannot authorize secret sharing if there is no DH shared secret");
                return Err(AccountCommandError::NoPairingSharedSecret);
            };

            let nonce = shared_key.random_nonce();
            let enc_mnemonic = shared_key
                .encrypt(account.get_mnemonic().as_bytes(), &nonce)
                .ok_or(AccountCommandError::Internal(String::from(
                    "encrypt mnemonic",
                )))?;
            (enc_mnemonic, nonce)
        };

        self.vpn_api_client
            .share_secret_account(account, enc_mnemonic, nonce.to_vec())
            .await?;

        self.process_handle = QrPairingProcessHandle::None;

        Ok(())
    }

    pub fn cancel_pairing_check(&mut self) {
        self.process_handle.cancel_pairing_check();
    }

    pub fn cancel_get_pairing_secret(&mut self) {
        self.process_handle.cancel_get_pairing_secret();
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use std::collections::HashMap;

    use nym_http_api_client::HttpClientError;
    use nym_vpn_api_client::{
        request::SharePairSecretBody, response::NymVpnPairingFinalisedResponse,
    };
    use tokio::time::Instant;

    use super::*;

    const TTL: Duration = Duration::from_mins(5);

    struct LoggedOutData {
        pairing_response: NymVpnPairingFinalisedResponse,
        token: String,
        started: Instant,
    }

    struct LoggedInData {
        paired_data: Option<LoggedOutData>,
        started: Instant,
    }

    struct SecretData {
        secret_body: SharePairSecretBody,
        shared: Instant,
    }

    #[derive(Default)]
    struct ApiDb {
        account_to_pub_key: HashMap<String, String>,
        pairing_logged_in_data: HashMap<String, LoggedInData>,
        pairing_secret_data: HashMap<String, SecretData>,
    }

    #[derive(Clone)]
    struct MockedApiClient {
        api_db: Arc<Mutex<ApiDb>>,
    }

    impl MockedApiClient {
        pub fn new(api_db: Arc<Mutex<ApiDb>>) -> Self {
            Self { api_db }
        }
    }

    #[async_trait::async_trait]
    impl PairingApiClient for MockedApiClient {
        async fn pair_account(
            &self,
            account: &VpnAccount,
            qr_pub_key: &str,
        ) -> Result<NymVpnPairingResponse, VpnApiClientError> {
            let mut db = self.api_db.lock().await;

            db.account_to_pub_key
                .insert(account.pub_key().to_string(), qr_pub_key.to_string());
            db.pairing_logged_in_data.insert(
                qr_pub_key.to_string(),
                LoggedInData {
                    paired_data: None,
                    started: Instant::now(),
                },
            );

            Ok(NymVpnPairingResponse {
                ttl_seconds: TTL.as_secs(),
            })
        }

        async fn check_pairing_account(
            &self,
            account: &VpnAccount,
        ) -> Result<NymVpnPairingCheckResponse, VpnApiClientError> {
            let db = self.api_db.lock().await;

            let Some(data) = db
                .account_to_pub_key
                .get(account.pub_key())
                .and_then(|k| db.pairing_logged_in_data.get(k))
            else {
                return Err(VpnApiClientError::CheckPairingAccount(Box::new(
                    HttpClientError::GenericRequestFailure("pairing not started".to_string()),
                )));
            };
            if Instant::now() - data.started > TTL {
                return Err(VpnApiClientError::CheckPairingAccount(Box::new(
                    HttpClientError::GenericRequestFailure("pairing timeframe expired".to_string()),
                )));
            }

            Ok(NymVpnPairingCheckResponse {
                inner: data
                    .paired_data
                    .as_ref()
                    .map(|d| d.pairing_response.clone()),
            })
        }

        async fn try_to_pair_to_account(
            &self,
            remote_qr_pub_key: &str,
            local_qr_pub_key: &str,
            pin_code: &str,
            token: &str,
        ) -> Result<NymVpnPairingResponse, VpnApiClientError> {
            let mut db = self.api_db.lock().await;

            let Some(data) = db.pairing_logged_in_data.get_mut(remote_qr_pub_key) else {
                return Err(VpnApiClientError::TryToPairToAccount(Box::new(
                    HttpClientError::GenericRequestFailure(
                        "no pairing process for the provided key".to_string(),
                    ),
                )));
            };

            if Instant::now() - data.started > TTL {
                return Err(VpnApiClientError::TryToPairToAccount(Box::new(
                    HttpClientError::GenericRequestFailure(
                        "pairing process TTL expired".to_string(),
                    ),
                )));
            }

            if data.paired_data.is_some() {
                return Err(VpnApiClientError::TryToPairToAccount(Box::new(
                    HttpClientError::GenericRequestFailure(
                        "pairing completes with first come first serverd party; you might be under a MitM attack".to_string(),
                    ),
                )));
            }

            data.paired_data = Some(LoggedOutData {
                pairing_response: NymVpnPairingFinalisedResponse {
                    pin_code: pin_code.to_string(),
                    remote_pub_key: local_qr_pub_key.to_string(),
                },
                token: token.to_string(),
                started: Instant::now(),
            });

            Ok(NymVpnPairingResponse {
                ttl_seconds: TTL.as_secs(),
            })
        }

        async fn try_to_get_secret_account(
            &self,
            remote_qr_pub_key: &str,
            pairing_params: &PairingParams,
        ) -> Result<NymVpnGetPairingSecretResponse, VpnApiClientError> {
            let db = self.api_db.lock().await;

            let Some(data) = db
                .pairing_logged_in_data
                .get(remote_qr_pub_key)
                .and_then(|data| data.paired_data.as_ref())
            else {
                return Err(VpnApiClientError::GetPairingSecretAccount(Box::new(
                    HttpClientError::GenericRequestFailure(
                        "pairing process not at the correct stage".to_string(),
                    ),
                )));
            };

            if pairing_params.token != data.token {
                return Err(VpnApiClientError::GetPairingSecretAccount(Box::new(
                    HttpClientError::GenericRequestFailure(
                        "token is not the same as the one from the previous pairing step"
                            .to_string(),
                    ),
                )));
            }

            let inner = db
                .pairing_secret_data
                .get(remote_qr_pub_key)
                .map(|d| d.secret_body.clone());

            Ok(NymVpnGetPairingSecretResponse { inner })
        }

        async fn share_secret_account(
            &self,
            account: &VpnAccount,
            enc_mnemonic: Vec<u8>,
            nonce: Vec<u8>,
        ) -> Result<(), VpnApiClientError> {
            let mut db = self.api_db.lock().await;

            let Some(qr_pub_key) = db.account_to_pub_key.get(account.pub_key()).cloned() else {
                return Err(VpnApiClientError::SharePairingSecretAccount(Box::new(
                    HttpClientError::GenericRequestFailure("pairing not started".to_string()),
                )));
            };

            db.pairing_secret_data.insert(
                qr_pub_key,
                SecretData {
                    secret_body: SharePairSecretBody {
                        enc_mnemonic,
                        nonce,
                    },
                    shared: Instant::now(),
                },
            );

            Ok(())
        }
    }

    #[tokio::test]
    async fn happy_flow() {
        let account = Arc::new(VpnAccount::generate_new().unwrap().0);
        let cancel_token = CancellationToken::new();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));

        let logged_in_client = MockedApiClient::new(api_db.clone());
        let mut logged_in_handler = QrPairingHandler::new(logged_in_client);
        let (logged_in_event_tx, mut logged_in_event_rx) = broadcast::channel(10);

        let logged_out_client = MockedApiClient::new(api_db);
        let mut logged_out_handler = QrPairingHandler::new(logged_out_client);
        let (logged_out_event_tx, mut logged_out_event_rx) = broadcast::channel(10);

        // user triggers the pairing process by opening the pairing page
        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, cancel_token.child_token())
            .await
            .unwrap()
            .url;

        // the URL gets encoded into a QR code that gets scanned by the logged out device
        // obtaining the public key needed for the pairing process
        let qr_pub_key = url.split("pubkey=").last().unwrap();

        // the logged out device shows a PIN code that needs to be confirmed manually by the user
        // on the logged in device
        let generated_pin_code = logged_out_handler
            .scanned_key(qr_pub_key, logged_out_event_tx, cancel_token.child_token())
            .await
            .unwrap()
            .pin_code;

        // the logged in device also receives the same PIN code that needs to be shown to the user
        // and be confirmed for match
        let TunnelEvent::Pairing(PairingResult::PinCode(received_pin_code)) =
            logged_in_event_rx.recv().await.unwrap()
        else {
            panic!("no pin code event")
        };
        assert_eq!(generated_pin_code, received_pin_code);

        // after PINs were cross checked, the logged in device initiates sharing of the secret
        logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap();

        // the logged out device receives the mnemonic in the form of an event
        let TunnelEvent::Pairing(PairingResult::Mnemonic(mnemonic)) =
            logged_out_event_rx.recv().await.unwrap()
        else {
            panic!("no mnemonic received")
        };

        // check that the correct mnemonic was shared between devices
        assert_eq!(*account.get_mnemonic(), mnemonic);
    }
}
