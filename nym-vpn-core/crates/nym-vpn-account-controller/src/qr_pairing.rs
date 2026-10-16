// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

use nym_crypto::{
    Aes256GcmSiv, blake3,
    generic_array::typenum::Unsigned,
    hkdf,
    symmetric::aead::{self, AeadKey, KeySizeUser, Nonce, nonce_size, random_nonce},
};
use nym_sdk::mixnet::x25519::{KeyPair, PublicKey};
use nym_vpn_api_client::{
    VpnApiClient,
    error::VpnApiClientError,
    response::{
        NymVpnGetPairingSecretResponse, NymVpnPairingCheckResponse, NymVpnPairingResponse, StatusOk,
    },
    types::{PairingParams, VpnAccount},
};
use nym_vpn_lib_types::{
    AccountCommandError, PairingEvent, PairingResult, QrPairingConfirmation, QrUrl, TunnelEvent,
};
use rand08::{RngCore, rngs::OsRng};
use tokio::sync::{Mutex, broadcast};
use tokio_util::sync::{CancellationToken, DropGuard};

const RETRY_INTERVAL: Duration = Duration::from_secs(5);

type SharedKeySize = <Aes256GcmSiv as KeySizeUser>::KeySize;

pub struct SharedSymmetricKey(AeadKey<Aes256GcmSiv>);

impl SharedSymmetricKey {
    pub fn random_nonce(&self) -> Nonce<Aes256GcmSiv> {
        random_nonce::<Aes256GcmSiv, _>(&mut OsRng)
    }

    pub fn validate_aead_nonce(raw: &[u8]) -> Option<Nonce<Aes256GcmSiv>> {
        if raw.len() != nonce_size::<Aes256GcmSiv>() {
            None
        } else {
            Some(Nonce::<Aes256GcmSiv>::clone_from_slice(raw))
        }
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
        _drop_guard: DropGuard,
    },
    GetPairingSecret {
        _drop_guard: DropGuard,
    },
}

impl QrPairingProcessHandle {
    pub fn cancel_pairing_check(&mut self) {
        if let QrPairingProcessHandle::PairingCheck { .. } = self {
            *self = QrPairingProcessHandle::None;
        }
    }

    pub fn cancel_get_pairing_secret(&mut self) {
        if let QrPairingProcessHandle::GetPairingSecret { .. } = self {
            *self = QrPairingProcessHandle::None;
        }
    }
}

pub struct QrPairingProcess<C: PairingApiClient> {
    qr_keypair: KeyPair,
    vpn_api_client: C,
    event_sender: broadcast::Sender<TunnelEvent>,
    cancel_token: CancellationToken,
    session_id: u32,
}

