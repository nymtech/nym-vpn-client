use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use hkdf::Hkdf;
use nym_crypto::asymmetric::x25519::{KeyPair, PublicKey};
use nym_vpn_lib_types::{AutologinResponse, DeeplinkKind};
use pbkdf2::pbkdf2_hmac;
use rand::{rngs::OsRng, RngCore};
use sha2::{Sha256, Sha512};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use url::Url;

pub struct Deeplink {
    id: u64,
    kind: DeeplinkKind,
    keypair: KeyPair,
    expiry_time: Instant,
}

impl Deeplink {
    const TTL_SECS: u64 = 300;

    pub fn new(params: &CreateDeeplinkParams) -> Self {
        let mut rng = OsRng;
        let id = rng.next_u64();
        let kind = params.kind;
        let keypair = KeyPair::new(&mut rng);

        let expiry_time = Instant::now() + Duration::from_secs(Self::TTL_SECS);

        // Note: CreateDeeplinkParams.name is not used.

        Self {
            id,
            kind,
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
        let mut url = base_url.clone();
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("deeplink_id", &deeplink_id);
            pairs.append_pair("pubkey", &pubkey);
            match self.kind {
                DeeplinkKind::Privy => {
                    pairs.append_pair("link_account", "0");
                }
                DeeplinkKind::PrivyLink => {
                    pairs.append_pair("link_account", "1");
                }
                _ => {}
            }
        }
        url
    }

