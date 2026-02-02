// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::jwt::Jwt;
use k256::ecdsa::{SigningKey, signature::Signer as _};
use nym_compact_ecash::scheme::keygen::KeyPairUser;
use nym_validator_client::{
    DirectSecp256k1HdWallet,
    nyxd::{AccountId, bip32::DerivationPath},
    signing::signer::OfflineSigner as _,
};
use nym_vpn_store::types::{StorableAccount, StoredAccountMode};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use std::fmt;
use time::{Duration, OffsetDateTime};
use zeroize::Zeroizing;

const MAX_ACCEPTABLE_SKEW_SECONDS: i64 = 60;
const SKEW_SECONDS_CONSIDERED_SAME: i64 = 2;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("wallet error")]
    Wallet(#[from] nym_validator_client::signing::direct_wallet::DirectSecp256k1HdWalletError),

    #[error("no accounts in wallet")]
    NoAccounts,

    #[error("invalid raw key entropy")]
    InvalidEntropy,

    #[error("raw key error: {0}")]
    RawKey(String),

    #[error("address derivation error")]
    AddressDerivation,

    #[error("signing error")]
    SigningFailed,

    #[error("operation not supported for raw key wallet")]
    NotSupportedForRawWallet,

    #[error("subscription expiry parse error: {0}")]
    SubscriptionExpiryParseError(time::error::Parse),

    #[error("traffic reset time parse error: {0}")]
    TrafficResetTimeParseError(time::error::Parse),
}

/// Defines the mode of operation of the associated account.
#[derive(Debug, Copy, Clone, strum_macros::Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum VpnAccountMode {
    /// Account works in the API mode, i.e. the subscription is managed
    /// by the VPN API which provides required ticketbooks
    Api,

    /// Account works in the decentralised mode, i.e. there is no associated subscription
    /// and the account uses its own funds for obtaining required ticketbooks
    // add an alias for our US friends
    Decentralised,

    /// Account works in the API mode, but the mnemonic is derived from the Privy
    /// wallet private key.
    Privy,
}

impl VpnAccountMode {
    pub fn is_api(&self) -> bool {
        matches!(self, Self::Api)
    }

    pub fn is_decentralised(&self) -> bool {
        matches!(self, Self::Decentralised)
    }

    pub fn is_privy(&self) -> bool {
        matches!(self, Self::Privy)
    }
}

impl From<StoredAccountMode> for VpnAccountMode {
    fn from(mode: StoredAccountMode) -> Self {
        match mode {
            StoredAccountMode::Api => VpnAccountMode::Api,
            StoredAccountMode::Decentralised => VpnAccountMode::Decentralised,
            StoredAccountMode::Privy => VpnAccountMode::Privy,
        }
    }
}

impl From<VpnAccountMode> for StoredAccountMode {
    fn from(mode: VpnAccountMode) -> Self {
        match mode {
            VpnAccountMode::Api => StoredAccountMode::Api,
            VpnAccountMode::Decentralised => StoredAccountMode::Decentralised,
            VpnAccountMode::Privy => StoredAccountMode::Privy,
        }
    }
}

/// Internal wallet representation supporting both HD and raw key derivation.
enum AccountWallet {
    /// Standard BIP-44 HD wallet for API and Decentralised accounts.
    Hd(DirectSecp256k1HdWallet),
    /// Raw secp256k1 key for Privy accounts — uses 32-byte entropy directly
    /// as the private key, matching the web app's RawSecp256k1Wallet.
    Raw {
        signing_key: SigningKey,
    },
}

pub struct VpnAccount {
    /// The underlying wallet behind the account.
    wallet: AccountWallet,

    /// Cosmos account identifier of the first derived account.
    id: AccountId,

    /// Base58-encoded public (secp256k1) key of the first derived account.
    pub_key: String,

    /// Raw compressed public key bytes (33 bytes).
    pub_key_bytes: Vec<u8>,

    /// Mode of operation of this account
    mode: VpnAccountMode,

    /// Base64-encoded signature on the account identifier of this account.
    signature_base64: String,
}

