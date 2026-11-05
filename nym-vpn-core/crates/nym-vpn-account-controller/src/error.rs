// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error as StdError;

use nym_validator_client::nyxd::error::NyxdError;
use nym_vpn_api_client::{error::VpnApiClientError, response::extract_error_response};
use nym_vpn_lib_types::{AccountControllerError, VpnApiError};

pub type BoxError = Box<dyn StdError + Send + Sync>;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Account controller error that retains the entire error chain.
///
/// Converted into [`AccountControllerError`] when a command result leaves the account controller.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("internal error")]
    Internal(#[source] BoxError),

    #[error("storage error")]
    Storage(#[source] BoxError),

    #[error("account store error")]
    AccountStore(#[source] BoxError),

    #[error("key store error")]
    KeyStore(#[source] BoxError),

    #[error("account summary store error")]
    AccountSummaryStore(#[source] BoxError),

    #[error("failed to setup account storage paths")]
    StoragePaths(#[source] Box<nym_sdk::Error>),

    #[error("wireguard keys storage error")]
    WireguardKeysStorage(#[from] nym_vpn_store::keys::wireguard::KeysDbError),

    #[error("vpn-api error")]
    VpnApi(#[from] VpnApiClientError),

    #[error("unexpected vpn-api response: {0}")]
    UnexpectedVpnApiResponse(String),

    #[error("failed to connect to nyxd instance")]
    NyxdConnectionFailure(#[source] BoxError),

    #[error("failed to resolve query to a nyxd instance")]
    NyxdQueryFailure(#[source] BoxError),

    #[error("account doesn't exist on chain")]
    AccountDoesntExistOnChain,

    #[error("no account stored")]
    NoAccountStored,

    #[error("no device stored")]
    NoDeviceStored,

    #[error("an account is already stored")]
    ExistingAccount,

    #[error("no connectivity")]
    Offline,

    #[error("account is not set in decentralised mode")]
    AccountNotDecentralised,

    #[error("account is set in decentralised mode")]
    AccountDecentralised,

    #[error("account does not have sufficient funds")]
    InsufficientFunds,

    #[error("failed to obtain zk-nym")]
    ZkNymAcquisitionFailure(#[source] BoxError),

    #[error("invalid passphrase")]
    InvalidMnemonic(#[source] BoxError),

    #[error("invalid secret")]
    InvalidSecret(#[source] BoxError),

    #[error("invalid public key")]
    InvalidPublicKey(#[source] BoxError),

    #[error("verify PIN has to be called after starting the pairing process")]
    PairingNotStarted,

    #[error("pairing operation for different session")]
    PairingSessionIdMismatch,

    #[error("pairing process hasn't produced a shared secret yet")]
    NoPairingSharedSecret,

    #[error("shared secret could not be derived for pairing process")]
    PairingDerivingSharedSecret(#[source] BoxError),

    #[error("failed to create ecash keypair")]
    CreateEcashKeyPair(#[source] BoxError),

    #[error("deeplink error")]
    DeeplinkError(#[source] BoxError),
}

impl Error {
    pub fn internal(error: impl Into<BoxError>) -> Self {
        Self::Internal(error.into())
    }

    pub fn storage(error: impl Into<BoxError>) -> Self {
        Self::Storage(error.into())
    }

    pub fn unexpected_response(message: impl std::fmt::Debug) -> Self {
        Self::UnexpectedVpnApiResponse(format!("{message:?}"))
    }
}

impl From<NyxdError> for Error {
    fn from(error: NyxdError) -> Self {
        Self::NyxdQueryFailure(Box::new(error))
    }
}

impl From<Error> for AccountControllerError {
    fn from(error: Error) -> Self {
        match error {
            Error::Internal(e) => Self::Internal(display_chain(&*e)),
            Error::Storage(e) => Self::Storage(display_chain(&*e)),
            Error::AccountStore(_)
            | Error::KeyStore(_)
            | Error::AccountSummaryStore(_)
            | Error::StoragePaths(_)
            | Error::WireguardKeysStorage(_) => Self::Storage(display_chain(&error)),
            Error::VpnApi(e) => match extract_error_response(&e) {
                Some(response) => Self::VpnApi(VpnApiError::Response(response.into())),
                None => Self::Internal(display_chain(&e)),
            },
            Error::UnexpectedVpnApiResponse(e) => Self::UnexpectedVpnApiResponse(e),
            Error::NyxdConnectionFailure(e) => Self::NyxdConnectionFailure(display_chain(&*e)),
            Error::NyxdQueryFailure(e) => Self::NyxdQueryFailure(display_chain(&*e)),
            Error::AccountDoesntExistOnChain => Self::AccountDoesntExistOnChain,
            Error::NoAccountStored => Self::NoAccountStored,
            Error::NoDeviceStored => Self::NoDeviceStored,
            Error::ExistingAccount => Self::ExistingAccount,
            Error::Offline => Self::Offline,
            Error::AccountNotDecentralised => Self::AccountNotDecentralised,
            Error::AccountDecentralised => Self::AccountDecentralised,
            Error::InsufficientFunds => Self::InsufficientFunds,
            Error::ZkNymAcquisitionFailure(e) => Self::ZkNymAcquisitionFailure(display_chain(&*e)),
            Error::InvalidMnemonic(e) => Self::InvalidMnemonic(display_chain(&*e)),
            Error::InvalidSecret(e) => Self::InvalidSecret(display_chain(&*e)),
            Error::InvalidPublicKey(e) => Self::InvalidPublicKey(display_chain(&*e)),
            Error::PairingNotStarted => Self::PairingNotStarted,
            Error::PairingSessionIdMismatch => Self::PairingSessionIdMismatch,
            Error::NoPairingSharedSecret => Self::NoPairingSharedSecret,
            Error::PairingDerivingSharedSecret(e) => {
                Self::PairingDerivingSharedSecret(display_chain(&*e))
            }
            Error::CreateEcashKeyPair(e) => Self::CreateEcashKeyPair(display_chain(&*e)),
            Error::DeeplinkError(e) => Self::DeeplinkError(display_chain(&*e)),
        }
    }
}

/// Renders the error and all of its sources on a single line, separated by `: `.
pub fn display_chain(error: &(dyn StdError + 'static)) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(error) = source {
        message.push_str(": ");
        message.push_str(&error.to_string());
        source = error.source();
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    #[error("outer")]
    struct Outer(#[source] Inner);

    #[derive(Debug, thiserror::Error)]
    #[error("inner")]
    struct Inner;

    #[test]
    fn public_error_contains_entire_chain() {
        let error = Error::KeyStore(Box::new(Outer(Inner)));

        assert_eq!(
            AccountControllerError::from(error),
            AccountControllerError::Storage("key store error: outer: inner".to_owned())
        );
    }

    #[test]
    fn public_error_from_message() {
        let error = Error::internal("something failed");

        assert_eq!(
            AccountControllerError::from(error),
            AccountControllerError::Internal("something failed".to_owned())
        );
    }
}
