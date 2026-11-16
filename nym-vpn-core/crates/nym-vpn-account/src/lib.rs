// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bip39::Mnemonic;
use nym_credentials_interface::KeyPairUser;
use nym_validator_client::{
    DirectSecp256k1HdWallet,
    nyxd::{
        AccountId,
        bip32::{self, DerivationPath},
    },
    signing::{direct_wallet::DirectSecp256k1HdWalletError, signer::OfflineSigner as _},
};
use nym_vpn_api_client::{
    request::AccountKind,
    types::{AccountSigner, AccountSignerError, VpnAccountMode},
};
use zeroize::Zeroizing;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("wallet error")]
    Wallet(#[from] DirectSecp256k1HdWalletError),

    #[error("no accounts in wallet")]
    NoAccounts,

    #[error("failed to generate mnemonic")]
    GenerateMnemonic(#[source] bip39::Error),

    #[error("invalid derivation path")]
    DerivationPath(#[source] bip32::Error),
}

/// Generates a new random 24-word account mnemonic.
pub fn generate_mnemonic() -> Result<Mnemonic, Error> {
    Mnemonic::generate(bip39::WordCount::Words24).map_err(Error::GenerateMnemonic)
}

/// Derives the ecash user keypair of the account defined by the mnemonic.
pub fn derive_ecash_keypair(mnemonic: Mnemonic) -> Result<KeyPairUser, Error> {
    let seed = ecash_keypair_seed(mnemonic)?;
    Ok(KeyPairUser::new_seeded(&seed))
}

fn ecash_keypair_seed(mnemonic: Mnemonic) -> Result<Zeroizing<Vec<u8>>, Error> {
    let wallet = DirectSecp256k1HdWallet::checked_from_mnemonic("n", mnemonic)?;
    let hd_path: DerivationPath = nym_config::defaults::COSMOS_DERIVATION_PATH
        .parse()
        .map_err(Error::DerivationPath)?;
    // `XPrv` zeroizes its key on drop; the copy taken here is zeroized too.
    let extended_private_key = wallet.derive_extended_private_key_with_password(&hd_path, "")?;
    let private_key = Zeroizing::new(extended_private_key.to_bytes());

    Ok(Zeroizing::new(private_key.to_vec()))
}

/// Account backed by an HD wallet derived from the account mnemonic.
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
}

impl VpnAccount {
    pub fn new(mnemonic: Mnemonic, mode: VpnAccountMode) -> Result<Self, Error> {
        let wallet = DirectSecp256k1HdWallet::checked_from_mnemonic("n", mnemonic)?;
        Self::derive_from_wallet(wallet, mode)
    }

    pub fn id(&self) -> String {
        self.id.to_string()
    }

    pub fn pub_key(&self) -> &str {
        &self.pub_key
    }

    pub fn mode(&self) -> VpnAccountMode {
        self.mode
    }

    /// Kind of the account as registered with the VPN API.
    pub fn kind(&self) -> AccountKind {
        if self.mode.is_privy() {
            AccountKind::PrivySecp256k1
        } else {
            AccountKind::UserGeneratedSecp256k1
        }
    }

    pub fn get_mnemonic(&self) -> Zeroizing<String> {
        self.wallet.mnemonic_string()
    }

    fn derive_from_wallet(
        wallet: DirectSecp256k1HdWallet,
        mode: VpnAccountMode,
    ) -> Result<Self, Error> {
        let accounts = wallet.get_accounts();
        let Some(first) = accounts.first() else {
            return Err(Error::NoAccounts);
        };
        let id = first.address().clone();
        let pub_key = bs58::encode(first.public_key().to_bytes()).into_string();

        Ok(Self {
            wallet,
            id,
            pub_key,
            mode,
        })
    }
}

impl AccountSigner for VpnAccount {
    fn id(&self) -> String {
        self.id.to_string()
    }

    fn pub_key(&self) -> &str {
        &self.pub_key
    }

    fn sign_raw(&self, message: &[u8]) -> Result<Vec<u8>, AccountSignerError> {
        let signature = self.wallet.sign_raw(&self.id, message)?;
        Ok(signature.to_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The default account mnemonic, the same as in the js integration tests
    const TEST_MNEMONIC: &str = "range mystery picture decline olympic acoustic lesson quick rebuild panda royal fold start leader egg hammer width olympic worry length crawl couch link mobile";
    const TEST_MNEMONIC_ID: &str = "n1sslaag27wfydyrvyua72hg5e0vteglxrs8nw3c";

    #[test]
    fn create_account_from_mnemonic() {
        let account =
            VpnAccount::new(Mnemonic::parse(TEST_MNEMONIC).unwrap(), VpnAccountMode::Api).unwrap();
        assert_eq!(account.id(), TEST_MNEMONIC_ID);
    }

    #[test]
    fn derive_wallets() {
        for word_count in [bip39::WordCount::Words12, bip39::WordCount::Words24] {
            let wallet = DirectSecp256k1HdWallet::generate("n", word_count).unwrap();
            VpnAccount::derive_from_wallet(wallet, VpnAccountMode::Api).unwrap();
        }
    }

    #[test]
    fn ecash_keypair_seed_is_stable() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        assert_eq!(
            hex::encode(&*ecash_keypair_seed(mnemonic).unwrap()),
            "9bbeaa761f384aff8532cf8f8f78e9cbbebea1f37bc9461d163163cb87fd2f9e"
        );
    }

    #[test]
    fn generate_mnemonic_has_24_words() {
        assert_eq!(generate_mnemonic().unwrap().word_count(), 24);
    }
}