impl VpnAccount {
    pub fn new(mnemonic: bip39::Mnemonic, mode: VpnAccountMode) -> Result<Self, Error> {
        if mode.is_privy() {
            // For Privy accounts, use the raw entropy bytes directly as the secp256k1
            // private key. This matches the web app's RawSecp256k1Wallet derivation,
            // ensuring both platforms derive the same Cosmos address from the same
            // Privy wallet key.
            let entropy = mnemonic.to_entropy();
            let raw_bytes: [u8; 32] = entropy
                .try_into()
                .map_err(|_| Error::InvalidEntropy)?;
            return Self::from_raw_key(&raw_bytes);
        }
        let wallet = DirectSecp256k1HdWallet::checked_from_mnemonic("n", mnemonic)?;
        Self::derive_from_hd_wallet(wallet, mode)
    }

    /// Create a Privy account using raw 32-byte entropy directly as secp256k1 private key.
    /// This matches the web app's RawSecp256k1Wallet derivation.
    pub fn from_raw_key(raw_bytes: &[u8; 32]) -> Result<Self, Error> {
        let signing_key = SigningKey::from_bytes(raw_bytes.into())
            .map_err(|e| Error::RawKey(e.to_string()))?;
        let verifying_key = signing_key.verifying_key();
        let pub_key_point = verifying_key.to_encoded_point(true); // compressed 33 bytes
        let pub_key_bytes = pub_key_point.as_bytes().to_vec();
        let pub_key = bs58::encode(&pub_key_bytes).into_string();

        // Derive cosmos address: RIPEMD160(SHA256(compressed_pubkey))
        let address = derive_cosmos_address("n", &pub_key_bytes)?;
        let id_str = address.to_string();

        // Sign the address string with the raw key (same as HD wallet does)
        let sig: k256::ecdsa::Signature = signing_key.sign(id_str.as_bytes());
        let signature_base64 = base64_url::encode(&sig.to_bytes());

        Ok(Self {
            wallet: AccountWallet::Raw { signing_key },
            id: address,
            pub_key,
            pub_key_bytes,
            mode: VpnAccountMode::Privy,
            signature_base64,
        })
    }

    pub fn generate_new() -> Result<(Self, bip39::Mnemonic), Error> {
        let mnemonic = bip39::Mnemonic::generate(24).unwrap();
        let wallet = DirectSecp256k1HdWallet::checked_from_mnemonic("n", mnemonic.clone())?;
        let account = Self::derive_from_hd_wallet(wallet, VpnAccountMode::Api)?;
        Ok((account, mnemonic))
    }

    fn derive_from_hd_wallet(
        wallet: DirectSecp256k1HdWallet,
        mode: VpnAccountMode,
    ) -> Result<Self, Error> {
        let accounts = wallet.get_accounts();
        let Some(first) = accounts.first() else {
            return Err(Error::NoAccounts);
        };
        let address = first.address().clone();
        let id = address.to_string();
        let raw_pub_key = first.public_key();
        let pub_key_bytes = raw_pub_key.to_bytes().to_vec();
        let pub_key = bs58::encode(&pub_key_bytes).into_string();

        let signature = wallet.sign_raw(&address, &id)?;
        let signature_bytes = signature.to_bytes();
        let signature_base64 = base64_url::encode(&signature_bytes);

        Ok(Self {
            wallet: AccountWallet::Hd(wallet),
            id: address,
            pub_key,
            pub_key_bytes,
            mode,
            signature_base64,
        })
    }

    pub fn id(&self) -> String {
        self.id.to_string()
    }

    pub fn id_typed(&self) -> &AccountId {
        &self.id
    }

    pub fn pub_key(&self) -> &str {
        &self.pub_key
    }

    pub fn pub_key_bytes(&self) -> &[u8] {
        &self.pub_key_bytes
    }

    pub fn signature_base64(&self) -> &str {
        &self.signature_base64
    }

