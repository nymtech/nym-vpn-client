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
    fn parse_privy_mnemonic() {
        let mnemonic = Mnemonic::generate(24).unwrap();
        let parsed_mnemonic = parse_account_request(&StoreAccountRequest::Privy {
            mnemonic: mnemonic.to_string(),
        })
        .unwrap();
        assert_eq!(mnemonic, parsed_mnemonic);

        assert!(
            parse_account_request(&StoreAccountRequest::Privy {
                mnemonic: String::from("invalid mnemonic")
            })
            .is_err()
        );
    }
}
