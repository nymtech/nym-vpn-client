// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_bandwidth_controller::{FetcherError, error::FetcherErrorKind};
use nym_credentials_interface::CompactEcashError;
use nym_vpn_api_client::error::{
    DEVICE_NOT_AUTHENTICATED_MESSAGE_ID, FAIR_USAGE_DEPLETED_CODE_ID, VpnApiClientError,
};
use nym_vpn_lib_types::{VpnApiError, VpnApiErrorResponse};

use crate::{credential_request::ZkNymId, storage::error::PendingCredentialRequestsStorageError};

/// Errors returned by the VPN-API credential fetcher.

#[derive(Debug, thiserror::Error)]
pub enum VpnApiFetcherError {
    #[error("pending credential storage error: {0}")]
    Storage(#[from] PendingCredentialRequestsStorageError),

    #[error("epoch id mismatch")]
    EpochIdMismatch,

    #[error("VPN API has failed to issue zk-nym")]
    IssuanceError,

    #[error("polled zk nym has been revoked or is in the process of getting revoked")]
    ZkNymRevoked,

    #[error("ticket type mismatch")]
    TicketTypeMismatch,

    #[error("timeout polling for zknym {id}")]
    PollingTimeout { id: ZkNymId },

    #[error("the fetcher was cancelled")]
    Cancelled,

    #[error("the upgrade mode JWT is malformed")]
    MalformedUpgradeModeJWT,

    #[error("nym-vpn-api response is missing blinded shares")]
    MissingBlindedShares,

    #[error("decoded keys missing index")]
    DecodedKeysMissingIndex,

    #[error("response contains invalid ticketbook type: {0}")]
    InvalidTicketTypeInResponse(String),

    #[error("received response is inconsistent: {0}")]
    InconsistentResponse(String),

    // API client errors
    #[error("transport error calling {endpoint}: {source}")]
    Transport {
        endpoint: String,
        #[source]
        source: Box<VpnApiClientError>,
    },

    #[error("timeout calling {endpoint}")]
    Timeout { endpoint: String },

    #[error("received an {status_code} error while calling {endpoint}: {msg}")]
    ApiStatusCodeError {
        endpoint: String,
        msg: String,
        status_code: u16,
    },

    #[error("received an error while calling {endpoint}: {source}")]
    ApiErrorResponse {
        endpoint: String,
        #[source]
        source: VpnApiErrorResponse,
    },

    #[error("no fair usage left")]
    BandwidthExceeded,

    // Cryptographic errors
    #[error("failed to create ecash keypair: {0}")]
    CreateEcashKeyPair(String),

    #[error("failed to construct withdrawal request: {0}")]
    ConstructWithdrawalRequest(CompactEcashError),

    #[error("failed to aggregate wallets: {0}")]
    AggregateWallets(CompactEcashError),

    #[error("invalid verification key: {0}")]
    InvalidVerificationKey(CompactEcashError),

    #[error("failed to deserialize blinded signature: {0}")]
    DeserializeBlindedSignature(CompactEcashError),

    #[error("failed to verify issued partial wallet: {0}")]
    IssuanceVerification(CompactEcashError),
}

impl VpnApiFetcherError {
    /// Whether this error is worth retrying: a transient network/availability failure rather than a
    /// definitive protocol, cryptographic, or server-side rejection.
    pub(crate) fn is_retryable(&self) -> bool {
        match self {
            Self::Timeout { .. } | Self::Transport { .. } | Self::PollingTimeout { .. } => true,
            Self::ApiErrorResponse { source, .. }
                if source.message_id.as_deref() == Some(DEVICE_NOT_AUTHENTICATED_MESSAGE_ID) =>
            {
                true
            }
            _ => false,
        }
    }

    // Returning a closure that takes the error as input, to simplify callsite
    pub(crate) fn vpn_api_error(endpoint: &str) -> impl FnOnce(VpnApiClientError) -> Self {
        |err| match VpnApiError::try_from(err) {
            Ok(VpnApiError::Response(source)) => {
                if source.code_reference_id.as_deref() == Some(FAIR_USAGE_DEPLETED_CODE_ID) {
                    Self::BandwidthExceeded
                } else {
                    Self::ApiErrorResponse {
                        endpoint: endpoint.into(),
                        source,
                    }
                }
            }
            Ok(VpnApiError::StatusCode { code, msg }) => Self::ApiStatusCodeError {
                endpoint: endpoint.into(),
                msg,
                status_code: code,
            },
            Ok(VpnApiError::Timeout(_)) => Self::Timeout {
                endpoint: endpoint.into(),
            },

            Err(source) => Self::Transport {
                endpoint: endpoint.into(),
                source: Box::new(source),
            },
        }
    }
}

impl FetcherError for VpnApiFetcherError {
    fn kind(&self) -> FetcherErrorKind {
        match self {
            VpnApiFetcherError::Timeout { .. }
            | VpnApiFetcherError::ApiStatusCodeError { .. }
            | VpnApiFetcherError::ApiErrorResponse { .. }
            | VpnApiFetcherError::Transport { .. }
            | VpnApiFetcherError::IssuanceError => FetcherErrorKind::Api,

            VpnApiFetcherError::InconsistentResponse(_) => FetcherErrorKind::Unexpected,

            VpnApiFetcherError::Storage(_) => FetcherErrorKind::Storage,

            VpnApiFetcherError::CreateEcashKeyPair(_)
            | VpnApiFetcherError::ConstructWithdrawalRequest(_)
            | VpnApiFetcherError::InvalidTicketTypeInResponse(_)
            | VpnApiFetcherError::TicketTypeMismatch
            | VpnApiFetcherError::PollingTimeout { .. }
            | VpnApiFetcherError::MissingBlindedShares
            | VpnApiFetcherError::EpochIdMismatch
            | VpnApiFetcherError::InvalidVerificationKey(_)
            | VpnApiFetcherError::DeserializeBlindedSignature(_)
            | VpnApiFetcherError::DecodedKeysMissingIndex
            | VpnApiFetcherError::AggregateWallets(_)
            | VpnApiFetcherError::IssuanceVerification(_)
            | VpnApiFetcherError::MalformedUpgradeModeJWT
            | VpnApiFetcherError::ZkNymRevoked
            | VpnApiFetcherError::Cancelled => FetcherErrorKind::Other,

            VpnApiFetcherError::BandwidthExceeded => FetcherErrorKind::BandwidthDepleted,
        }
    }
}

#[cfg(test)]
mod tests {
    use nym_vpn_lib_types::VpnApiErrorResponse;

    use super::*;

    fn api_error(message_id: Option<&str>) -> VpnApiFetcherError {
        VpnApiFetcherError::ApiErrorResponse {
            endpoint: "zknym".into(),
            source: VpnApiErrorResponse {
                message: "denied".into(),
                message_id: message_id.map(str::to_owned),
                code_reference_id: None,
            },
        }
    }

    #[test]
    fn is_retryable_device_not_authenticated() {
        assert!(api_error(Some(DEVICE_NOT_AUTHENTICATED_MESSAGE_ID)).is_retryable());
    }

    #[test]
    fn is_retryable_other_api_response_is_not() {
        assert!(!api_error(Some("other")).is_retryable());
        assert!(!api_error(None).is_retryable());
        assert!(!VpnApiFetcherError::IssuanceError.is_retryable());
    }
}