    pub(crate) fn jwt(&self, remote_time: Option<VpnApiTime>) -> Jwt {
        let now = match remote_time {
            Some(rt) => rt.estimate_remote_now_unix(),
            None => std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as u128,
        };
        match &self.wallet {
            AccountWallet::Hd(wallet) => Jwt::new_secp256k1_with_now(wallet, now),
            AccountWallet::Raw { signing_key } => {
                Jwt::new_secp256k1_from_raw(
                    &self.id,
                    &self.pub_key_bytes,
                    signing_key,
                    now,
                )
            }
        }
    }

    pub fn create_ecash_keypair(&self) -> Result<KeyPairUser, Error> {
        let seed = self.ecash_keypair_seed()?;
        Ok(KeyPairUser::new_seeded(&seed))
    }

    pub fn ecash_keypair_seed(&self) -> Result<Zeroizing<Vec<u8>>, Error> {
        match &self.wallet {
            AccountWallet::Hd(wallet) => {
                let hd_path = cosmos_derivation_path();
                // TODO: private key is NOT zeroized here
                let extended_private_key = wallet
                    .derive_extended_private_key_with_password(&hd_path, "")?;

                Ok(Zeroizing::new(
                    extended_private_key.private_key().to_bytes().to_vec(),
                ))
            }
            AccountWallet::Raw { .. } => Err(Error::NotSupportedForRawWallet),
        }
    }

    pub fn get_mnemonic(&self) -> Zeroizing<String> {
        match &self.wallet {
            AccountWallet::Hd(wallet) => wallet.mnemonic_string(),
            AccountWallet::Raw { .. } => Zeroizing::new(String::new()),
        }
    }

    pub fn mode(&self) -> VpnAccountMode {
        self.mode
    }
}

impl TryFrom<StorableAccount> for VpnAccount {
    type Error = Error;

    fn try_from(account: StorableAccount) -> Result<Self, Self::Error> {
        Self::new(account.mnemonic, account.mode.into())
    }
}

/// Derive a Cosmos bech32 address from a compressed secp256k1 public key.
/// Uses the standard Cosmos address derivation: bech32(prefix, RIPEMD160(SHA256(pubkey)))
fn derive_cosmos_address(prefix: &str, compressed_pubkey: &[u8]) -> Result<AccountId, Error> {
    let sha_hash = Sha256::digest(compressed_pubkey);
    let ripemd_hash = Ripemd160::digest(&sha_hash);
    AccountId::new(prefix, ripemd_hash.as_slice()).map_err(|_| Error::AddressDerivation)
}

fn cosmos_derivation_path() -> DerivationPath {
    nym_config::defaults::COSMOS_DERIVATION_PATH
        .parse()
        .unwrap()
}

#[derive(Clone, Copy, Debug)]
pub struct VpnApiTime {
    // The local time on the client.
    pub local_time: OffsetDateTime,

    // The estimated time on the remote server. Based on RTT, it's not guaranteed to be accurate.
    pub estimated_remote_time: OffsetDateTime,
}

impl VpnApiTime {
    pub fn from_estimated_remote_time(
        local_time: OffsetDateTime,
        estimated_remote_time: OffsetDateTime,
    ) -> Self {
        Self {
            local_time,
            estimated_remote_time,
        }
    }

    pub fn from_remote_timestamp(
        local_time_before_request: OffsetDateTime,
        remote_timestamp: OffsetDateTime,
        local_time_after_request: OffsetDateTime,
    ) -> Self {
        let rtt = local_time_after_request - local_time_before_request;
        let estimated_remote_time = remote_timestamp + (rtt / 2);
        Self {
            local_time: local_time_after_request,
            estimated_remote_time,
        }
    }

    // Local time minus remote time. Meaning if the value is positive, the local time is ahead
    // of the remote time.
    pub fn local_time_ahead_skew(&self) -> Duration {
        self.local_time - self.estimated_remote_time
    }

    pub fn is_almost_same(&self) -> bool {
        self.local_time_ahead_skew().abs().whole_seconds() < SKEW_SECONDS_CONSIDERED_SAME
    }

    pub fn is_acceptable_synced(&self) -> bool {
        self.local_time_ahead_skew().abs().whole_seconds() < MAX_ACCEPTABLE_SKEW_SECONDS
    }

