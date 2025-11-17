// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::VpnApiError;
use std::fmt::Debug;

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Error))]
pub enum RequestZkNymError {
    #[error(transparent)]
    GetZkNymsAvailableForDownloadEndpointFailure { response: VpnApiError },

    #[error("failed to create ecash keypair: {0}")]
    CreateEcashKeyPair(String),

    #[error("failed to construct withdrawal request: {0}")]
    ConstructWithdrawalRequest(String),

    #[error("{response}, {ticket_type}")]
    RequestZkNymEndpointFailure {
        ticket_type: String,
        response: VpnApiError,
    },

    #[error("response contains invalid ticketbook type: {0}")]
    InvalidTicketTypeInResponse(String),

    #[error("ticket type mismatch")]
    TicketTypeMismatch,

    #[error(transparent)]
    PollZkNymEndpointFailure { response: VpnApiError },

    #[error("timeout polling for zknym {id}")]
    PollingTimeout { id: ZkNymId },

    #[error("nym-vpn-api response is missing blinded shares")]
    MissingBlindedShares,

    #[error("response contains invalid master verification key: {0}")]
    ResponseHasInvalidMasterVerificationKey(String),

    #[error("epoch id mismatch")]
    EpochIdMismatch,

    #[error("expiration date mismatch")]
    ExpirationDateMismatch,

    #[error("{response}")]
    GetPartialVerificationKeysEndpointFailure {
        epoch_id: u64,
        response: VpnApiError,
    },

    #[error("no master verification key in storage")]
    NoMasterVerificationKeyInStorage,

    #[error("no coin index signatures in storage")]
    NoCoinIndexSignaturesInStorage,

    #[error("no expiration date signatures in storage")]
    NoExpirationDateSignaturesInStorage,

    #[error("invalid verification key: {0}")]
    InvalidVerificationKey(String),

    #[error("failed to deserialize blinded signature: {0}")]
    DeserializeBlindedSignature(String),

    #[error("decoded keys missing index")]
    DecodedKeysMissingIndex,

    #[error("failed to import zknym")]
    ImportZkNym { ticket_type: String, error: String },

    #[error("failed to aggregate wallets: {0}")]
    AggregateWallets(String),

    #[error("{response}")]
    ConfirmZkNymDownloadEndpointFailure { id: ZkNymId, response: VpnApiError },

    #[error("missing pending request: {0}")]
    MissingPendingRequest(ZkNymId),

    #[error("credential storage error: {0}")]
    CredentialStorage(String),

    #[error("nym-vpn-api: unexpected error response: {0}")]
    UnexpectedErrorResponse(String),

    #[error("polled zk nym has been revoked or is in the process of getting revoked")]
    ZkNymRevoked,

    #[error("VPN API has failed to issue zk-nym")]
    IssuanceError,

    #[error("the upgrade mode JWT is malformed")]
    MalformedUpgradeModeJWT,

    #[error("received response is inconsistent: {reason}")]
    InconsistentResponse { reason: String },

    #[error("internal error: {0}")]
    Internal(String),
}

impl RequestZkNymError {
    pub fn internal(message: impl ToString) -> Self {
        RequestZkNymError::Internal(message.to_string())
    }

    pub fn unexpected_response(message: impl Debug) -> Self {
        RequestZkNymError::UnexpectedErrorResponse(format!("{message:?}"))
    }

    pub fn vpn_api_error(&self) -> Option<VpnApiError> {
        match self {
            RequestZkNymError::GetZkNymsAvailableForDownloadEndpointFailure { response }
            | RequestZkNymError::RequestZkNymEndpointFailure {
                response,
                ticket_type: _,
            }
            | RequestZkNymError::PollZkNymEndpointFailure { response }
            | RequestZkNymError::GetPartialVerificationKeysEndpointFailure {
                response,
                epoch_id: _,
            }
            | RequestZkNymError::ConfirmZkNymDownloadEndpointFailure { response, id: _ } => {
                Some(response.clone())
            }
            _ => None,
        }
    }

