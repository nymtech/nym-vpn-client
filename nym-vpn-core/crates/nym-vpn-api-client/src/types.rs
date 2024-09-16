// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub use nym_contracts_common::Percent;

use std::sync::Arc;

use nym_crypto::asymmetric::ed25519;
use nym_validator_client::{signing::signer::OfflineSigner as _, DirectSecp256k1HdWallet};

use crate::{jwt::Jwt, VpnApiClientError};

pub struct VpnApiAccount {
    wallet: DirectSecp256k1HdWallet,
}

impl VpnApiAccount {
    #[allow(unused)]
    fn random() -> Self {
        let mnemonic = bip39::Mnemonic::generate(24).unwrap();
        let wallet = DirectSecp256k1HdWallet::from_mnemonic("n", mnemonic);
        Self { wallet }
    }

    pub fn id(&self) -> String {
        self.wallet.get_accounts().unwrap()[0].address().to_string()
    }

    pub(crate) fn jwt(&self) -> Jwt {
        Jwt::new_secp256k1(&self.wallet)
    }
}

impl From<bip39::Mnemonic> for VpnApiAccount {
    fn from(mnemonic: bip39::Mnemonic) -> Self {
        let wallet = DirectSecp256k1HdWallet::from_mnemonic("n", mnemonic);
        Self { wallet }
    }
}

pub struct Device {
    keypair: Arc<ed25519::KeyPair>,
}

impl Device {
    pub(crate) fn identity_key(&self) -> &ed25519::PublicKey {
        self.keypair.public_key()
    }

    pub(crate) fn jwt(&self) -> Jwt {
        Jwt::new_ecdsa(&self.keypair)
    }
}

impl From<Arc<ed25519::KeyPair>> for Device {
    fn from(keypair: Arc<ed25519::KeyPair>) -> Self {
        Self { keypair }
    }
}

impl From<ed25519::KeyPair> for Device {
    fn from(keypair: ed25519::KeyPair) -> Self {
        Self {
            keypair: Arc::new(keypair),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct GatewayMinPerformance {
    pub mixnet_min_performance: Option<Percent>,
    pub vpn_min_performance: Option<Percent>,
}

impl GatewayMinPerformance {
    pub fn from_percentage_values(
        mixnet_min_performance: Option<u64>,
        vpn_min_performance: Option<u64>,
    ) -> Result<Self, VpnApiClientError> {
        let mixnet_min_performance = mixnet_min_performance
            .map(Percent::from_percentage_value)
            .transpose()
            .map_err(VpnApiClientError::InvalidPercentValue)?;
        let vpn_min_performance = vpn_min_performance
            .map(Percent::from_percentage_value)
            .transpose()
            .map_err(VpnApiClientError::InvalidPercentValue)?;
        Ok(Self {
            mixnet_min_performance,
            vpn_min_performance,
        })
    }

    pub(crate) fn to_param(&self) -> Vec<(String, String)> {
        let mut params = vec![];
        if let Some(threshold) = self.mixnet_min_performance {
            params.push((
                crate::routes::MIXNET_MIN_PERFORMANCE.to_string(),
                threshold.to_string(),
            ));
        };
        if let Some(threshold) = self.vpn_min_performance {
            params.push((
                crate::routes::VPN_MIN_PERFORMANCE.to_string(),
                threshold.to_string(),
            ));
        };
        params
    }
}