    pub fn is_synced(&self) -> VpnApiTimeSynced {
        if self.is_almost_same() {
            VpnApiTimeSynced::AlmostSame
        } else if self.is_acceptable_synced() {
            VpnApiTimeSynced::AcceptableSynced
        } else {
            VpnApiTimeSynced::NotSynced
        }
    }

    pub fn estimate_remote_now(&self) -> OffsetDateTime {
        tracing::debug!(
            "Estimating remote now using (local time ahead) skew: {}",
            self.local_time_ahead_skew()
        );
        let local_time_now = OffsetDateTime::now_utc();
        local_time_now - self.local_time_ahead_skew()
    }

    pub fn estimate_remote_now_unix(&self) -> u128 {
        self.estimate_remote_now().unix_timestamp() as u128
    }
}

impl fmt::Display for VpnApiTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Local time: {}, Remote time: {}, Skew: {}",
            self.local_time,
            self.estimated_remote_time,
            self.local_time_ahead_skew(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VpnApiTimeSynced {
    AlmostSame,
    AcceptableSynced,
    NotSynced,
}

impl VpnApiTimeSynced {
    pub fn is_synced(&self) -> bool {
        matches!(
            self,
            VpnApiTimeSynced::AlmostSame | VpnApiTimeSynced::AcceptableSynced
        )
    }

    pub fn is_not_synced(&self) -> bool {
        !self.is_synced()
    }
}

#[cfg(test)]
mod tests {
    use crate::types::test_fixtures::{TEST_DEFAULT_MNEMONIC, TEST_DEFAULT_MNEMONIC_ID};

    use super::*;

    #[test]
    fn create_account_from_mnemonic() {
        let account = VpnAccount::new(
            bip39::Mnemonic::parse(TEST_DEFAULT_MNEMONIC).unwrap(),
            VpnAccountMode::Api,
        )
        .unwrap();
        assert_eq!(account.id(), TEST_DEFAULT_MNEMONIC_ID);
    }

    #[test]
    fn create_random_account() {
        let (_, mnemonic) = VpnAccount::generate_new().unwrap();
        assert_eq!(mnemonic.word_count(), 24);
    }

    #[test]
    fn derive_wallets() {
        for word_count in [12, 24] {
            let wallet = DirectSecp256k1HdWallet::generate("n", word_count).unwrap();
            VpnAccount::derive_from_hd_wallet(wallet, VpnAccountMode::Api).unwrap();
        }
    }

    #[test]
    fn raw_key_derivation_produces_valid_account() {
        // Test that raw key derivation works and produces a valid Cosmos address
        let raw_bytes: [u8; 32] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ];
        let account = VpnAccount::from_raw_key(&raw_bytes).unwrap();
        assert!(account.id().starts_with("n1"));
        assert!(!account.pub_key().is_empty());
        assert!(!account.signature_base64().is_empty());
        assert_eq!(account.mode(), VpnAccountMode::Privy);
    }

    #[test]
    fn privy_mode_uses_raw_key_derivation() {
        // When constructing with Privy mode, raw key derivation should be used.
        // The address should differ from HD derivation of the same mnemonic.
        let raw_bytes: [u8; 32] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ];
        let mnemonic = bip39::Mnemonic::from_entropy(&raw_bytes).unwrap();

        let privy_account = VpnAccount::new(mnemonic.clone(), VpnAccountMode::Privy).unwrap();
        let api_account = VpnAccount::new(mnemonic, VpnAccountMode::Api).unwrap();

        // Raw key derivation (Privy) and HD derivation (Api) should produce different addresses
        assert_ne!(privy_account.id(), api_account.id());
        assert_eq!(privy_account.mode(), VpnAccountMode::Privy);
        assert_eq!(api_account.mode(), VpnAccountMode::Api);
    }

    #[test]
    fn raw_key_derivation_is_deterministic() {
        let raw_bytes: [u8; 32] = [0x42; 32];
        let account1 = VpnAccount::from_raw_key(&raw_bytes).unwrap();
        let account2 = VpnAccount::from_raw_key(&raw_bytes).unwrap();
        assert_eq!(account1.id(), account2.id());
        assert_eq!(account1.pub_key(), account2.pub_key());
    }
}
