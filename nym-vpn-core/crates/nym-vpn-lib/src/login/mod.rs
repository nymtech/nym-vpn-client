// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use error::LoginError;
use nym_vpn_lib_types::StoreAccountRequest;
use nym_vpn_store::account::Mnemonic;

pub mod error;
pub mod privy;

pub fn parse_account_request(request: &StoreAccountRequest) -> Result<Mnemonic, LoginError> {
    match request {
        StoreAccountRequest::Vpn { mnemonic }
        | StoreAccountRequest::Decentralised { mnemonic }
        | StoreAccountRequest::Privy { mnemonic } => Mnemonic::parse(mnemonic).map_err(Into::into),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vpn_mnemonic() {
        let mnemonic = Mnemonic::generate(24).unwrap();
        let parsed_mnemonic = parse_account_request(&StoreAccountRequest::Vpn {
            mnemonic: mnemonic.to_string(),
        })
        .unwrap();
        assert_eq!(mnemonic, parsed_mnemonic);

        assert!(
            parse_account_request(&StoreAccountRequest::Vpn {
                mnemonic: String::from("invalid mnemonic")
            })
            .is_err()
        );
    }

    #[test]
    fn parse_decentralised_mnemonic() {
        let mnemonic = Mnemonic::generate(24).unwrap();
        let parsed_mnemonic = parse_account_request(&StoreAccountRequest::Decentralised {
            mnemonic: mnemonic.to_string(),
        })
        .unwrap();
        assert_eq!(mnemonic, parsed_mnemonic);

        assert!(
            parse_account_request(&StoreAccountRequest::Decentralised {
                mnemonic: String::from("invalid mnemonic")
            })
            .is_err()
        );
    }

    #[test]
    fn parse_hex_signature() {
        let hex_signature = String::from(
            "a564a87ccbed5cb5be4929201e555f5b5e26cb01d300d621520d724e57c582c33fa374caf21fd0c5e3118d70d14894845a32acfee47da7f347a0b9a57cba07931c",
        );
        assert!(
            parse_account_request(&StoreAccountRequest::Privy {
                mnemonic: hex_signature
            })
            .is_ok()
        );
    }
}