impl<C: PairingApiClient + Send + Sync + 'static> QrPairingProcess<C> {
    pub fn new(
        vpn_api_client: C,
        event_sender: broadcast::Sender<TunnelEvent>,
        cancel_token: CancellationToken,
        session_id: u32,
    ) -> Self {
        let qr_keypair = KeyPair::new(&mut OsRng);

        Self {
            qr_keypair,
            vpn_api_client,
            event_sender,
            cancel_token,
            session_id,
        }
    }

    fn send_event(&self, result: PairingResult) {
        let event = PairingEvent {
            session_id: self.session_id,
            result,
        };
        self.event_sender.send(TunnelEvent::Pairing(event)).ok();
    }

    /// Runs the request unless the process is cancelled first, so a slow request
    /// can't report anything for a cancelled session
    async fn unless_cancelled<F: Future>(&self, request: F) -> Option<F::Output> {
        tokio::select! {
            biased;
            _ = self.cancel_token.cancelled() => None,
            output = request => Some(output),
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
                    biased;
                    _ = self.cancel_token.cancelled() => {
                        tracing::debug!("QR pairing loop cancelled");
                        self.send_event(PairingResult::Cancelled);
                        break;
                    }
                    _ = interval.tick() => {
                        if initial_instant.elapsed() > ttl {
                            tracing::warn!("QR pairing loop timeout reached");
                            self.send_event(PairingResult::TimedOut);
                            break;
                        }
                        let Some(response) = self
                            .unless_cancelled(self.vpn_api_client.check_pairing_account(&account))
                            .await
                        else {
                            tracing::debug!("QR pairing loop cancelled");
                            self.send_event(PairingResult::Cancelled);
                            break;
                        };
                        match response {
                            Ok(response) => {
                                if let Some(response) = response.inner {
                                    let Ok(remote_pub_key) = PublicKey::from_str(&response.remote_pub_key) else {
                                        tracing::error!("Invalid public key received in QR pairing response");
                                        self.send_event(PairingResult::Error("invalid public key received from API".to_string()));
                                        break;
                                    };
                                    *shared_key_setter.lock().await = Some(self.derive_shared_key(&remote_pub_key));
                                    tracing::debug!("QR code was scanned, checking PIN code");
                                    self.send_event(PairingResult::PinCode(response.pin_code));
                                    break;
                                } else {
                                    tracing::debug!("QR pairing not yet completed, retrying in {RETRY_INTERVAL:?}");
                                }
                            }
                            Err(e) => {
                                tracing::error!("Error checking QR pairing, stopping the checks: {e:?}");
                                self.send_event(PairingResult::Error(e.to_string()));
                                break;
                            }
                        }
                    }
                }
            }
        });

        QrPairingProcessHandle::PairingCheck {
            _drop_guard: cancel_token.drop_guard(),
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
                    biased;
                    _ = self.cancel_token.cancelled() => {
                        tracing::debug!("QR pairing loop cancelled");
                        self.send_event(PairingResult::Cancelled);
                        break;
                    }
                    _ = interval.tick() => {
                        if initial_instant.elapsed() > ttl {
                            tracing::warn!("QR pairing loop timeout reached");
                            self.send_event(PairingResult::TimedOut);
                            break;
                        }
                        let Some(response) = self
                            .unless_cancelled(
                                self.vpn_api_client
                                    .try_to_get_secret_account(&remote_qr_pub_key.to_string(), &pairing_params),
                            )
                            .await
                        else {
                            tracing::debug!("QR pairing loop cancelled");
                            self.send_event(PairingResult::Cancelled);
                            break;
                        };
                        match response {
                            Ok(response) => {
                                if let Some(response) = response.inner {
                                    let Some(mnemonic_bytes) = SharedSymmetricKey::validate_aead_nonce(&response.nonce)
                                        .and_then(|nonce| shared_key.decrypt(&response.enc_mnemonic, &nonce)
                                    ) else {
                                        tracing::error!("decrypt mnemonic");
                                        self.send_event(PairingResult::Error("could not decrypt mnemonic".to_string()));
                                        break;
                                    };
                                    let Ok(mnemonic) = String::from_utf8(mnemonic_bytes) else {
                                        tracing::error!("decode mnemonic bytes");
                                        self.send_event(PairingResult::Error("could not decode mnemonic".to_string()));
                                        break;
                                    };
                                    tracing::debug!("Received account secret mnemonic, pairing completed");
                                    self.send_event(PairingResult::Mnemonic(mnemonic));
                                    break;
                                } else {
                                    tracing::debug!("QR pairing not yet completed, retrying in {RETRY_INTERVAL:?}");
                                }
                            }
                            Err(e) => {
                                tracing::error!("Error getting the account secret, stopping the queries {e:?}");
                                self.send_event(PairingResult::Error(e.to_string()));
                                break;
                            }
                        }
                    }
                }
            }
        });

        QrPairingProcessHandle::GetPairingSecret {
            _drop_guard: cancel_token.drop_guard(),
        }
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
    ) -> Result<StatusOk, VpnApiClientError>;
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
    ) -> Result<StatusOk, VpnApiClientError> {
        self.share_secret_account(account, enc_mnemonic, nonce)
            .await
    }
}

fn random_pin() -> String {
    let mut random = vec![0u8; 6];
    OsRng.fill_bytes(&mut random);
    random.iter().map(|n| (n % 10).to_string()).collect()
}

