use nym_crypto::asymmetric::ed25519;
use nym_vpn_lib_types::DeeplinkKind;
use rand::{RngCore, rngs::OsRng};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use url::Url;

#[derive(Debug)]
pub struct Deeplink {
    id: u64,
    _kind: DeeplinkKind,
    _name: String,
    keypair: ed25519::KeyPair,
    expiry_time: Instant,
}

impl Deeplink {
    const TTL_SECS: u64 = 300;

    pub fn new(params: &CreateDeeplinkParams) -> Self {
        let mut rng = OsRng;
        let keypair = ed25519::KeyPair::new(&mut rng);
        let id = rng.next_u64();
        let expiry_time = Instant::now() + Duration::from_secs(Self::TTL_SECS);

        Self {
            id,
            _kind: params.kind,
            _name: params.name.clone(),
            keypair,
            expiry_time,
        }
    }

    #[allow(unused)] // TEMP
    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expiry_time
    }

    pub fn create_url(&self, base_url: &Url) -> Url {
        let pubkey = bs58::encode(self.keypair.public_key().to_bytes()).into_string();
        let mut url = base_url.clone();
        url.query_pairs_mut()
            .append_pair("deeplink_id", &self.id.to_string())
            .append_pair("pubkey", &pubkey);
        url
    }
}

#[derive(Debug, Default)]
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

    #[allow(unused)] // TEMP
    pub fn derive_mnemonic(&mut self, url_str: &str) -> Result<String, DeeplinkError> {
        let url =
            url::Url::parse(url_str).map_err(|_| DeeplinkError::InvalidUrl(url_str.to_string()))?;

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

        let Some(encrypted_key_str) = url_params.get("encryptedkey") else {
            return Err(DeeplinkError::MissingEncryptedKey(url_str.to_string()));
        };

        let _encrypted_wallet_key = bs58::decode(encrypted_key_str)
            .into_vec()
            .map_err(|_| DeeplinkError::InvalidEncryptedKey(encrypted_key_str.clone()))?;

        let Some(deeplink) = self.0.remove(&deeplink_id) else {
            return Err(DeeplinkError::DeeplinkNotFound(deeplink_id));
        };

        if deeplink.is_expired() {
            return Err(DeeplinkError::DeeplinkExpired(deeplink_id));
        }

        // TBC
        Err(DeeplinkError::InvalidUrl("Not yet implemented".to_string()))
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

    #[error("missing encrytpedkey parameter in URL: {0}")]
    MissingEncryptedKey(String),

    #[error("invalid base-58 encoded encrytpedkey: {0}")]
    InvalidEncryptedKey(String),

    #[error("deeplink with id {0} not found")]
    DeeplinkNotFound(u64),

    #[error("deeplink with id {0} has expired")]
    DeeplinkExpired(u64),
}

#[derive(Clone, Debug)]
pub struct CreateDeeplinkParams {
    pub kind: DeeplinkKind,
    pub name: String,
    pub base_url: Url,
}
