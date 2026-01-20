use nym_crypto::asymmetric::ed25519;
use rand::{rngs::OsRng, RngCore};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

#[derive(Debug)]
pub struct Deeplink {
    id: u64,
    kind: DeeplinkKind,
    name: String,
    keypair: ed25519::KeyPair,
    expiry_time: Instant,
}

impl Deeplink {
    const TTL_SECS: u64 = 300;

    pub fn new(name: &str, kind: DeeplinkKind) -> Self {
        let mut rng = OsRng;
        let keypair = ed25519::KeyPair::new(&mut rng);
        let id = rng.next_u64();
        let expiry_time = Instant::now() + Duration::from_secs(Self::TTL_SECS);

        Self {
            id,
            kind,
            name: name.to_string(),
            keypair,
            expiry_time,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expiry_time
    }

    pub fn create_url(&self, base_uri: &str) -> String {
        let pubkey = bs58::encode(self.keypair.public_key().to_bytes()).into_string();
        format!(
            "{base_uri}?deeplink_id={deeplink_id}&pubkey={pubkey}",
            deeplink_id = self.id
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DeeplinkKind {
    Privy,
}

pub struct Deeplinks(HashMap<u64, Deeplink>);

impl Deeplinks {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn create_deeplink(&mut self, name: &str, kind: DeeplinkKind) -> &Deeplink {
        let deeplink = Deeplink::new(name, kind);
        let id = deeplink.id;
        self.0.insert(id, deeplink);
        self.0.get(&id).unwrap()
    }

    /// If the deeplink is found then it's also removed.
    pub fn retrieve_deeplink(&mut self, id: u64) -> Option<Deeplink> {
        self.0.remove(&id)
    }

    pub fn remove_expired(&mut self) {
        let now = Instant::now();
        self.0.retain(|_, deeplink| deeplink.expiry_time > now);
    }

    pub fn derive_mnemonic(&self, url_str: &str) -> Result<String, DeeplinkError> {
        let url =
            url::Url::parse(url_str).map_err(|e| DeeplinkError::InvalidUrl(url_str.to_string()))?;
        let url_params: HashMap<String, String> = url
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let Some(deeplink_id_str) = url_params.get("deeplink_id") else {
            return Err(DeeplinkError::MissingDeeplinkId(url_str.to_string()));
        };
        
        let Some(encrypted_key_str) = url_params.get("encryptedkey") else {
            return Err(DeeplinkError::MissingEncryptedKey(url_str.to_string()))
        };

        Ok("x".to_string())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeeplinkError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("missing deeplink_id parameter in URL: {0}")]
    MissingDeeplinkId(String),

    #[error("missing encryypedkey parameter in URL: {0}")]
    MissingEncryptedKey(String),

}