fn random_token() -> String {
    let mut random = vec![0u8; 16];
    OsRng.fill_bytes(&mut random);
    bs58::encode(random).into_string()
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

    fn new_pairing_process(
        &self,
        event_sender: broadcast::Sender<TunnelEvent>,
        cancel_token: CancellationToken,
    ) -> QrPairingProcess<C> {
        // Unique across handlers, as the account controllers of an app can share the same
        // event listener
        static LAST_SESSION_ID: AtomicU32 = AtomicU32::new(0);
        let session_id = LAST_SESSION_ID
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1);

        QrPairingProcess::new(
            self.vpn_api_client.clone(),
            event_sender,
            cancel_token,
            session_id,
        )
    }

    pub(crate) async fn pair_account(
        &mut self,
        account: &Arc<VpnAccount>,
        event_sender: broadcast::Sender<TunnelEvent>,
        cancel_token: CancellationToken,
    ) -> Result<QrUrl, AccountCommandError> {
        let qr_pairing_process = self.new_pairing_process(event_sender, cancel_token);

        let response = self
            .vpn_api_client
            .pair_account(account, &qr_pairing_process.qr_public_key())
            .await?;

        let ret = QrUrl {
            url: format!(
                "https://nym.com/qr-pairing?pubkey={}",
                qr_pairing_process.qr_public_key()
            ),
            session_id: qr_pairing_process.session_id,
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
        let qr_pairing_process = self.new_pairing_process(event_sender, cancel_token);
        let session_id = qr_pairing_process.session_id;

        let pin_code = random_pin();
        let token = random_token();
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

        Ok(QrPairingConfirmation {
            pin_code,
            session_id,
        })
    }

    async fn verified_pairing_pin_can_fail(
        &self,
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
                tracing::error!(
                    "Function was called without actually awaiting for the PIN, aborting pairing"
                );
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

        Ok(())
    }

    pub(crate) async fn verified_pairing_pin(
        &mut self,
        account: &Arc<VpnAccount>,
    ) -> Result<(), AccountCommandError> {
        let ret = self.verified_pairing_pin_can_fail(account).await;
        self.process_handle = QrPairingProcessHandle::None;
        ret
    }

    pub fn cancel_pairing_check(&mut self) {
        self.process_handle.cancel_pairing_check();
    }

    pub fn cancel_get_pairing_secret(&mut self) {
        self.process_handle.cancel_get_pairing_secret();
    }

    /// Cancels whichever pairing process is running, if any
    pub fn cancel_pairing_process(&mut self) {
        self.process_handle = QrPairingProcessHandle::None;
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
        /// Makes the polling requests hang, like a stalled network
        hang_requests: bool,
        hanging_requests: usize,
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
            let mut db = self.api_db.lock().await;
            if db.hang_requests {
                db.hanging_requests += 1;
                drop(db);
                return std::future::pending().await;
            }

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
            let mut db = self.api_db.lock().await;
            if db.hang_requests {
                db.hanging_requests += 1;
                drop(db);
                return std::future::pending().await;
            }

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
        ) -> Result<StatusOk, VpnApiClientError> {
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

            Ok(StatusOk {
                status: "Ok".to_string(),
            })
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
        let TunnelEvent::Pairing(PairingEvent {
            result: PairingResult::PinCode(received_pin_code),
            ..
        }) = logged_in_event_rx.recv().await.unwrap()
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
        let TunnelEvent::Pairing(PairingEvent {
            result: PairingResult::Mnemonic(mnemonic),
            ..
        }) = logged_out_event_rx.recv().await.unwrap()
        else {
            panic!("no mnemonic received")
        };

        // check that the correct mnemonic was shared between devices
        assert_eq!(*account.get_mnemonic(), mnemonic);
    }

    type TestHandler = QrPairingHandler<MockedApiClient>;

    fn new_handler(
        api_db: &Arc<Mutex<ApiDb>>,
    ) -> (
        TestHandler,
        broadcast::Sender<TunnelEvent>,
        broadcast::Receiver<TunnelEvent>,
    ) {
        let (event_tx, event_rx) = broadcast::channel(10);
        (
            QrPairingHandler::new(MockedApiClient::new(api_db.clone())),
            event_tx,
            event_rx,
        )
    }

    fn new_account() -> Arc<VpnAccount> {
        Arc::new(VpnAccount::generate_new().unwrap().0)
    }

    fn pub_key_from_url(url: &QrUrl) -> &str {
        url.url.split("pubkey=").last().unwrap()
    }

    /// Waits for the next pairing event, checking that it comes from the expected session
    async fn next_pairing_event(
        event_rx: &mut broadcast::Receiver<TunnelEvent>,
        session_id: u32,
    ) -> PairingResult {
        match event_rx.recv().await.unwrap() {
            TunnelEvent::Pairing(event) => {
                assert_eq!(event.session_id, session_id, "{event:?}");
                event.result
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    async fn assert_no_event_for(event_rx: &mut broadcast::Receiver<TunnelEvent>, wait: Duration) {
        if let Ok(event) = tokio::time::timeout(wait, event_rx.recv()).await {
            panic!("unexpected event: {event:?}");
        }
    }

    /// Waits until a polling request hangs in the API
    async fn wait_for_hanging_request(api_db: &Arc<Mutex<ApiDb>>) {
        let wait = async {
            while api_db.lock().await.hanging_requests == 0 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        };
        tokio::time::timeout(RETRY_INTERVAL * 2, wait)
            .await
            .expect("no polling request hung");
    }

    /// Checks that every process that could send on this channel has finished
    async fn assert_channel_closed(event_rx: &mut broadcast::Receiver<TunnelEvent>) {
        let event = tokio::time::timeout(RETRY_INTERVAL * 2, event_rx.recv())
            .await
            .expect("channel still open");
        assert!(
            matches!(event, Err(broadcast::error::RecvError::Closed)),
            "{event:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn pairing_check_times_out_when_qr_is_never_scanned() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);

        let started = Instant::now();
        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();

        // nothing should happen for the whole TTL
        assert_no_event_for(&mut logged_in_event_rx, TTL).await;

        // the timeout is detected on the first retry tick after the TTL
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::TimedOut), "{event:?}");
        assert!(started.elapsed() > TTL);
        assert!(started.elapsed() <= TTL + RETRY_INTERVAL);

        // the shared key was never derived, so the PIN can't be verified
        let err = logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::NoPairingSharedSecret);

        // the failed verification also discards the timed out process
        let err = logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::PairingNotStarted);
    }

    #[tokio::test(start_paused = true)]
    async fn get_pairing_secret_times_out_when_pin_is_never_verified() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();

        let started = Instant::now();
        let confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();

        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");

        // the logged in device never confirms the PIN
        assert_no_event_for(&mut logged_out_event_rx, TTL - started.elapsed()).await;

        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        assert!(matches!(event, PairingResult::TimedOut), "{event:?}");
        assert!(started.elapsed() > TTL);
        assert!(started.elapsed() <= TTL + RETRY_INTERVAL);
    }

    #[tokio::test(start_paused = true)]
    async fn pairing_check_is_cancelled_by_parent_token() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let cancel_token = CancellationToken::new();

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, cancel_token.child_token())
            .await
            .unwrap();

        cancel_token.cancel();

        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::Cancelled), "{event:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn cancelled_pairing_check_cannot_be_verified() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();

        logged_in_handler.cancel_pairing_check();

        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::Cancelled), "{event:?}");

        let err = logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::PairingNotStarted);
    }

    #[tokio::test(start_paused = true)]
    async fn cancel_only_affects_matching_process_kind() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();

        // the logged in device runs a pairing check, not a secret query
        logged_in_handler.cancel_get_pairing_secret();
        assert_no_event_for(&mut logged_in_event_rx, RETRY_INTERVAL * 2).await;

        let confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");

        // the logged out device runs a secret query, not a pairing check
        logged_out_handler.cancel_pairing_check();
        assert_no_event_for(&mut logged_out_event_rx, RETRY_INTERVAL * 2).await;

        logged_out_handler.cancel_get_pairing_secret();
        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        assert!(matches!(event, PairingResult::Cancelled), "{event:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn cancel_pairing_process_stops_pairing_check() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();

        logged_in_handler.cancel_pairing_process();

        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::Cancelled), "{event:?}");
        assert_channel_closed(&mut logged_in_event_rx).await;

        let err = logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::PairingNotStarted);
    }

    #[tokio::test(start_paused = true)]
    async fn cancel_pairing_process_stops_secret_query() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        let confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");

        // the user closes the pairing window on the logged out device
        logged_out_handler.cancel_pairing_process();
        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        assert!(matches!(event, PairingResult::Cancelled), "{event:?}");

        // the secret shared afterwards is never picked up
        logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap();
        assert_channel_closed(&mut logged_out_event_rx).await;
    }

    #[tokio::test(start_paused = true)]
    async fn cancel_pairing_process_without_running_process() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        // nothing to cancel yet
        logged_in_handler.cancel_pairing_process();

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        let confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");
        logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap();
        let PairingResult::Mnemonic(mnemonic) =
            next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await
        else {
            panic!("no mnemonic received")
        };
        assert_eq!(*account.get_mnemonic(), mnemonic);

        // both processes are done, so closing the pairing windows reports nothing
        logged_in_handler.cancel_pairing_process();
        logged_out_handler.cancel_pairing_process();
        assert_channel_closed(&mut logged_in_event_rx).await;
        assert_channel_closed(&mut logged_out_event_rx).await;
    }

    #[tokio::test(start_paused = true)]
    async fn cancel_pairing_process_interrupts_hanging_pairing_check() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();

        // the API stops answering while a check is in flight
        api_db.lock().await.hang_requests = true;
        wait_for_hanging_request(&api_db).await;

        logged_in_handler.cancel_pairing_process();
        let event = tokio::time::timeout(
            RETRY_INTERVAL,
            next_pairing_event(&mut logged_in_event_rx, url.session_id),
        )
        .await
        .expect("cancelling did not interrupt the hanging request");
        assert!(matches!(event, PairingResult::Cancelled), "{event:?}");
        assert_channel_closed(&mut logged_in_event_rx).await;
    }

    #[tokio::test(start_paused = true)]
    async fn cancel_pairing_process_interrupts_hanging_secret_query() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        let confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");

        // the API stops answering while a secret query is in flight
        api_db.lock().await.hang_requests = true;
        wait_for_hanging_request(&api_db).await;

        logged_out_handler.cancel_pairing_process();
        let event = tokio::time::timeout(
            RETRY_INTERVAL,
            next_pairing_event(&mut logged_out_event_rx, confirmation.session_id),
        )
        .await
        .expect("cancelling did not interrupt the hanging request");
        assert!(matches!(event, PairingResult::Cancelled), "{event:?}");
        assert_channel_closed(&mut logged_out_event_rx).await;
    }

    #[tokio::test(start_paused = true)]
    async fn cancel_pairing_process_after_pin_is_shown() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, _logged_out_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        logged_out_handler
            .scanned_key(
                pub_key_from_url(&url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");

        // the PINs don't match, so the user closes the pairing window
        logged_in_handler.cancel_pairing_process();

        // the check had already finished, so there is nothing left to report
        assert_channel_closed(&mut logged_in_event_rx).await;

        // and the mnemonic can't be shared anymore
        let err = logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::PairingNotStarted);
        assert!(api_db.lock().await.pairing_secret_data.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn cancel_pairing_process_wins_over_ready_secret() {
        // without preferring the cancellation, the mnemonic would get through about half the time
        for _ in 0..16 {
            let account = new_account();
            let api_db = Arc::new(Mutex::new(ApiDb::default()));
            let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
                new_handler(&api_db);
            let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
                new_handler(&api_db);

            let url = logged_in_handler
                .pair_account(&account, logged_in_event_tx, CancellationToken::new())
                .await
                .unwrap();
            let confirmation = logged_out_handler
                .scanned_key(
                    pub_key_from_url(&url),
                    logged_out_event_tx,
                    CancellationToken::new(),
                )
                .await
                .unwrap();
            let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
            assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");
            logged_in_handler
                .verified_pairing_pin(&account)
                .await
                .unwrap();

            // the next secret query waits on the API, which already has the mnemonic
            let db = api_db.lock().await;
            tokio::time::sleep(RETRY_INTERVAL + Duration::from_millis(1)).await;

            // the user closes the window just as the API answers
            logged_out_handler.cancel_pairing_process();
            drop(db);

            let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
            assert!(matches!(event, PairingResult::Cancelled), "{event:?}");
            assert_channel_closed(&mut logged_out_event_rx).await;
        }
    }

    #[tokio::test]
    async fn verify_pin_before_pairing_started() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, _, _) = new_handler(&api_db);

        let err = logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::PairingNotStarted);
        assert!(api_db.lock().await.pairing_secret_data.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn verify_pin_before_qr_is_scanned() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();

        let err = logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::NoPairingSharedSecret);
        assert!(api_db.lock().await.pairing_secret_data.is_empty());

        // verifying too early aborts the pairing
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::Cancelled), "{event:?}");
        let err = logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::PairingNotStarted);
    }

    #[tokio::test]
    async fn verify_pin_twice() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, _logged_out_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        logged_out_handler
            .scanned_key(
                pub_key_from_url(&url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");

        logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap();
        let err = logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::PairingNotStarted);
    }

    #[tokio::test(start_paused = true)]
    async fn verify_pin_on_logged_out_device() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, _logged_in_event_rx) = new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        let confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();

        // the logged out device is waiting for the secret, it has no PIN to verify
        let err = logged_out_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::PairingNotStarted);

        // and the secret query is aborted
        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        assert!(matches!(event, PairingResult::Cancelled), "{event:?}");
    }

    #[tokio::test]
    async fn scanned_key_is_not_a_valid_public_key() {
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_out_handler, logged_out_event_tx, _logged_out_event_rx) =
            new_handler(&api_db);

        let err = logged_out_handler
            .scanned_key(
                "not-a-public-key",
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap_err();
        assert!(
            matches!(err, AccountCommandError::InvalidPublicKey(_)),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn scanned_key_without_pairing_started() {
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_out_handler, logged_out_event_tx, _logged_out_event_rx) =
            new_handler(&api_db);
        let unknown_key = KeyPair::new(&mut OsRng).public_key().to_string();

        let err = logged_out_handler
            .scanned_key(&unknown_key, logged_out_event_tx, CancellationToken::new())
            .await
            .unwrap_err();
        assert!(matches!(err, AccountCommandError::Internal(_)), "{err:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn second_device_scanning_same_qr_is_rejected() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);
        let (mut intruder_handler, intruder_event_tx, _intruder_event_rx) = new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        let confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();

        let err = intruder_handler
            .scanned_key(
                pub_key_from_url(&url),
                intruder_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AccountCommandError::Internal(_)), "{err:?}");

        // the first device to scan still completes the pairing
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");
        logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap();
        let PairingResult::Mnemonic(mnemonic) =
            next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await
        else {
            panic!("no mnemonic received")
        };
        assert_eq!(*account.get_mnemonic(), mnemonic);
    }

    #[tokio::test(start_paused = true)]
    async fn pairing_check_reports_api_error() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();

        // the API forgets about the pairing process
        *api_db.lock().await = ApiDb::default();

        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        let PairingResult::Error(err) = event else {
            panic!("unexpected event: {event:?}")
        };
        assert_eq!(err, "failed to check account pairing");
    }

    #[tokio::test(start_paused = true)]
    async fn get_pairing_secret_reports_api_error() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, _logged_in_event_rx) = new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        let confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();

        // the API forgets about the pairing process
        *api_db.lock().await = ApiDb::default();

        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        let PairingResult::Error(err) = event else {
            panic!("unexpected event: {event:?}")
        };
        assert_eq!(err, "failed to get pairing secret account");
    }

    #[tokio::test(start_paused = true)]
    async fn tampered_secret_is_reported() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        let confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");

        // a secret not encrypted with the DH shared key ends up on the API
        let nonce = random_nonce::<Aes256GcmSiv, _>(&mut OsRng);
        MockedApiClient::new(api_db.clone())
            .share_secret_account(&account, vec![0u8; 64], nonce.to_vec())
            .await
            .unwrap();

        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        assert!(matches!(event, PairingResult::Error(_)), "{event:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn secret_with_wrong_nonce_length_is_reported() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        let confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");

        MockedApiClient::new(api_db.clone())
            .share_secret_account(&account, vec![0u8; 64], vec![0u8; 3])
            .await
            .unwrap();

        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        assert!(matches!(event, PairingResult::Error(_)), "{event:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn pairing_check_reports_invalid_remote_key() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();

        // the API hands back a malformed key for the scanning device
        MockedApiClient::new(api_db.clone())
            .try_to_pair_to_account(pub_key_from_url(&url), "not-a-public-key", "", "")
            .await
            .unwrap();

        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::Error(_)), "{event:?}");

        let err = logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::NoPairingSharedSecret);
    }

    #[tokio::test(start_paused = true)]
    async fn restarting_pairing_cancels_previous_check() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        // both processes report on the same channel, like they do in the app
        let first_url = logged_in_handler
            .pair_account(
                &account,
                logged_in_event_tx.clone(),
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let second_url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        assert_ne!(first_url.url, second_url.url);
        assert_ne!(first_url.session_id, second_url.session_id);

        // the stale process reports its cancellation under its own session
        let event = next_pairing_event(&mut logged_in_event_rx, first_url.session_id).await;
        assert!(matches!(event, PairingResult::Cancelled), "{event:?}");

        let confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&second_url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let event = next_pairing_event(&mut logged_in_event_rx, second_url.session_id).await;
        assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");

        logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap();
        let PairingResult::Mnemonic(mnemonic) =
            next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await
        else {
            panic!("no mnemonic received")
        };
        assert_eq!(*account.get_mnemonic(), mnemonic);

        // the first process is gone for good
        assert_channel_closed(&mut logged_in_event_rx).await;
    }

    #[tokio::test(start_paused = true)]
    async fn rescanning_cancels_previous_secret_query() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        let first_url = logged_in_handler
            .pair_account(
                &account,
                logged_in_event_tx.clone(),
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let first_confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&first_url),
                logged_out_event_tx.clone(),
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let event = next_pairing_event(&mut logged_in_event_rx, first_url.session_id).await;
        assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");

        // the user restarts the whole flow and scans a new QR code
        let second_url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        let second_confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&second_url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_ne!(
            first_confirmation.session_id,
            second_confirmation.session_id
        );

        // the stale query reports its cancellation under its own session
        let event =
            next_pairing_event(&mut logged_out_event_rx, first_confirmation.session_id).await;
        assert!(matches!(event, PairingResult::Cancelled), "{event:?}");

        let event = next_pairing_event(&mut logged_in_event_rx, second_url.session_id).await;
        assert!(matches!(event, PairingResult::PinCode(_)), "{event:?}");
        logged_in_handler
            .verified_pairing_pin(&account)
            .await
            .unwrap();
        let PairingResult::Mnemonic(mnemonic) =
            next_pairing_event(&mut logged_out_event_rx, second_confirmation.session_id).await
        else {
            panic!("no mnemonic received")
        };
        assert_eq!(*account.get_mnemonic(), mnemonic);
    }

    #[tokio::test]
    async fn session_ids_are_unique_across_handlers() {
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut first_handler, first_event_tx, _first_event_rx) = new_handler(&api_db);
        let (mut second_handler, second_event_tx, _second_event_rx) = new_handler(&api_db);

        // two account controllers sharing one event listener must never reuse a session ID
        let first_url = first_handler
            .pair_account(&new_account(), first_event_tx, CancellationToken::new())
            .await
            .unwrap();
        let second_url = second_handler
            .pair_account(&new_account(), second_event_tx, CancellationToken::new())
            .await
            .unwrap();
        assert_ne!(first_url.session_id, second_url.session_id);
    }

    #[tokio::test(start_paused = true)]
    async fn dropping_handler_cancels_process() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);

        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        drop(logged_in_handler);

        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(matches!(event, PairingResult::Cancelled), "{event:?}");
    }

    #[test]
    fn pin_generation() {
        let pin = random_pin();
        assert_eq!(pin.len(), 6);
        for c in pin.chars() {
            assert!(c >= '0' && c <= '9');
        }
    }
}