    pub fn create_autologin_url(
        &self,
        base_url: &Url,
        mnemonic: String,
    ) -> Result<AutologinResponse, DeeplinkError> {
        let pin_code = PinCode::new(6)?;

        let encrypted_mnemonic = pin_code.encrypt(&mnemonic)?;

        let mut url = base_url.clone();
        url.query_pairs_mut()
            .append_pair("encmn", &encrypted_mnemonic);

        if let Some(redirect) = self.kind.redirect() {
            url.query_pairs_mut().append_pair("redirect", redirect);
        }

        Ok(AutologinResponse {
            url: url.to_string(),
            pin_code: pin_code.code,
        })
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

    pub fn derive_mnemonic(&mut self, url_str: &str) -> Result<DeeplinkMnemonic, DeeplinkError> {
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

        if decrypted_bytes.len() != 32 {
            return Err(DeeplinkError::InvalidPayload(
                "invalid x25519 private key length".to_string(),
            ));
        }

        let mnemonic = bip39::Mnemonic::from_entropy(&decrypted_bytes)
            .map_err(|_| DeeplinkError::InvalidPayload("failed to create mnemonic".to_string()))?;

        Ok(DeeplinkMnemonic {
            mnemonic,
            kind: deeplink.kind,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CreateDeeplinkParams {
    pub kind: DeeplinkKind,
    pub name: String,
    pub base_url: Url,
}

#[derive(Debug)]
pub struct DeeplinkMnemonic {
    pub kind: DeeplinkKind,
    pub mnemonic: bip39::Mnemonic,
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

struct PinCode {
    code: String,
}

impl PinCode {
    const SALT_LEN: usize = 16;
    const IV_LEN: usize = 12;
    const TAG_LEN: usize = 16;
    const KEY_LEN: usize = 32;
    const PBKDF2_ITERATIONS: u32 = 210_000;

    fn new(length: usize) -> Result<Self, DeeplinkError> {
        let mut random = vec![0u8; length];

        OsRng.fill_bytes(&mut random);

        let encoded = bs58::encode(random).into_string();

        if encoded.len() < length {
            return Err(DeeplinkError::InvalidPayload(
                "pin code too short".to_string(),
            ));
        }

        let mut pin = encoded[..length].to_string();

        pin = pin.replace('1', "i").replace('0', "o").to_lowercase();

        Ok(Self { code: pin })
    }

    fn encrypt(&self, message: &str) -> Result<String, DeeplinkError> {
        let mut salt = [0u8; Self::SALT_LEN];
        let mut iv = [0u8; Self::IV_LEN];

        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut iv);

        let key = self.get_key(&self.code, &salt);
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| DeeplinkError::InvalidPayload(e.to_string()))?;
        let nonce = Nonce::from_slice(&iv);

        let encrypted = cipher
            .encrypt(nonce, message.as_bytes())
            .map_err(|e| DeeplinkError::InvalidPayload(e.to_string()))?;

        let split_at = encrypted.len() - Self::TAG_LEN;
        let cipher_text = &encrypted[..split_at];
        let tag = &encrypted[split_at..];

        let mut output =
            Vec::with_capacity(Self::SALT_LEN + Self::IV_LEN + Self::TAG_LEN + cipher_text.len());
        output.extend_from_slice(&salt);
        output.extend_from_slice(&iv);
        output.extend_from_slice(tag);
        output.extend_from_slice(cipher_text);

        Ok(bs58::encode(output).into_string())
    }

    fn get_key(&self, password: &str, salt: &[u8]) -> [u8; Self::KEY_LEN] {
        let mut key = [0u8; Self::KEY_LEN];
        pbkdf2_hmac::<Sha512>(password.as_bytes(), salt, Self::PBKDF2_ITERATIONS, &mut key);
        key
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

#[cfg(test)]
mod tests {
    use super::*;
    use nym_vpn_lib_types::DeeplinkKind;

    fn params(kind: DeeplinkKind) -> CreateDeeplinkParams {
        CreateDeeplinkParams {
            kind,
            name: "test".to_string(),
            base_url: Url::parse("https://account.example/").unwrap(),
        }
    }

    fn query_map(url: &Url) -> HashMap<String, String> {
        url.query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    }

    fn encrypt_mnemonic_payload(recipient: &Deeplink, entropy: &[u8; 32]) -> String {
        let mut rng = OsRng;
        let sender = KeyPair::new(&mut rng);
        let shared = sender
            .private_key()
            .diffie_hellman(recipient.keypair.public_key());
        let mut salt = [0u8; 16];
        let mut iv = [0u8; 12];
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut iv);
        let hk = Hkdf::<Sha256>::new(Some(&salt), &shared);
        let mut key = [0u8; 32];
        hk.expand(b"nym-deeplink-v1", &mut key)
            .expect("hkdf expand");
        let cipher = Aes256Gcm::new_from_slice(&key).expect("aes key");
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&iv), entropy.as_slice())
            .expect("encrypt");
        let mut packet = Vec::with_capacity(32 + 16 + 12 + ciphertext.len());
        packet.extend_from_slice(&sender.public_key().to_bytes());
        packet.extend_from_slice(&salt);
        packet.extend_from_slice(&iv);
        packet.extend_from_slice(&ciphertext);
        bs58::encode(packet).into_string()
    }

    #[test]
    fn deeplink_create_url_query_pairs() {
        for (kind, link_account) in [
            (DeeplinkKind::Privy, Some("0")),
            (DeeplinkKind::PrivyLink, Some("1")),
            (DeeplinkKind::CreateAccount, None),
        ] {
            let deeplink = Deeplink::new(&params(kind));
            let q = query_map(&deeplink.create_url(&params(kind).base_url));
            assert!(q
                .get("deeplink_id")
                .and_then(|id| id.parse::<u64>().ok())
                .is_some());
            assert!(!q.get("pubkey").map(String::as_str).unwrap_or("").is_empty());
            assert_eq!(q.get("link_account").map(String::as_str), link_account);
        }
    }

    #[test]
    fn deeplink_fresh_is_not_expired() {
        assert!(!Deeplink::new(&params(DeeplinkKind::Privy)).is_expired());
    }

    #[test]
    fn deeplink_autologin_query_and_pin() {
        for kind in [
            DeeplinkKind::AutologinView,
            DeeplinkKind::AutologinRenew,
            DeeplinkKind::Privy,
        ] {
            let deeplink = Deeplink::new(&params(kind));
            let autologin = deeplink
                .create_autologin_url(&params(kind).base_url, "test-mnemonic".to_string())
                .unwrap();
            let q = query_map(&Url::parse(&autologin.url).unwrap());
            assert!(!q.get("encmn").map(String::as_str).unwrap_or("").is_empty());
            assert_eq!(q.get("redirect").map(String::as_str), kind.redirect());
            assert_eq!(autologin.pin_code.len(), 6);
            assert!(!autologin.pin_code.chars().any(|c| c == '1' || c == '0'));
            assert_eq!(autologin.pin_code, autologin.pin_code.to_lowercase());
        }
    }

    #[test]
    fn deeplink_derive_mnemonic_rejects_bad_callback() {
        let mut deeplinks = Deeplinks::default();
        let stored = deeplinks
            .create_deeplink(&params(DeeplinkKind::Privy))
            .unwrap();
        let live_id = stored.id;

        assert!(matches!(
            deeplinks.derive_mnemonic("not a url"),
            Err(DeeplinkError::InvalidUrl(_))
        ));
        assert!(matches!(
            deeplinks.derive_mnemonic("https://account.example/?payload=aa"),
            Err(DeeplinkError::MissingDeeplinkId(_))
        ));
        assert!(matches!(
            deeplinks.derive_mnemonic("https://account.example/?deeplink_id=nope&payload=aa"),
            Err(DeeplinkError::InvalidDeeplinkId(_))
        ));
        assert!(matches!(
            deeplinks.derive_mnemonic("https://account.example/?deeplink_id=1"),
            Err(DeeplinkError::MissingPayload(_))
        ));
        assert!(matches!(
            deeplinks.derive_mnemonic("https://account.example/?deeplink_id=1&payload=aa"),
            Err(DeeplinkError::DeeplinkNotFound(1))
        ));
        assert!(matches!(
            deeplinks.derive_mnemonic(&format!(
                "https://account.example/?deeplink_id={live_id}&payload=aa"
            )),
            Err(DeeplinkError::InvalidPayload(_))
        ));
    }

    #[test]
    fn deeplink_derive_mnemonic_decrypts_known_entropy() {
        let mut deeplinks = Deeplinks::default();
        let kind = DeeplinkKind::PrivyLink;
        let stored = deeplinks.create_deeplink(&params(kind)).unwrap();
        let id = stored.id;
        let entropy = [0x11u8; 32];
        let expected = bip39::Mnemonic::from_entropy(&entropy).unwrap();
        let payload = encrypt_mnemonic_payload(stored, &entropy);
        let got = deeplinks
            .derive_mnemonic(&format!(
                "https://account.example/?deeplink_id={id}&payload={payload}"
            ))
            .unwrap();
        assert_eq!(got.kind, kind);
        assert_eq!(got.mnemonic, expected);
    }

    #[test]
    fn deeplink_remove_expired_keeps_fresh() {
        let mut deeplinks = Deeplinks::default();
        let id = deeplinks
            .create_deeplink(&params(DeeplinkKind::Privy))
            .unwrap()
            .id;
        deeplinks.remove_expired();
        assert!(matches!(
            deeplinks.derive_mnemonic(&format!(
                "https://account.example/?deeplink_id={id}&payload=aa"
            )),
            Err(DeeplinkError::InvalidPayload(_))
        ));
    }
}
