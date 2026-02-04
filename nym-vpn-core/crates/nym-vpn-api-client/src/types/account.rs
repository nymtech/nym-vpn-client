// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::jwt::Jwt;
use nym_compact_ecash::scheme::keygen::KeyPairUser;
use nym_validator_client::{
    DirectSecp256k1HdWallet,
    nyxd::{AccountId, bip32::DerivationPath},
    signing::signer::OfflineSigner as _,
};
use nym_vpn_store::types::{StorableAccount, StoredAccountMode};
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

pub struct VpnAccount {
    /// The underlying wallet behind the account that defines
    /// the associated private key(s).
    wallet: DirectSecp256k1HdWallet,

    /// Cosmos account identifier of the first derived account.
    id: AccountId,

    /// Base58-encoded public (secp256k1) key of the first derived account.
    pub_key: String,

    /// Mode of operation of this account
    mode: VpnAccountMode,

    /// Base64-encoded signature on the account identifier of this account.
    // Note that it is seemingly (?) only ever used once during account creation,
    // so lack of replay protection whilst sketchy is fine.
    signature_base64: String,
}

impl VpnAccount {
    pub fn new(mnemonic: bip39::Mnemonic, mode: VpnAccountMode) -> Result<Self, Error> {
        let wallet = DirectSecp256k1HdWallet::checked_from_mnemonic("n", mnemonic)?;
        Self::derive_from_wallet(wallet, mode)
    }

    pub fn generate_new() -> Result<(Self, bip39::Mnemonic), Error> {
        let mnemonic = bip39::Mnemonic::generate(24).unwrap();
        let wallet = DirectSecp256k1HdWallet::checked_from_mnemonic("n", mnemonic.clone())?;
        let account = Self::derive_from_wallet(wallet, VpnAccountMode::Api)?;
        Ok((account, mnemonic))
    }

    fn derive_from_wallet(
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
        let pub_key = bs58::encode(raw_pub_key.to_bytes()).into_string();

        let signature = wallet.sign_raw(&address, &id)?;
        let signature_bytes = signature.to_bytes();
        let signature_base64 = base64_url::encode(&signature_bytes);

        Ok(Self {
            wallet,
            id: address,
            pub_key,
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

    pub fn signature_base64(&self) -> &str {
        &self.signature_base64
    }

    pub(crate) fn jwt(&self, remote_time: Option<VpnApiTime>) -> Jwt {
        match remote_time {
            Some(remote_time) => Jwt::new_secp256k1_synced(&self.wallet, remote_time),
            None => Jwt::new_secp256k1(&self.wallet),
        }
    }

    pub fn create_ecash_keypair(&self) -> Result<KeyPairUser, Error> {
        let seed = self.ecash_keypair_seed()?;
        Ok(KeyPairUser::new_seeded(&seed))
    }

    pub fn ecash_keypair_seed(&self) -> Result<Zeroizing<Vec<u8>>, Error> {
        let hd_path = cosmos_derivation_path();
        // TODO: private key is NOT zeroized here
        let extended_private_key = self
            .wallet
            .derive_extended_private_key_with_password(&hd_path, "")?; // No password is used

        Ok(Zeroizing::new(
            extended_private_key.private_key().to_bytes().to_vec(),
        ))
    }

    pub fn get_mnemonic(&self) -> Zeroizing<String> {
        self.wallet.mnemonic_string()
    }

    pub fn mode(&self) -> VpnAccountMode {
        self.mode
    }

    /// Signs the message with the account's private key and returns the signature as a hex-encoded string.
    pub fn sign(&self, message: &str) -> Result<String, Error> {
        let signature = self.wallet.sign_raw(&self.id, message.as_bytes())?;
        Ok(hex::encode(signature.to_bytes()))
    }
}

impl TryFrom<StorableAccount> for VpnAccount {
    type Error = Error;

    fn try_from(account: StorableAccount) -> Result<Self, Self::Error> {
        Self::new(account.mnemonic, account.mode.into())
    }
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
            VpnAccount::derive_from_wallet(wallet, VpnAccountMode::Api).unwrap();
        }
    }
}
