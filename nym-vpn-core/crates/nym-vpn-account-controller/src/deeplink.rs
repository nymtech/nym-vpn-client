use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use hkdf::Hkdf;
use nym_crypto::asymmetric::x25519::{KeyPair, PublicKey};
use nym_vpn_lib_types::DeeplinkKind;
use rand::{RngCore, rngs::OsRng};
use sha2::Sha256;
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use url::Url;

pub struct Deeplink {
    id: u64,
    kind: DeeplinkKind,
    _name: String,
    keypair: KeyPair,
    expiry_time: Instant,
}

impl Deeplink {
    const TTL_SECS: u64 = 300;

    pub fn new(params: &CreateDeeplinkParams) -> Self {
        let mut rng = OsRng;
        let keypair = KeyPair::new(&mut rng);
        let id = rng.next_u64();
        let expiry_time = Instant::now() + Duration::from_secs(Self::TTL_SECS);

        Self {
            id,
            kind: params.kind,
            _name: params.name.clone(),
            keypair,
            expiry_time,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expiry_time
    }

    pub fn create_url(&self, base_url: &Url) -> Url {
        let deeplink_id = self.id.to_string();
        let pubkey = bs58::encode(self.keypair.public_key().to_bytes()).into_string();
        let existing_user = match self.kind {
            DeeplinkKind::Privy => "0",
            DeeplinkKind::PrivyAssociate => "1",
        };
        let mut url = base_url.clone();
        url.query_pairs_mut()
            .append_pair("deeplink_id", &deeplink_id)
            .append_pair("pubkey", &pubkey)
            .append_pair("existing_user", existing_user);
        url
    }
}

#[derive(Default)]
pub struct Deeplinks(HashMap<u64, Deeplink>);

impl Deeplinks {
    pub fn create_deeplink(
        &mut self,
        params: &CreateDeeplinkParams,
    ) -> Result<&Deeplink, DeeplinkError> {
        let deeplink = Deeplink::new(params);
        let id = deeplink.id;
        self.0.insert(id, deeplink);
        self.0.get(&id).ok_or(DeeplinkError::DeeplinkNotFound(id))
    }

    pub fn remove_expired(&mut self) {
        let now = Instant::now();
        self.0.retain(|_, deeplink| deeplink.expiry_time > now);
    }

    pub fn derive_mnemonic(&mut self, url_str: &str) -> Result<bip39::Mnemonic, DeeplinkError> {
        let url =
            Url::parse(url_str).map_err(|_| DeeplinkError::InvalidUrl(url_str.to_string()))?;

        let url_params: HashMap<String, String> = url
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let Some(deeplink_id_str) = url_params.get("deeplink_id") else {
            return Err(DeeplinkError::MissingDeeplinkId(url_str.to_string()));
        };

        let Ok(deeplink_id) = deeplink_id_str.parse::<u64>() else {
            return Err(DeeplinkError::InvalidDeeplinkId(deeplink_id_str.clone()));
        };

        let Some(payload_b58) = url_params.get("payload") else {
            return Err(DeeplinkError::MissingPayload(url_str.to_string()));
        };

        let Some(deeplink) = self.0.remove(&deeplink_id) else {
            return Err(DeeplinkError::DeeplinkNotFound(deeplink_id));
        };

        if deeplink.is_expired() {
            return Err(DeeplinkError::DeeplinkExpired(deeplink_id));
        }

        let payload_bytes = bs58::decode(payload_b58)
            .into_vec()
            .map_err(|_| DeeplinkError::InvalidPayload("base-58 encoding".to_string()))?;

        if payload_bytes.len() < 60 {
            return Err(DeeplinkError::InvalidPayload("too short".to_string()));
        }

        let sender_public_key_bytes = payload_bytes[0..32].try_into().map_err(|_| {
            DeeplinkError::InvalidPayload("invalid sender public key length".to_string())
        })?;

        let sender_public_key = PublicKey::from_bytes(sender_public_key_bytes)
            .map_err(|_| DeeplinkError::InvalidPayload("invalid sender public key".to_string()))?;

        let cipher_packet = CipherPacket::from_bytes(&payload_bytes[32..])?;

        let decrypted_bytes = cipher_packet.decrypt(&deeplink.keypair, &sender_public_key)?;

        let mnemonic = bip39::Mnemonic::from_entropy(&decrypted_bytes)
            .map_err(|_| DeeplinkError::InvalidPayload("failed to create mnemonic".to_string()))?;

        Ok(mnemonic)
    }
}

#[derive(Clone, Debug)]
pub struct CreateDeeplinkParams {
    pub kind: DeeplinkKind,
    pub name: String,
    pub base_url: Url,
}

#[derive(Debug, Clone)]
struct CipherPacket {
    salt: [u8; 16],
    iv: [u8; 12],
    ciphertext: Vec<u8>, // includes AES-GCM tag at end
}

impl CipherPacket {
    fn from_bytes(b: &[u8]) -> Result<Self, DeeplinkError> {
        if b.len() < 28 {
            return Err(DeeplinkError::InvalidPayload(
                "cipher packet too short".to_string(),
            ));
        }

        let salt = b[0..16]
            .try_into()
            .map_err(|_| DeeplinkError::InvalidPayload("invalid salt length".to_string()))?;

        let iv = b[16..28]
            .try_into()
            .map_err(|_| DeeplinkError::InvalidPayload("invalid iv length".to_string()))?;

        let ciphertext = b[28..].to_vec();

        Ok(Self {
            salt,
            iv,
            ciphertext,
        })
    }

    fn decrypt(
        &self,
        recipient_sk: &KeyPair,
        sender_pk: &PublicKey,
    ) -> Result<Vec<u8>, DeeplinkError> {
        let info = b"nym-deeplink-v1";

        let shared_bytes = recipient_sk.private_key().diffie_hellman(sender_pk);

        let key_bytes = self.derive_aes256gcm_key(&shared_bytes, info)?;

        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|_| DeeplinkError::InvalidPayload("invalid AES256GCM key".to_string()))?;

        let nonce = Nonce::from_slice(&self.iv);

        let plaintext = cipher
            .decrypt(nonce, self.ciphertext.as_ref())
            .map_err(|_| DeeplinkError::InvalidPayload("decryption failed".to_string()))?;

        Ok(plaintext)
    }

    fn derive_aes256gcm_key(
        &self,
        shared_secret_32: &[u8; 32],
        info: &[u8],
    ) -> Result<[u8; 32], DeeplinkError> {
        let hk = Hkdf::<Sha256>::new(Some(&self.salt), shared_secret_32);
        let mut okm = [0u8; 32];
        hk.expand(info, &mut okm)
            .map_err(|_| DeeplinkError::InvalidPayload("failed to expand hkdf data".to_string()))?;
        Ok(okm)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeeplinkError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("missing deeplink_id parameter in URL: {0}")]
    MissingDeeplinkId(String),

    #[error("invalid deeplink_id parameter: {0}")]
    InvalidDeeplinkId(String),

    #[error("missing payload parameter in URL: {0}")]
    MissingPayload(String),

    #[error("invalid payload: {0}")]
    InvalidPayload(String),

    #[error("deeplink with id {0} not found")]
    DeeplinkNotFound(u64),

    #[error("deeplink with id {0} has expired")]
    DeeplinkExpired(u64),
}
