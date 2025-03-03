// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use nym_compact_ecash::scheme::keygen::KeyPairUser;
use nym_validator_client::{
    nyxd::bip32::DerivationPath, signing::signer::OfflineSigner as _, DirectSecp256k1HdWallet,
};
use time::{Duration, OffsetDateTime};

use crate::{error::Result, jwt::Jwt, VpnApiClientError};

const MAX_ACCEPTABLE_SKEW_SECONDS: i64 = 30;
const SKEW_SECONDS_CONSIDERED_SAME: i64 = 2;

#[derive(Clone, Debug)]
pub struct VpnApiAccount {
    wallet: DirectSecp256k1HdWallet,
}

impl VpnApiAccount {
    #[allow(unused)]
    fn random() -> Self {
        let mnemonic = bip39::Mnemonic::generate(24).unwrap();
        let wallet = DirectSecp256k1HdWallet::from_mnemonic("n", mnemonic.clone());
        Self { wallet }
    }

    pub fn id(&self) -> String {
        self.wallet.get_accounts().unwrap()[0].address().to_string()
    }

    pub(crate) fn jwt(&self, remote_time: Option<VpnApiTime>) -> Jwt {
        match remote_time {
            Some(remote_time) => Jwt::new_secp256k1_synced(&self.wallet, remote_time),
            None => Jwt::new_secp256k1(&self.wallet),
        }
    }

    pub fn create_ecash_keypair(&self) -> Result<KeyPairUser> {
        let hd_path = cosmos_derivation_path();
        let extended_private_key = self
            .wallet
            .derive_extended_private_key(&hd_path)
            .map_err(VpnApiClientError::CosmosDeriveFromPath)?;
        Ok(KeyPairUser::new_seeded(
            extended_private_key.private_key().to_bytes(),
        ))
    }
}

impl From<bip39::Mnemonic> for VpnApiAccount {
    fn from(mnemonic: bip39::Mnemonic) -> Self {
        let wallet = DirectSecp256k1HdWallet::from_mnemonic("n", mnemonic.clone());
        Self { wallet }
    }
}

fn cosmos_derivation_path() -> DerivationPath {
    nym_config::defaults::COSMOS_DERIVATION_PATH
        .parse()
        .unwrap()
}

#[derive(Clone, Debug)]
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
            local_time: local_time_before_request,
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

#[cfg(test)]
mod tests {
    use crate::types::test_fixtures::{TEST_DEFAULT_MNEMONIC, TEST_DEFAULT_MNEMONIC_ID};

    use super::*;

    #[test]
    fn create_account_from_mnemonic() {
        let account = VpnApiAccount::from(bip39::Mnemonic::parse(TEST_DEFAULT_MNEMONIC).unwrap());
        assert_eq!(account.id(), TEST_DEFAULT_MNEMONIC_ID);
    }
}