    pub fn message(&self) -> Option<String> {
        self.vpn_api_error().map(|err| err.message())
    }

    pub fn message_id(&self) -> Option<String> {
        self.vpn_api_error().and_then(|err| err.message_id())
    }

    pub fn code_reference_id(&self) -> Option<String> {
        self.vpn_api_error().and_then(|err| err.code_reference_id())
    }

    pub fn ticket_type(&self) -> Option<String> {
        match self {
            RequestZkNymError::RequestZkNymEndpointFailure {
                response: _,
                ticket_type,
            } => Some(ticket_type.clone()),
            RequestZkNymError::ImportZkNym {
                ticket_type,
                error: _,
            } => Some(ticket_type.clone()),
            _ => None,
        }
    }

    pub fn inconsistent_response<S: Into<String>>(reason: S) -> Self {
        RequestZkNymError::InconsistentResponse {
            reason: reason.into(),
        }
    }
}

// Simplified version of the error enum suitable for app API
#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum RequestZkNymErrorReason {
    #[error(transparent)]
    VpnApi(VpnApiError),

    #[error("nym-vpn-api: unexpected error response: {0}")]
    UnexpectedVpnApiResponse(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("{0}")]
    Internal(String),
}

impl From<RequestZkNymError> for RequestZkNymErrorReason {
    fn from(err: RequestZkNymError) -> Self {
        match err {
            RequestZkNymError::GetZkNymsAvailableForDownloadEndpointFailure { response }
            | RequestZkNymError::RequestZkNymEndpointFailure {
                response,
                ticket_type: _,
            }
            | RequestZkNymError::PollZkNymEndpointFailure { response }
            | RequestZkNymError::GetPartialVerificationKeysEndpointFailure {
                response,
                epoch_id: _,
            }
            | RequestZkNymError::ConfirmZkNymDownloadEndpointFailure { response, id: _ } => {
                Self::VpnApi(response)
            }
            RequestZkNymError::UnexpectedErrorResponse(reason)
            | RequestZkNymError::InconsistentResponse { reason } => {
                Self::UnexpectedVpnApiResponse(reason)
            }
            RequestZkNymError::IssuanceError => {
                Self::UnexpectedVpnApiResponse("'error' status response received'".to_string())
            }
            RequestZkNymError::CredentialStorage(message) => Self::Storage(message),
            RequestZkNymError::CreateEcashKeyPair(_)
            | RequestZkNymError::ConstructWithdrawalRequest(_)
            | RequestZkNymError::InvalidTicketTypeInResponse(_)
            | RequestZkNymError::TicketTypeMismatch
            | RequestZkNymError::PollingTimeout { .. }
            | RequestZkNymError::MissingBlindedShares
            | RequestZkNymError::ResponseHasInvalidMasterVerificationKey(_)
            | RequestZkNymError::EpochIdMismatch
            | RequestZkNymError::ExpirationDateMismatch
            | RequestZkNymError::NoMasterVerificationKeyInStorage
            | RequestZkNymError::NoCoinIndexSignaturesInStorage
            | RequestZkNymError::NoExpirationDateSignaturesInStorage
            | RequestZkNymError::InvalidVerificationKey(_)
            | RequestZkNymError::DeserializeBlindedSignature(_)
            | RequestZkNymError::DecodedKeysMissingIndex
            | RequestZkNymError::ImportZkNym { .. }
            | RequestZkNymError::AggregateWallets(_)
            | RequestZkNymError::MissingPendingRequest(_)
            | RequestZkNymError::Internal(_)
            | RequestZkNymError::MalformedUpgradeModeJWT
            | RequestZkNymError::ZkNymRevoked => Self::Internal(err.to_string()),
        }
    }
}

pub type ZkNymId = String;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
pub enum RequestZkNymSuccess {
    Ticketbook {
        id: ZkNymId,
        ticketbook_type: String,
    },
    UpgradeMode {
        id: ZkNymId,
    },
}
