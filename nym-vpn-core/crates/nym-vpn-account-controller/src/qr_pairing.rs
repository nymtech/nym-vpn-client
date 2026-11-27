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

use bip39::Mnemonic;
use nym_crypto::{
    Aes256GcmSiv, blake3, hkdf,
    symmetric::aead::{self, AeadKey, KeySizeUser, Nonce, nonce_size, random_nonce},
};
use nym_qr_pairing_requests::{
    NymVpnGetPairingSecretResponse, NymVpnPairingCheckResponse, NymVpnPairingFinalisedResponse,
    NymVpnPairingResponse,
};
use nym_sdk::mixnet::x25519::{KeyPair, PublicKey};
use nym_vpn_api_client::{
    VpnApiClient, error::VpnApiClientError, response::StatusOk, types::VpnAccount,
};
use nym_vpn_lib_types::{AccountCommandError, QrPairingConfirmation, QrUrl};
use rand::Rng;
use tokio::sync::Mutex;
use tokio_util::sync::{CancellationToken, DropGuard};

use crate::{
    AccountControllerEventSender,
    event::{
        AccountControllerEvent, AccountControllerPairingEvent, AccountControllerPairingResult,
    },
};

const RETRY_INTERVAL: Duration = Duration::from_secs(5);
const PIN_ENC_INFO: &str = "pin_enc_info";
const PIN_RE_ENC_INFO: &str = "pin_re_enc_info";
const MNEMONIC_ENC_INFO: &str = "mnemonic_enc_info";

type SharedKeySizeUser = Aes256GcmSiv;

pub struct SharedSymmetricKey(AeadKey<Aes256GcmSiv>);

impl SharedSymmetricKey {
    pub fn random_nonce(&self) -> Nonce<Aes256GcmSiv> {
        let mut rng = rand::rng();
        random_nonce::<Aes256GcmSiv, _>(&mut rng)
    }

    pub fn validate_aead_nonce(raw: &[u8]) -> Option<Nonce<Aes256GcmSiv>> {
        if raw.len() != nonce_size::<Aes256GcmSiv>() {
            None
        } else {
            Nonce::<Aes256GcmSiv>::try_from(raw).ok()
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
        qr_pubkey: String,
        session_id: u32,
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
    event_sender: AccountControllerEventSender,
    cancel_token: CancellationToken,
    session_id: u32,
}

impl<C: PairingApiClient + Send + Sync + 'static> QrPairingProcess<C> {
    pub fn new(
        vpn_api_client: C,
        event_sender: AccountControllerEventSender,
        cancel_token: CancellationToken,
        session_id: u32,
    ) -> Self {
        let mut rng = rand::rng();
        let qr_keypair = KeyPair::new(&mut rng);

        Self {
            qr_keypair,
            vpn_api_client,
            event_sender,
            cancel_token,
            session_id,
        }
    }

    fn send_event(&self, result: AccountControllerPairingResult) {
        let event = AccountControllerPairingEvent {
            session_id: self.session_id,
            result,
        };
        self.event_sender
            .broadcast(AccountControllerEvent::Pairing(event));
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

    fn derive_shared_key(
        &self,
        remote_public: &PublicKey,
        info: &str,
    ) -> Result<SharedSymmetricKey, AccountCommandError> {
        let dh_result = self.qr_keypair.private_key().diffie_hellman(remote_public);
        let key_size = SharedKeySizeUser::key_size();

        let okm = hkdf::extract_then_expand::<blake3::Hasher>(
            None,
            &dh_result,
            Some(info.as_bytes()),
            key_size,
        )
        .expect("somehow too long okm was provided");

        AeadKey::<Aes256GcmSiv>::try_from(&okm)
            .map(SharedSymmetricKey)
            .map_err(|err| AccountCommandError::PairingDerivingSharedSecret(err.to_string()))
    }

    fn treat_err(&self, err_msg: &str) -> String {
        tracing::error!(err_msg);
        self.send_event(AccountControllerPairingResult::Error(err_msg.to_string()));
        err_msg.to_string()
    }

    async fn check_pairing_account(
        &self,
        account: &VpnAccount,
    ) -> Result<Option<NymVpnPairingFinalisedResponse>, String> {
        match self
            .unless_cancelled(
                self.vpn_api_client
                    .check_pairing_account(account, &self.qr_keypair.public_key().to_string()),
            )
            .await
        {
            Some(Ok(response)) => {
                if response.inner.is_none() {
                    tracing::debug!("QR pairing not yet completed, retrying in {RETRY_INTERVAL:?}")
                }
                Ok(response.inner)
            }
            Some(Err(VpnApiClientError::HttpClient(e))) if e.is_timeout() => {
                tracing::debug!("Timeout on checking the pairing");
                Ok(None)
            }
            Some(Err(err)) => Err(self.treat_err(&format!(
                "Error checking QR pairing, stopping the checks: {err:?}"
            ))),
            None => {
                tracing::debug!("QR pairing loop cancelled");
                self.send_event(AccountControllerPairingResult::Cancelled);
                Err("cancelled".to_string())
            }
        }
    }

    async fn confirm_pairing_pin(
        &self,
        account: &VpnAccount,
        response: NymVpnPairingFinalisedResponse,
    ) -> Result<Option<(SharedSymmetricKey, String)>, String> {
        let Ok(remote_pub_key) = PublicKey::from_str(&response.remote_pub_key) else {
            return Err(self.treat_err("Invalid public key received in QR pairing response"));
        };
        let Ok(first_shared_key) = self.derive_shared_key(&remote_pub_key, PIN_ENC_INFO) else {
            return Err(self.treat_err("First key derivation not possible"));
        };
        let Some(received_nonce) = SharedSymmetricKey::validate_aead_nonce(&response.nonce) else {
            return Err(self.treat_err("Nonce validation failed"));
        };
        let Some(pin_code_bytes) =
            first_shared_key.decrypt(&response.encrypted_pin_code, &received_nonce)
        else {
            return Err(self.treat_err("Could not decrypt pin code"));
        };
        if !is_valid_pin(&pin_code_bytes) {
            return Err(self.treat_err("Invalid PIN code"));
        }

        let Ok(second_shared_key) = self.derive_shared_key(&remote_pub_key, PIN_RE_ENC_INFO) else {
            return Err(self.treat_err("Second key derivation not possible"));
        };
        let nonce = second_shared_key.random_nonce();
        let Some(re_encrypted_pin_code) = second_shared_key.encrypt(&pin_code_bytes, &nonce) else {
            return Err(self.treat_err("Could not re-encrypt PIN"));
        };

        let pin_code = String::from_utf8_lossy_owned(pin_code_bytes);
        let Ok(third_shared_key) = self.derive_shared_key(&remote_pub_key, MNEMONIC_ENC_INFO)
        else {
            return Err(self.treat_err("Third key derivation not possible"));
        };

        match self
            .unless_cancelled(self.vpn_api_client.confirm_pin(
                account,
                &self.qr_keypair.public_key().to_string(),
                re_encrypted_pin_code,
                nonce.to_vec(),
            ))
            .await
        {
            Some(Err(VpnApiClientError::HttpClient(e))) if e.is_timeout() => {
                tracing::debug!("Timeout on confirming the PIN");
                Ok(None)
            }
            Some(Err(err)) => Err(self.treat_err(&format!("Confirming PIN: {err:?}"))),
            None => {
                tracing::debug!("QR confirm pairing PIN loop cancelled");
                self.send_event(AccountControllerPairingResult::Cancelled);
                Err("cancelled".to_string())
            }
            _ => Ok(Some((third_shared_key, pin_code))),
        }
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
        let session_id = self.session_id;
        let qr_pubkey = self.qr_keypair.public_key().to_string();
        let mut received_response: Option<NymVpnPairingFinalisedResponse> = None;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = self.cancel_token.cancelled() => {
                        tracing::debug!("QR pairing loop cancelled");
                        self.send_event(AccountControllerPairingResult::Cancelled);
                        break;
                    }
                    _ = interval.tick() => {
                        if initial_instant.elapsed() > ttl {
                            tracing::warn!("QR pairing loop timeout reached");
                            self.send_event(AccountControllerPairingResult::TimedOut);
                            break;
                        }
                        let response = if let Some(r) = &received_response {
                            r.clone()
                        } else {
                            let response = match self.check_pairing_account(&account).await {
                                Ok(Some(response)) => response,
                                Ok(None) => continue,
                                Err(_) => break,
                            };
                            received_response = Some(response.clone());
                            response
                        };

                        let (shared_key, pin_code) = match self.confirm_pairing_pin(&account, response).await {
                            Ok(Some((key, pin_code))) => (key, pin_code),
                            Ok(None) => continue,
                            Err(_) => break,
                        };
                        *shared_key_setter.lock().await = Some(shared_key);
                        tracing::debug!("QR code was scanned, checking PIN code");
                        self.send_event(AccountControllerPairingResult::PinCode(pin_code));
                        break;
                    }
                }
            }
        });

        QrPairingProcessHandle::PairingCheck {
            _drop_guard: cancel_token.drop_guard(),
            session_id,
            shared_key,
            qr_pubkey,
        }
    }

    async fn get_pairing_secret(
        &self,
        remote_qr_pub_key: &PublicKey,
        pin_code: &str,
        pairing_token: &str,
        confirmed: &mut bool,
    ) -> Result<Option<Mnemonic>, String> {
        match self
            .unless_cancelled(
                self.vpn_api_client
                    .try_to_get_secret_account(&remote_qr_pub_key.to_string(), pairing_token),
            )
            .await
        {
            Some(Ok(NymVpnGetPairingSecretResponse::Nothing)) => {
                tracing::debug!("QR pairing not yet completed, retrying in {RETRY_INTERVAL:?}");
                Ok(None)
            }
            Some(Ok(NymVpnGetPairingSecretResponse::PinConfirmation(confirmation))) => {
                if *confirmed {
                    tracing::debug!(
                        "QR pairing PIN already confirmed, waiting for the secret, retrying in {RETRY_INTERVAL:?}"
                    );
                    return Ok(None);
                }
                let Some(received_nonce) =
                    SharedSymmetricKey::validate_aead_nonce(&confirmation.nonce)
                else {
                    return Err(self.treat_err("Nonce validation failed on PIN confirmation"));
                };
                let Ok(shared_key) = self.derive_shared_key(remote_qr_pub_key, PIN_RE_ENC_INFO)
                else {
                    return Err(self.treat_err("Second PIN encryption derived key"));
                };
                let Some(pin_code_bytes) =
                    shared_key.decrypt(&confirmation.enc_pin_code, &received_nonce)
                else {
                    return Err(self.treat_err("Could not decrypt pin code"));
                };
                let confirmed_pin = String::from_utf8_lossy_owned(pin_code_bytes);
                if *pin_code != confirmed_pin {
                    return Err(
                        self.treat_err("PIN not confirmed, the pairing process might be MiMed")
                    );
                }
                *confirmed = true;
                tracing::debug!("QR pairing PIN confirmed, showing PIN to user");
                self.send_event(AccountControllerPairingResult::PinCode(confirmed_pin));
                Ok(None)
            }
            Some(Ok(NymVpnGetPairingSecretResponse::Secret(secret))) => {
                if !*confirmed {
                    // If the user clicks the PIN confirmation button before they actually see the PIN on the other device, we'll get
                    // the secret eariler. That's not a big issue and it's on the user's responsability.
                    // In case there's a MiM attack where the user didn't click the confirmation button, the decryption bellow will fail anyway.
                    // In case there's a MiM attack and the user clicked the confirmation button without actually confirming, than they
                    // leaked their mnemonic X)
                    tracing::warn!(
                        "The user might have clicked the PIN confirmation without actually seeing the PIN match."
                    )
                }
                let Some(received_nonce) = SharedSymmetricKey::validate_aead_nonce(&secret.nonce)
                else {
                    return Err(self.treat_err("Nonce validation failed on secret"));
                };
                let Ok(shared_key) = self.derive_shared_key(remote_qr_pub_key, MNEMONIC_ENC_INFO)
                else {
                    return Err(self.treat_err("Mnemonic encryption derived key"));
                };
                let Some(mnemonic_bytes) =
                    shared_key.decrypt(&secret.enc_mnemonic, &received_nonce)
                else {
                    return Err(self.treat_err("Could not decrypt mnemonic"));
                };
                let Ok(mnemonic) = String::from_utf8(mnemonic_bytes) else {
                    return Err(self.treat_err("Could not decode mnemonic bytes"));
                };
                let Ok(parsed_mnemonic) = Mnemonic::parse(&mnemonic) else {
                    return Err(self.treat_err("Could not parse mnemonic bytes"));
                };
                tracing::debug!("Received account secret mnemonic, pairing completed");
                Ok(Some(parsed_mnemonic))
            }
            Some(Err(VpnApiClientError::HttpClient(e))) if e.is_timeout() => {
                tracing::debug!("Timeout on checking the pairing");
                Ok(None)
            }
            Some(Err(err)) => Err(self.treat_err(&format!(
                "Error getting the account secret, stopping the queries {err:?}"
            ))),
            None => {
                tracing::debug!("QR get pairing secret loop cancelled");
                self.send_event(AccountControllerPairingResult::Cancelled);
                Err("cancelled".to_string())
            }
        }
    }

    fn spawn_get_pairing_secret_task(
        self,
        remote_qr_pub_key: PublicKey,
        pin_code: String,
        pairing_token: String,
        ttl: Duration,
    ) -> QrPairingProcessHandle {
        let initial_instant = tokio::time::Instant::now();
        let mut interval = tokio::time::interval(RETRY_INTERVAL);
        let cancel_token = self.cancel_token.clone();
        let mut confirmed = false;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = self.cancel_token.cancelled() => {
                        tracing::debug!("QR pairing loop cancelled");
                        self.send_event(AccountControllerPairingResult::Cancelled);
                        break;
                    }
                    _ = interval.tick() => {
                        if initial_instant.elapsed() > ttl {
                            tracing::warn!("QR pairing loop timeout reached");
                            self.send_event(AccountControllerPairingResult::TimedOut);
                            break;
                        }
                        let mnemonic = match self.get_pairing_secret(&remote_qr_pub_key, &pin_code, &pairing_token, &mut confirmed).await {
                            Ok(Some(mnemonic)) => mnemonic,
                            Ok(None) => continue,
                            Err(_) => break,
                        };
                        self.send_event(AccountControllerPairingResult::Mnemonic(mnemonic));
                        break;
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
        qr_pub_key: &str,
    ) -> Result<NymVpnPairingCheckResponse, VpnApiClientError>;
    async fn confirm_pin(
        &self,
        account: &VpnAccount,
        qr_pub_key: &str,
        enc_pin_code: Vec<u8>,
        nonce: Vec<u8>,
    ) -> Result<StatusOk, VpnApiClientError>;
    async fn try_to_pair_to_account(
        &self,
        remote_qr_pub_key: &str,
        local_qr_pub_key: &str,
        encrypted_pin_code: &[u8],
        nonce: &[u8],
        token: &str,
    ) -> Result<NymVpnPairingResponse, VpnApiClientError>;
    async fn try_to_get_secret_account(
        &self,
        remote_qr_pub_key: &str,
        pairing_token: &str,
    ) -> Result<NymVpnGetPairingSecretResponse, VpnApiClientError>;
    async fn share_secret_account(
        &self,
        account: &VpnAccount,
        remote_qr_pub_key: &str,
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
        qr_pub_key: &str,
    ) -> Result<NymVpnPairingCheckResponse, VpnApiClientError> {
        self.check_pairing_account(account, qr_pub_key).await
    }

    async fn confirm_pin(
        &self,
        account: &VpnAccount,
        qr_pub_key: &str,
        enc_pin_code: Vec<u8>,
        nonce: Vec<u8>,
    ) -> Result<StatusOk, VpnApiClientError> {
        self.confirm_pin(account, qr_pub_key, enc_pin_code, nonce)
            .await
    }

    async fn try_to_pair_to_account(
        &self,
        remote_qr_pub_key: &str,
        local_qr_pub_key: &str,
        encrypted_pin_code: &[u8],
        nonce: &[u8],
        token: &str,
    ) -> Result<NymVpnPairingResponse, VpnApiClientError> {
        self.try_to_pair_to_account(
            remote_qr_pub_key,
            local_qr_pub_key,
            encrypted_pin_code,
            nonce,
            token,
        )
        .await
    }

    async fn try_to_get_secret_account(
        &self,
        remote_qr_pub_key: &str,
        pairing_token: &str,
    ) -> Result<NymVpnGetPairingSecretResponse, VpnApiClientError> {
        self.try_to_get_secret_account(remote_qr_pub_key, pairing_token)
            .await
    }

    async fn share_secret_account(
        &self,
        account: &VpnAccount,
        own_pubkey: &str,
        enc_mnemonic: Vec<u8>,
        nonce: Vec<u8>,
    ) -> Result<StatusOk, VpnApiClientError> {
        self.share_secret_account(account, own_pubkey, enc_mnemonic, nonce)
            .await
    }
}

const PIN_LENGTH: usize = 6;

fn random_pin() -> String {
    let mut random = vec![0u8; PIN_LENGTH];
    rand::rng().fill_bytes(&mut random);
    random.iter().map(|n| (n % 10).to_string()).collect()
}

fn is_valid_pin(pin: &[u8]) -> bool {
    pin.len() == PIN_LENGTH && pin.iter().all(u8::is_ascii_digit)
}

fn random_token() -> String {
    let mut random = vec![0u8; 16];
    rand::rng().fill_bytes(&mut random);
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
        event_sender: AccountControllerEventSender,
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
        event_sender: AccountControllerEventSender,
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
        event_sender: AccountControllerEventSender,
        cancel_token: CancellationToken,
    ) -> Result<QrPairingConfirmation, AccountCommandError> {
        let qr_pairing_process = self.new_pairing_process(event_sender, cancel_token);
        let session_id = qr_pairing_process.session_id;

        let pin_code = random_pin();
        let token = random_token();
        let remote_qr_pub_key = PublicKey::from_str(remote_qr_pub_key_str)
            .map_err(|err| AccountCommandError::InvalidPublicKey(err.to_string()))?;
        let shared_key = qr_pairing_process.derive_shared_key(&remote_qr_pub_key, PIN_ENC_INFO)?;

        let nonce = shared_key.random_nonce();
        let enc_pin = shared_key.encrypt(pin_code.as_bytes(), &nonce).ok_or(
            AccountCommandError::Internal(String::from("encrypt mnemonic")),
        )?;

        let response = self
            .vpn_api_client
            .try_to_pair_to_account(
                remote_qr_pub_key_str,
                &qr_pairing_process.qr_public_key(),
                &enc_pin,
                &nonce,
                &token,
            )
            .await?;
        self.process_handle = qr_pairing_process.spawn_get_pairing_secret_task(
            remote_qr_pub_key,
            pin_code,
            token,
            Duration::from_secs(response.ttl_seconds),
        );

        Ok(QrPairingConfirmation { session_id })
    }

    async fn verified_pairing_pin_can_fail(
        &self,
        account: &Arc<VpnAccount>,
        verified_session_id: u32,
    ) -> Result<(), AccountCommandError> {
        let QrPairingProcessHandle::PairingCheck {
            ref shared_key,
            session_id,
            ref qr_pubkey,
            ..
        } = self.process_handle
        else {
            tracing::warn!("Pairing needs to be started first");
            return Err(AccountCommandError::PairingNotStarted);
        };
        if verified_session_id != session_id {
            tracing::error!("Started a new session with a different PIN");
            return Err(AccountCommandError::PairingSessionIdMismatch);
        }
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
            .share_secret_account(account, qr_pubkey, enc_mnemonic, nonce.to_vec())
            .await?;

        Ok(())
    }

    pub(crate) async fn verified_pairing_pin(
        &mut self,
        account: &Arc<VpnAccount>,
        session_id: u32,
    ) -> Result<(), AccountCommandError> {
        let ret = self
            .verified_pairing_pin_can_fail(account, session_id)
            .await;
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

    use nym_http_api_client::{HttpClientError, ResolveError};
    use nym_qr_pairing_requests::{
        NymVpnPairingFinalisedResponse, PairingConfirmPinRequestBody, SharePairSecretBody,
    };
    use tokio::time::Instant;

    use crate::event_sender::AccountControllerEventReceiver;

    use super::*;

    const TTL: Duration = Duration::from_mins(5);
    // The TTL returned by the API is informational: the API and the client each enforce it on
    // their own clock, so around the TTL a process ends either with the client's own timeout or
    // with the API rejecting a poll for an already expired process. Tests only check that the
    // process is still running well within the TTL, and has ended well after it.
    const WITHIN_TTL: Duration = Duration::from_mins(4);
    const PAST_TTL: Duration = Duration::from_mins(6);

    // The mocked API mirrors the pairing service's `PairingManager`
    // (`websites/packages/qr-pairing/rust/pairing-service/src/service/manager.rs`): one pairing
    // process per QR public key, owned by the account that registered it, moving through
    // `Empty -> SharedPin -> ConfirmedPin -> SharedSecret`, and expiring TTL after it was
    // registered or paired.

    struct LoggedOutData {
        pairing_response: NymVpnPairingFinalisedResponse,
        token: String,
    }

    enum PairingState {
        /// Waiting for a logged out device to scan the QR code
        Empty,
        /// Scanned; waiting for the logged in device to confirm it received the PIN
        SharedPin(LoggedOutData),
        /// PIN confirmed; the logged out device checks the confirmation against its own PIN
        /// while the logged in device waits for the user to share the secret
        ConfirmedPin {
            token: String,
            confirmation: PairingConfirmPinRequestBody,
        },
        /// Secret shared; handed out to whoever presents the token from the pairing request
        SharedSecret {
            token: String,
            secret_body: SharePairSecretBody,
        },
    }

    struct PairingProcess {
        account_id: String,
        state: PairingState,
        expires_at: Instant,
    }

    #[derive(Default)]
    struct ApiDb {
        /// Makes the polling requests hang, like a stalled network
        hang_requests: bool,
        hanging_requests: usize,
        /// Makes the API refuse PIN confirmations
        reject_pin_confirmations: bool,
        /// Makes the API keep the next PIN confirmation but let its request time out, like a
        /// reply that never arrives
        time_out_next_pin_confirmation: bool,
        /// Pairing processes by QR public key
        pairing_processes: HashMap<String, PairingProcess>,
    }

    impl ApiDb {
        /// Like `PairingManager::active_pairing_process`: the process must exist and not be
        /// expired, and when an account is given, it must own the process.
        fn active_process(
            &mut self,
            qr_pub_key: &str,
            account_id: Option<&str>,
        ) -> Result<&mut PairingProcess, Box<HttpClientError>> {
            let process = self
                .pairing_processes
                .get_mut(qr_pub_key)
                .filter(|process| process.expires_at > Instant::now())
                .ok_or_else(|| {
                    api_error("404: no active pairing process for the provided QR public key")
                })?;
            if account_id.is_some_and(|account_id| account_id != process.account_id) {
                return Err(api_error("401: the public key is part of another account"));
            }
            Ok(process)
        }

        fn has_shared_secret(&self) -> bool {
            self.pairing_processes
                .values()
                .any(|process| matches!(process.state, PairingState::SharedSecret { .. }))
        }
    }

    fn api_error(message: &str) -> Box<HttpClientError> {
        Box::new(HttpClientError::GenericRequestFailure(message.to_string()))
    }

    /// A timed out request; a DNS lookup timeout is the only one that can be made without
    /// sending a real request
    fn timeout_error() -> Box<HttpClientError> {
        Box::new(HttpClientError::DnsLookupFailure {
            inner: ResolveError::Timeout,
        })
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

            // registering a QR key again replaces its process
            db.pairing_processes.insert(
                qr_pub_key.to_string(),
                PairingProcess {
                    account_id: account.id(),
                    state: PairingState::Empty,
                    expires_at: Instant::now() + TTL,
                },
            );

            Ok(NymVpnPairingResponse {
                ttl_seconds: TTL.as_secs(),
            })
        }

        async fn check_pairing_account(
            &self,
            account: &VpnAccount,
            qr_pub_key: &str,
        ) -> Result<NymVpnPairingCheckResponse, VpnApiClientError> {
            let mut db = self.api_db.lock().await;
            if db.hang_requests {
                db.hanging_requests += 1;
                drop(db);
                return std::future::pending().await;
            }

            let process = db
                .active_process(qr_pub_key, Some(&account.id()))
                .map_err(VpnApiClientError::HttpClient)?;
            let inner = match &process.state {
                PairingState::Empty => None,
                PairingState::SharedPin(data) => Some(data.pairing_response.clone()),
                PairingState::ConfirmedPin { .. } | PairingState::SharedSecret { .. } => {
                    return Err(VpnApiClientError::HttpClient(api_error(
                        "400: the process already advanced",
                    )));
                }
            };

            Ok(NymVpnPairingCheckResponse { inner })
        }

        async fn confirm_pin(
            &self,
            account: &VpnAccount,
            qr_pub_key: &str,
            enc_pin_code: Vec<u8>,
            nonce: Vec<u8>,
        ) -> Result<StatusOk, VpnApiClientError> {
            let mut db = self.api_db.lock().await;
            if db.reject_pin_confirmations {
                return Err(VpnApiClientError::HttpClient(api_error(
                    "500: PIN confirmation refused",
                )));
            }

            let account_id = account.id();
            let process = db
                .active_process(qr_pub_key, Some(&account_id))
                .map_err(VpnApiClientError::HttpClient)?;
            let PairingState::SharedPin(data) = &process.state else {
                return Err(VpnApiClientError::HttpClient(api_error(
                    "400: not in the step of PIN confirmation",
                )));
            };
            process.state = PairingState::ConfirmedPin {
                token: data.token.clone(),
                confirmation: PairingConfirmPinRequestBody {
                    account_id,
                    pubkey: qr_pub_key.to_string(),
                    enc_pin_code,
                    nonce,
                },
            };
            if db.time_out_next_pin_confirmation {
                db.time_out_next_pin_confirmation = false;
                return Err(VpnApiClientError::HttpClient(timeout_error()));
            }

            Ok(StatusOk {
                status: "Ok".to_string(),
            })
        }

        async fn try_to_pair_to_account(
            &self,
            remote_qr_pub_key: &str,
            local_qr_pub_key: &str,
            encrypted_pin_code: &[u8],
            nonce: &[u8],
            token: &str,
        ) -> Result<NymVpnPairingResponse, VpnApiClientError> {
            let mut db = self.api_db.lock().await;

            let process = db
                .active_process(remote_qr_pub_key, None)
                .map_err(VpnApiClientError::HttpClient)?;
            if !matches!(process.state, PairingState::Empty) {
                // first come, first served: a second scan might be a MitM attempt
                return Err(VpnApiClientError::HttpClient(api_error(
                    "400: pairing request has already been submitted for this QR public key",
                )));
            }

            process.state = PairingState::SharedPin(LoggedOutData {
                pairing_response: NymVpnPairingFinalisedResponse {
                    encrypted_pin_code: encrypted_pin_code.to_vec(),
                    nonce: nonce.to_vec(),
                    remote_pub_key: local_qr_pub_key.to_string(),
                },
                token: token.to_string(),
            });
            process.expires_at = Instant::now() + TTL;

            Ok(NymVpnPairingResponse {
                ttl_seconds: TTL.as_secs(),
            })
        }

        async fn try_to_get_secret_account(
            &self,
            remote_qr_pub_key: &str,
            pairing_token: &str,
        ) -> Result<NymVpnGetPairingSecretResponse, VpnApiClientError> {
            let mut db = self.api_db.lock().await;
            if db.hang_requests {
                db.hanging_requests += 1;
                drop(db);
                return std::future::pending().await;
            }

            let process = db
                .active_process(remote_qr_pub_key, None)
                .map_err(VpnApiClientError::HttpClient)?;
            let check_token = |token: &str| {
                if pairing_token != token {
                    return Err(VpnApiClientError::HttpClient(api_error(
                        "401: provided token is not correct",
                    )));
                }
                Ok(())
            };
            let response = match &process.state {
                PairingState::Empty => {
                    return Err(VpnApiClientError::HttpClient(api_error(
                        "400: pairing process not started",
                    )));
                }
                PairingState::SharedPin(_) => NymVpnGetPairingSecretResponse::Nothing,
                PairingState::ConfirmedPin {
                    token,
                    confirmation,
                } => {
                    check_token(token)?;
                    NymVpnGetPairingSecretResponse::PinConfirmation(confirmation.clone())
                }
                PairingState::SharedSecret { token, secret_body } => {
                    check_token(token)?;
                    NymVpnGetPairingSecretResponse::Secret(secret_body.clone())
                }
            };

            Ok(response)
        }

        async fn share_secret_account(
            &self,
            account: &VpnAccount,
            qr_pub_key: &str,
            enc_mnemonic: Vec<u8>,
            nonce: Vec<u8>,
        ) -> Result<StatusOk, VpnApiClientError> {
            let mut db = self.api_db.lock().await;

            let account_id = account.id();
            let process = db
                .active_process(qr_pub_key, Some(&account_id))
                .map_err(VpnApiClientError::HttpClient)?;
            let token = match &process.state {
                PairingState::Empty => {
                    return Err(VpnApiClientError::HttpClient(api_error(
                        "400: process was not started",
                    )));
                }
                PairingState::SharedPin(_) => {
                    return Err(VpnApiClientError::HttpClient(api_error(
                        "400: pin should be confirmed first",
                    )));
                }
                PairingState::ConfirmedPin { token, .. } => token.clone(),
                PairingState::SharedSecret { .. } => {
                    return Err(VpnApiClientError::HttpClient(api_error(
                        "400: secret was already shared",
                    )));
                }
            };
            process.state = PairingState::SharedSecret {
                token,
                secret_body: SharePairSecretBody {
                    own_pubkey: qr_pub_key.to_string(),
                    enc_mnemonic,
                    nonce,
                    account_id,
                },
            };

            Ok(StatusOk {
                status: "Ok".to_string(),
            })
        }
    }

    #[tokio::test(start_paused = true)]
    async fn happy_flow() {
        let account = Arc::new(VpnAccount::generate_new().unwrap().0);
        let cancel_token = CancellationToken::new();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));

        let logged_in_client = MockedApiClient::new(api_db.clone());
        let mut logged_in_handler = QrPairingHandler::new(logged_in_client);
        let logged_in_event_sender = AccountControllerEventSender::new();
        let mut logged_in_events = logged_in_event_sender.subscribe();

        let logged_out_client = MockedApiClient::new(api_db.clone());
        let mut logged_out_handler = QrPairingHandler::new(logged_out_client);
        let logged_out_event_sender = AccountControllerEventSender::new();
        let mut logged_out_events = logged_out_event_sender.subscribe();

        // user triggers the pairing process by opening the pairing page
        let url = logged_in_handler
            .pair_account(
                &account,
                logged_in_event_sender.clone(),
                cancel_token.child_token(),
            )
            .await
            .unwrap()
            .url;

        // the URL gets encoded into a QR code that gets scanned by the logged out device
        // obtaining the public key needed for the pairing process
        let qr_pub_key = url.split("pubkey=").last().unwrap();

        // the logged out device generates a PIN code and sends it to the API, encrypted with the
        // key it derives from the scanned public key
        logged_out_handler
            .scanned_key(
                qr_pub_key,
                logged_out_event_sender.clone(),
                cancel_token.child_token(),
            )
            .await
            .unwrap();

        // the logged in device decrypts the PIN code and confirms it to the API, re-encrypted,
        // before showing it to the user
        let AccountControllerEvent::Pairing(AccountControllerPairingEvent {
            result: AccountControllerPairingResult::PinCode(received_pin_code),
            session_id,
        }) = logged_in_events.recv().await.unwrap()
        else {
            panic!("no pin code event")
        };
        assert!(matches!(
            api_db.lock().await.pairing_processes[qr_pub_key].state,
            PairingState::ConfirmedPin { .. }
        ));

        // the logged out device only shows its PIN code once it gets the confirmation back and
        // checks it against the PIN code it generated
        let AccountControllerEvent::Pairing(AccountControllerPairingEvent {
            result: AccountControllerPairingResult::PinCode(generated_pin_code),
            ..
        }) = logged_out_events.recv().await.unwrap()
        else {
            panic!("no pin code confirmation event")
        };

        // the user checks that both devices show the same PIN code
        assert_eq!(generated_pin_code, received_pin_code);

        // after PINs were cross checked, the logged in device initiates sharing of the secret
        logged_in_handler
            .verified_pairing_pin(&account, session_id)
            .await
            .unwrap();

        // the logged out device receives the mnemonic in the form of an event
        let AccountControllerEvent::Pairing(AccountControllerPairingEvent {
            result: AccountControllerPairingResult::Mnemonic(mnemonic),
            ..
        }) = logged_out_events.recv().await.unwrap()
        else {
            panic!("no mnemonic received")
        };

        // check that the correct mnemonic was shared between devices
        assert_eq!(*account.get_mnemonic(), mnemonic.to_string());
    }

    type TestHandler = QrPairingHandler<MockedApiClient>;

    fn new_handler(
        api_db: &Arc<Mutex<ApiDb>>,
    ) -> (
        TestHandler,
        AccountControllerEventSender,
        AccountControllerEventReceiver,
    ) {
        let event_tx = AccountControllerEventSender::new();
        let event_rx = event_tx.subscribe();
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

    /// A process playing a MitM between the two devices
    fn new_mitm(api_db: &Arc<Mutex<ApiDb>>) -> QrPairingProcess<MockedApiClient> {
        QrPairingProcess::new(
            MockedApiClient::new(api_db.clone()),
            AccountControllerEventSender::new(),
            CancellationToken::new(),
            0,
        )
    }

    /// The MitM's own pairing request: its public key, and a PIN the logged in device can decrypt
    fn mitm_pairing_request(
        mitm: &QrPairingProcess<MockedApiClient>,
        qr_pub_key: &str,
        pin: &str,
    ) -> NymVpnPairingFinalisedResponse {
        let key = mitm
            .derive_shared_key(&PublicKey::from_str(qr_pub_key).unwrap(), PIN_ENC_INFO)
            .unwrap();
        let nonce = key.random_nonce();
        NymVpnPairingFinalisedResponse {
            encrypted_pin_code: key.encrypt(pin.as_bytes(), &nonce).unwrap(),
            nonce: nonce.to_vec(),
            remote_pub_key: mitm.qr_public_key(),
        }
    }

    /// Waits for the next pairing event, checking that it comes from the expected session
    async fn next_pairing_event(
        event_rx: &mut AccountControllerEventReceiver,
        session_id: u32,
    ) -> AccountControllerPairingResult {
        let AccountControllerEvent::Pairing(event) = event_rx.recv().await.unwrap() else {
            panic!("Wrong type of event")
        };
        assert_eq!(event.session_id, session_id, "{event:?}");
        event.result
    }

    /// Waits for the next pairing event of each of the given sessions, which share the channel
    /// and can report in any order
    async fn next_pairing_events<const N: usize>(
        event_rx: &mut AccountControllerEventReceiver,
        session_ids: [u32; N],
    ) -> [AccountControllerPairingResult; N] {
        let mut results = [const { None }; N];
        while results.iter().any(Option::is_none) {
            let AccountControllerEvent::Pairing(event) = event_rx.recv().await.unwrap() else {
                panic!("Wrong type of event")
            };
            let Some(index) = session_ids.iter().position(|id| *id == event.session_id) else {
                panic!("event from an unexpected session: {event:?}")
            };
            assert!(results[index].is_none(), "unexpected event: {event:?}");
            results[index] = Some(event.result);
        }
        results.map(Option::unwrap)
    }

    /// Waits for both devices to show the PIN code, checking that it's the same one
    async fn expect_matching_pins(
        logged_in_event_rx: &mut AccountControllerEventReceiver,
        logged_in_session_id: u32,
        logged_out_event_rx: &mut AccountControllerEventReceiver,
        logged_out_session_id: u32,
    ) -> String {
        let event = next_pairing_event(logged_in_event_rx, logged_in_session_id).await;
        let AccountControllerPairingResult::PinCode(logged_in_pin) = event else {
            panic!("unexpected event on the logged in device: {event:?}")
        };
        let event = next_pairing_event(logged_out_event_rx, logged_out_session_id).await;
        let AccountControllerPairingResult::PinCode(logged_out_pin) = event else {
            panic!("unexpected event on the logged out device: {event:?}")
        };
        assert_eq!(logged_in_pin, logged_out_pin);
        logged_in_pin
    }

    async fn assert_no_event_for(event_rx: &mut AccountControllerEventReceiver, wait: Duration) {
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
    async fn assert_channel_closed(event_rx: &mut AccountControllerEventReceiver) {
        let event = tokio::time::timeout(RETRY_INTERVAL * 2, event_rx.recv())
            .await
            .expect("channel still open");
        assert!(
            matches!(event, Err(tokio::sync::broadcast::error::RecvError::Closed)),
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

        // nothing should happen well within the TTL
        assert_no_event_for(&mut logged_in_event_rx, WITHIN_TTL).await;

        // the check ends around the TTL, timed out by the client or expired by the API
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(
            matches!(
                event,
                AccountControllerPairingResult::TimedOut | AccountControllerPairingResult::Error(_)
            ),
            "{event:?}"
        );
        assert!(started.elapsed() <= PAST_TTL);

        // the shared key was never derived, so the PIN can't be verified
        let err = logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::NoPairingSharedSecret);

        // the failed verification also discards the ended process
        let err = logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
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

        expect_matching_pins(
            &mut logged_in_event_rx,
            url.session_id,
            &mut logged_out_event_rx,
            confirmation.session_id,
        )
        .await;

        // the user never verifies the PIN on the logged in device
        assert_no_event_for(&mut logged_out_event_rx, WITHIN_TTL - started.elapsed()).await;

        // the secret query ends around the TTL, timed out by the client or expired by the API
        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        assert!(
            matches!(
                event,
                AccountControllerPairingResult::TimedOut | AccountControllerPairingResult::Error(_)
            ),
            "{event:?}"
        );
        assert!(started.elapsed() <= PAST_TTL);
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
        assert!(
            matches!(event, AccountControllerPairingResult::Cancelled),
            "{event:?}"
        );
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
        assert!(
            matches!(event, AccountControllerPairingResult::Cancelled),
            "{event:?}"
        );

        let err = logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
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
        expect_matching_pins(
            &mut logged_in_event_rx,
            url.session_id,
            &mut logged_out_event_rx,
            confirmation.session_id,
        )
        .await;

        // the logged out device runs a secret query, not a pairing check
        logged_out_handler.cancel_pairing_check();
        assert_no_event_for(&mut logged_out_event_rx, RETRY_INTERVAL * 2).await;

        logged_out_handler.cancel_get_pairing_secret();
        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        assert!(
            matches!(event, AccountControllerPairingResult::Cancelled),
            "{event:?}"
        );
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
        assert!(
            matches!(event, AccountControllerPairingResult::Cancelled),
            "{event:?}"
        );
        assert_channel_closed(&mut logged_in_event_rx).await;

        let err = logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
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
        expect_matching_pins(
            &mut logged_in_event_rx,
            url.session_id,
            &mut logged_out_event_rx,
            confirmation.session_id,
        )
        .await;

        // the user closes the pairing window on the logged out device
        logged_out_handler.cancel_pairing_process();
        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        assert!(
            matches!(event, AccountControllerPairingResult::Cancelled),
            "{event:?}"
        );

        // the secret shared afterwards is never picked up
        logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
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
        expect_matching_pins(
            &mut logged_in_event_rx,
            url.session_id,
            &mut logged_out_event_rx,
            confirmation.session_id,
        )
        .await;
        logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
            .await
            .unwrap();
        let AccountControllerPairingResult::Mnemonic(mnemonic) =
            next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await
        else {
            panic!("no mnemonic received")
        };
        assert_eq!(*account.get_mnemonic(), mnemonic.to_string());

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
        assert!(
            matches!(event, AccountControllerPairingResult::Cancelled),
            "{event:?}"
        );
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
        expect_matching_pins(
            &mut logged_in_event_rx,
            url.session_id,
            &mut logged_out_event_rx,
            confirmation.session_id,
        )
        .await;

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
        assert!(
            matches!(event, AccountControllerPairingResult::Cancelled),
            "{event:?}"
        );
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
        assert!(
            matches!(event, AccountControllerPairingResult::PinCode(_)),
            "{event:?}"
        );

        // the PINs don't match, so the user closes the pairing window
        logged_in_handler.cancel_pairing_process();

        // the check had already finished, so there is nothing left to report
        assert_channel_closed(&mut logged_in_event_rx).await;

        // and the mnemonic can't be shared anymore
        let err = logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::PairingNotStarted);
        assert!(!api_db.lock().await.has_shared_secret());
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
            expect_matching_pins(
                &mut logged_in_event_rx,
                url.session_id,
                &mut logged_out_event_rx,
                confirmation.session_id,
            )
            .await;
            logged_in_handler
                .verified_pairing_pin(&account, url.session_id)
                .await
                .unwrap();

            // the next secret query waits on the API, which already has the mnemonic
            let db = api_db.lock().await;
            tokio::time::sleep(RETRY_INTERVAL + Duration::from_millis(1)).await;

            // the user closes the window just as the API answers
            logged_out_handler.cancel_pairing_process();
            drop(db);

            let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
            assert!(
                matches!(event, AccountControllerPairingResult::Cancelled),
                "{event:?}"
            );
            assert_channel_closed(&mut logged_out_event_rx).await;
        }
    }

    #[tokio::test]
    async fn verify_pin_before_pairing_started() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, _, _) = new_handler(&api_db);

        let err = logged_in_handler
            .verified_pairing_pin(&account, 0)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::PairingNotStarted);
        assert!(!api_db.lock().await.has_shared_secret());
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
            .verified_pairing_pin(&account, url.session_id)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::NoPairingSharedSecret);
        assert!(!api_db.lock().await.has_shared_secret());

        // verifying too early aborts the pairing
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(
            matches!(event, AccountControllerPairingResult::Cancelled),
            "{event:?}"
        );
        let err = logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
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
        assert!(
            matches!(event, AccountControllerPairingResult::PinCode(_)),
            "{event:?}"
        );

        logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
            .await
            .unwrap();
        let err = logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
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
            .verified_pairing_pin(&account, url.session_id)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::PairingNotStarted);

        // and the secret query is aborted
        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        assert!(
            matches!(event, AccountControllerPairingResult::Cancelled),
            "{event:?}"
        );
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
        let mut rng = rand::rng();
        let unknown_key = KeyPair::new(&mut rng).public_key().to_string();

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
        expect_matching_pins(
            &mut logged_in_event_rx,
            url.session_id,
            &mut logged_out_event_rx,
            confirmation.session_id,
        )
        .await;
        logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
            .await
            .unwrap();
        let AccountControllerPairingResult::Mnemonic(mnemonic) =
            next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await
        else {
            panic!("no mnemonic received")
        };
        assert_eq!(*account.get_mnemonic(), mnemonic.to_string());
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
        let AccountControllerPairingResult::Error(err) = event else {
            panic!("unexpected event: {event:?}")
        };
        assert!(err.starts_with("Error checking QR pairing"), "{err}");
        assert!(err.contains("no active pairing process"), "{err}");
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
        let AccountControllerPairingResult::Error(err) = event else {
            panic!("unexpected event: {event:?}")
        };
        assert!(err.starts_with("Error getting the account secret"), "{err}");
        assert!(err.contains("no active pairing process"), "{err}");
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
        expect_matching_pins(
            &mut logged_in_event_rx,
            url.session_id,
            &mut logged_out_event_rx,
            confirmation.session_id,
        )
        .await;

        // a secret not encrypted with the DH shared key ends up on the API
        let mut rng = rand::rng();
        let nonce = random_nonce::<Aes256GcmSiv, _>(&mut rng);
        MockedApiClient::new(api_db.clone())
            .share_secret_account(
                &account,
                pub_key_from_url(&url),
                vec![0u8; 64],
                nonce.to_vec(),
            )
            .await
            .unwrap();

        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        assert!(
            matches!(event, AccountControllerPairingResult::Error(_)),
            "{event:?}"
        );
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
        expect_matching_pins(
            &mut logged_in_event_rx,
            url.session_id,
            &mut logged_out_event_rx,
            confirmation.session_id,
        )
        .await;

        MockedApiClient::new(api_db.clone())
            .share_secret_account(
                &account,
                pub_key_from_url(&url),
                vec![0u8; 64],
                vec![0u8; 3],
            )
            .await
            .unwrap();

        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        assert!(
            matches!(event, AccountControllerPairingResult::Error(_)),
            "{event:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn confirmation_of_another_pin_is_reported() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        // the logged in device is played by hand, to confirm a PIN it was never sent
        let api_client = MockedApiClient::new(api_db.clone());
        let logged_in_process = QrPairingProcess::new(
            api_client.clone(),
            AccountControllerEventSender::new(),
            CancellationToken::new(),
            0,
        );
        let qr_pub_key = logged_in_process.qr_public_key();
        api_client
            .pair_account(&account, &qr_pub_key)
            .await
            .unwrap();

        let confirmation = logged_out_handler
            .scanned_key(&qr_pub_key, logged_out_event_tx, CancellationToken::new())
            .await
            .unwrap();

        // the logged out device shows nothing until its PIN gets confirmed
        assert_no_event_for(&mut logged_out_event_rx, RETRY_INTERVAL * 2).await;

        let pairing_request = api_client
            .check_pairing_account(&account, &qr_pub_key)
            .await
            .unwrap()
            .inner
            .unwrap();
        let confirmation_key = logged_in_process
            .derive_shared_key(
                &PublicKey::from_str(&pairing_request.remote_pub_key).unwrap(),
                PIN_RE_ENC_INFO,
            )
            .unwrap();
        let nonce = confirmation_key.random_nonce();
        api_client
            .confirm_pin(
                &account,
                &qr_pub_key,
                confirmation_key.encrypt(b"not the PIN", &nonce).unwrap(),
                nonce.to_vec(),
            )
            .await
            .unwrap();

        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        let AccountControllerPairingResult::Error(err) = event else {
            panic!("unexpected event: {event:?}")
        };
        assert!(err.starts_with("PIN not confirmed"), "{err}");
        assert_channel_closed(&mut logged_out_event_rx).await;
    }

    #[tokio::test(start_paused = true)]
    async fn pin_confirmed_through_mitm_is_reported() {
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
        let qr_pub_key = pub_key_from_url(&url);
        let confirmation = logged_out_handler
            .scanned_key(qr_pub_key, logged_out_event_tx, CancellationToken::new())
            .await
            .unwrap();

        // before the logged in device picks up the pairing request, a MitM swaps in its own key
        // and PIN, to be able to decrypt what the logged in device sends next
        let mitm = new_mitm(&api_db);
        let mitm_pin = "123456";
        {
            let mut db = api_db.lock().await;
            let process = db.pairing_processes.get_mut(qr_pub_key).unwrap();
            let PairingState::SharedPin(data) = &mut process.state else {
                panic!("the pairing request was already picked up")
            };
            data.pairing_response = mitm_pairing_request(&mitm, qr_pub_key, mitm_pin);
        }

        // the logged in device shows and confirms the MitM's PIN
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        let AccountControllerPairingResult::PinCode(pin) = event else {
            panic!("unexpected event: {event:?}")
        };
        assert_eq!(pin, mitm_pin);

        // but the logged out device can't decrypt that confirmation, so it never shows a PIN the
        // user could take for a match
        let event = next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await;
        assert!(
            matches!(event, AccountControllerPairingResult::Error(_)),
            "{event:?}"
        );
        assert_channel_closed(&mut logged_out_event_rx).await;
    }

    #[tokio::test(start_paused = true)]
    async fn pin_reflected_as_confirmation_is_reported() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        // the logged in device checks at t=0, 5, 10, ... and the logged out device, scanning at
        // t=2.5, polls at t=2.5, 7.5, ..., which leaves the MitM time to act in between
        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        let qr_pub_key = pub_key_from_url(&url);
        tokio::time::sleep(RETRY_INTERVAL / 2).await;
        let logged_out_session = logged_out_handler
            .scanned_key(qr_pub_key, logged_out_event_tx, CancellationToken::new())
            .await
            .unwrap()
            .session_id;

        // a MitM swaps in its own key and PIN, keeping the logged out device's encrypted PIN
        let mitm = new_mitm(&api_db);
        let mitm_pin = "123456";
        let genuine_request = {
            let mut db = api_db.lock().await;
            let process = db.pairing_processes.get_mut(qr_pub_key).unwrap();
            let PairingState::SharedPin(data) = &mut process.state else {
                panic!("the pairing request was already picked up")
            };
            std::mem::replace(
                &mut data.pairing_response,
                mitm_pairing_request(&mitm, qr_pub_key, mitm_pin),
            )
        };

        // t=5: the logged in device shows and confirms the MitM's PIN
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        let AccountControllerPairingResult::PinCode(pin) = event else {
            panic!("unexpected event: {event:?}")
        };
        assert_eq!(pin, mitm_pin);

        // the MitM can't read the logged out device's PIN, but it can send the encrypted PIN back
        // to that device as the confirmation
        {
            let mut db = api_db.lock().await;
            let process = db.pairing_processes.get_mut(qr_pub_key).unwrap();
            let PairingState::ConfirmedPin { confirmation, .. } = &mut process.state else {
                panic!("the PIN wasn't confirmed")
            };
            confirmation.enc_pin_code = genuine_request.encrypted_pin_code;
            confirmation.nonce = genuine_request.nonce;
        }

        // t=7.5: confirmations are encrypted with another key than PINs, so the logged out device
        // doesn't take its own PIN for a confirmation
        let event = next_pairing_event(&mut logged_out_event_rx, logged_out_session).await;
        let AccountControllerPairingResult::Error(err) = event else {
            panic!("unexpected event: {event:?}")
        };
        assert_eq!(err, "Could not decrypt pin code");
        assert_channel_closed(&mut logged_out_event_rx).await;
    }

    #[tokio::test(start_paused = true)]
    async fn confirmation_retry_ignores_a_swapped_pairing_request() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        api_db.lock().await.time_out_next_pin_confirmation = true;
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);

        // the logged in device checks at t=0, 5, 10, ... and the logged out device, scanning at
        // t=2.5, polls at t=2.5, 7.5, ..., which leaves the MitM time to act in between
        let url = logged_in_handler
            .pair_account(&account, logged_in_event_tx, CancellationToken::new())
            .await
            .unwrap();
        let qr_pub_key = pub_key_from_url(&url);
        tokio::time::sleep(RETRY_INTERVAL / 2).await;
        let logged_out_session = logged_out_handler
            .scanned_key(qr_pub_key, logged_out_event_tx, CancellationToken::new())
            .await
            .unwrap()
            .session_id;

        // t=5: the logged in device confirms the genuine PIN; a MitM keeps the confirmation but
        // withholds the reply, so the request times out
        // t=7.5: the logged out device gets the confirmation and shows its PIN
        let event = next_pairing_event(&mut logged_out_event_rx, logged_out_session).await;
        let AccountControllerPairingResult::PinCode(logged_out_pin) = event else {
            panic!("unexpected event: {event:?}")
        };
        assert_no_event_for(&mut logged_in_event_rx, Duration::ZERO).await;

        // the MitM puts its own pairing request up for the logged in device's next check
        let mitm = new_mitm(&api_db);
        {
            let mut db = api_db.lock().await;
            let process = db.pairing_processes.get_mut(qr_pub_key).unwrap();
            let PairingState::ConfirmedPin { token, .. } = &process.state else {
                panic!("the PIN wasn't confirmed")
            };
            process.state = PairingState::SharedPin(LoggedOutData {
                pairing_response: mitm_pairing_request(&mitm, qr_pub_key, "123456"),
                token: token.clone(),
            });
        }

        // t=10: the logged in device retries the confirmation for the pairing request it already
        // decrypted, instead of checking for a new one, so it shows the same PIN
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        let AccountControllerPairingResult::PinCode(logged_in_pin) = event else {
            panic!("unexpected event: {event:?}")
        };
        assert_eq!(logged_in_pin, logged_out_pin);

        // and the secret is shared with the logged out device, not with the MitM
        logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
            .await
            .unwrap();
        {
            let db = api_db.lock().await;
            let PairingState::SharedSecret { secret_body, .. } =
                &db.pairing_processes[qr_pub_key].state
            else {
                panic!("the secret wasn't shared")
            };
            let mitm_key = mitm
                .derive_shared_key(&PublicKey::from_str(qr_pub_key).unwrap(), MNEMONIC_ENC_INFO)
                .unwrap();
            let nonce = SharedSymmetricKey::validate_aead_nonce(&secret_body.nonce).unwrap();
            let decrypted = mitm_key.decrypt(&secret_body.enc_mnemonic, &nonce);
            assert!(decrypted.is_none(), "the MitM decrypted the secret");
        }
        let AccountControllerPairingResult::Mnemonic(mnemonic) =
            next_pairing_event(&mut logged_out_event_rx, logged_out_session).await
        else {
            panic!("no mnemonic received")
        };
        assert_eq!(*account.get_mnemonic(), mnemonic.to_string());
    }

    #[tokio::test(start_paused = true)]
    async fn text_instead_of_pin_is_reported() {
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
        let qr_pub_key = pub_key_from_url(&url);
        logged_out_handler
            .scanned_key(qr_pub_key, logged_out_event_tx, CancellationToken::new())
            .await
            .unwrap();

        // before the logged in device picks up the pairing request, a MitM swaps in its own key,
        // with a message to the user in place of the PIN
        let mitm = new_mitm(&api_db);
        {
            let mut db = api_db.lock().await;
            let process = db.pairing_processes.get_mut(qr_pub_key).unwrap();
            let PairingState::SharedPin(data) = &mut process.state else {
                panic!("the pairing request was already picked up")
            };
            data.pairing_response =
                mitm_pairing_request(&mitm, qr_pub_key, "Codes match \u{2713} Tap Approve");
        }

        // the logged in device neither shows nor confirms it
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        let AccountControllerPairingResult::Error(err) = event else {
            panic!("unexpected event: {event:?}")
        };
        assert_eq!(err, "Invalid PIN code");
        assert_channel_closed(&mut logged_in_event_rx).await;

        // so there's nothing to approve
        let err = logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::NoPairingSharedSecret);
        assert!(!api_db.lock().await.has_shared_secret());

        // and the logged out device keeps waiting for a confirmation
        assert_no_event_for(&mut logged_out_event_rx, RETRY_INTERVAL * 2).await;
    }

    #[tokio::test(start_paused = true)]
    async fn pin_is_not_shown_when_confirmation_is_refused() {
        let account = new_account();
        let api_db = Arc::new(Mutex::new(ApiDb::default()));
        let (mut logged_in_handler, logged_in_event_tx, mut logged_in_event_rx) =
            new_handler(&api_db);
        let (mut logged_out_handler, logged_out_event_tx, mut logged_out_event_rx) =
            new_handler(&api_db);
        api_db.lock().await.reject_pin_confirmations = true;

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

        // the logged in device doesn't show a PIN that the logged out device would never show
        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        let AccountControllerPairingResult::Error(err) = event else {
            panic!("unexpected event: {event:?}")
        };
        assert!(err.starts_with("Confirming PIN"), "{err}");
        assert_channel_closed(&mut logged_in_event_rx).await;

        // so the secret can't be shared
        let err = logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
            .await
            .unwrap_err();
        assert_eq!(err, AccountCommandError::NoPairingSharedSecret);
        assert!(!api_db.lock().await.has_shared_secret());

        // and the logged out device keeps waiting for a confirmation
        assert_no_event_for(&mut logged_out_event_rx, RETRY_INTERVAL * 2).await;
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
            .try_to_pair_to_account(pub_key_from_url(&url), "not-a-public-key", &[], &[], "")
            .await
            .unwrap();

        let event = next_pairing_event(&mut logged_in_event_rx, url.session_id).await;
        assert!(
            matches!(event, AccountControllerPairingResult::Error(_)),
            "{event:?}"
        );

        let err = logged_in_handler
            .verified_pairing_pin(&account, url.session_id)
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
        assert!(
            matches!(event, AccountControllerPairingResult::Cancelled),
            "{event:?}"
        );

        let confirmation = logged_out_handler
            .scanned_key(
                pub_key_from_url(&second_url),
                logged_out_event_tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        expect_matching_pins(
            &mut logged_in_event_rx,
            second_url.session_id,
            &mut logged_out_event_rx,
            confirmation.session_id,
        )
        .await;

        logged_in_handler
            .verified_pairing_pin(&account, second_url.session_id)
            .await
            .unwrap();
        let AccountControllerPairingResult::Mnemonic(mnemonic) =
            next_pairing_event(&mut logged_out_event_rx, confirmation.session_id).await
        else {
            panic!("no mnemonic received")
        };
        assert_eq!(*account.get_mnemonic(), mnemonic.to_string());

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
        expect_matching_pins(
            &mut logged_in_event_rx,
            first_url.session_id,
            &mut logged_out_event_rx,
            first_confirmation.session_id,
        )
        .await;

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

        // the stale query reports its cancellation under its own session, while the new one
        // shows the new PIN code once it's confirmed
        let [stale, fresh] = next_pairing_events(
            &mut logged_out_event_rx,
            [
                first_confirmation.session_id,
                second_confirmation.session_id,
            ],
        )
        .await;
        assert!(
            matches!(stale, AccountControllerPairingResult::Cancelled),
            "{stale:?}"
        );
        let AccountControllerPairingResult::PinCode(logged_out_pin) = fresh else {
            panic!("unexpected event: {fresh:?}")
        };

        let event = next_pairing_event(&mut logged_in_event_rx, second_url.session_id).await;
        let AccountControllerPairingResult::PinCode(logged_in_pin) = event else {
            panic!("unexpected event: {event:?}")
        };
        assert_eq!(logged_in_pin, logged_out_pin);
        logged_in_handler
            .verified_pairing_pin(&account, second_url.session_id)
            .await
            .unwrap();
        let AccountControllerPairingResult::Mnemonic(mnemonic) =
            next_pairing_event(&mut logged_out_event_rx, second_confirmation.session_id).await
        else {
            panic!("no mnemonic received")
        };
        assert_eq!(*account.get_mnemonic(), mnemonic.to_string());
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
        assert!(
            matches!(event, AccountControllerPairingResult::Cancelled),
            "{event:?}"
        );
    }

    #[test]
    fn pin_generation() {
        let pin = random_pin();
        assert!(is_valid_pin(pin.as_bytes()), "{pin}");
    }

    #[test]
    fn pin_validation() {
        assert!(is_valid_pin(b"012345"));
        for invalid in [
            &b""[..],
            b"12345",
            b"1234567",
            b"12345a",
            b"12 345",
            "Codes match \u{2713} Tap Approve".as_bytes(),
        ] {
            assert!(!is_valid_pin(invalid), "{invalid:?}");
        }
    }
}
