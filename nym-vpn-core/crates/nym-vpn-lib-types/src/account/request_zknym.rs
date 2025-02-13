// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::VpnApiErrorResponse;

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum RequestZkNymError {
    #[error("no account stored")]
    NoAccountStored,

    #[error("no device stored")]
    NoDeviceStored,

    #[error("failed to get zk-nyms available for download: {response}")]
    GetZkNymsAvailableForDownloadEndpointFailure { response: VpnApiErrorResponse },

    #[error("failed to create ecash keypair: {0}")]
    CreateEcashKeyPair(String),

    #[error("failed to construct withdrawal request: {0}")]
    ConstructWithdrawalRequest(String),

    #[error("failed to request zk-nym endpoint for {ticket_type}: {response}")]
    RequestZkNymEndpointFailure {
        ticket_type: String,
        response: VpnApiErrorResponse,
    },

    #[error("response contains invalid ticketbook type: {0}")]
    InvalidTicketTypeInResponse(String),

    #[error("ticket type mismatch")]
    TicketTypeMismatch,

    #[error("error polling for zknym result: {response}")]
    PollZkNymEndpointFailure { response: VpnApiErrorResponse },

    #[error("polling task failed")]
    PollingTaskError,

    #[error("timeout polling for zknym {id}")]
    PollingTimeout { id: ZkNymId },

    #[error("response is missing blinded shares")]
    MissingBlindedShares,

    #[error("response contains invalid master verification key: {0}")]
    ResponseHasInvalidMasterVerificationKey(String),

    #[error("epoch id mismatch")]
    EpochIdMismatch,

    #[error("expiration date mismatch")]
    ExpirationDateMismatch,

    #[error("failed to request partial verification keys for epoch {epoch_id}: {response}")]
    GetPartialVerificationKeysEndpointFailure {
        epoch_id: u64,
        response: VpnApiErrorResponse,
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

    #[error("failed to confirm zknym {id} download: {response}")]
    ConfirmZkNymDownloadEndpointFailure {
        id: ZkNymId,
        response: VpnApiErrorResponse,
    },

    #[error("missing pending request: {0}")]
    MissingPendingRequest(ZkNymId),

    #[error("failed to remove pending zk-nym request {id}: {error}")]
    RemovePendingRequest { id: String, error: String },

    #[error("credential storage error: {0}")]
    CredentialStorage(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("nym-vpn-api: unexpected error response: {0}")]
    UnexpectedErrorResponse(String),
}

impl RequestZkNymError {
    pub fn internal(message: impl ToString) -> Self {
        RequestZkNymError::Internal(message.to_string())
    }

    pub fn unexpected_response(message: impl ToString) -> Self {
        RequestZkNymError::UnexpectedErrorResponse(message.to_string())
    }

    pub fn vpn_api_error(&self) -> Option<VpnApiErrorResponse> {
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
        self.vpn_api_error().map(|err| err.message.clone())
    }

    pub fn message_id(&self) -> Option<String> {
        self.vpn_api_error().and_then(|err| err.message_id.clone())
    }

    pub fn code_reference_id(&self) -> Option<String> {
        self.vpn_api_error()
            .and_then(|err| err.code_reference_id.clone())
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
}

// Simplified version of the error enum suitable for app API
#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum RequestZkNymErrorReason {
    #[error("no account stored")]
    NoAccountStored,

    #[error("no device stored")]
    NoDeviceStored,

    #[error(transparent)]
    VpnApi(VpnApiErrorResponse),

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
            RequestZkNymError::NoAccountStored => Self::NoAccountStored,
            RequestZkNymError::NoDeviceStored => Self::NoDeviceStored,
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
            RequestZkNymError::UnexpectedErrorResponse(message) => {
                Self::UnexpectedVpnApiResponse(message)
            }
            RequestZkNymError::CredentialStorage(message) => Self::Storage(message),
            RequestZkNymError::CreateEcashKeyPair(_)
            | RequestZkNymError::ConstructWithdrawalRequest(_)
            | RequestZkNymError::InvalidTicketTypeInResponse(_)
            | RequestZkNymError::TicketTypeMismatch
            | RequestZkNymError::PollingTaskError
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
            | RequestZkNymError::RemovePendingRequest { .. }
            | RequestZkNymError::Internal(_) => Self::Internal(err.to_string()),
        }
    }
}

pub type ZkNymId = String;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestZkNymSuccess {
    pub id: ZkNymId,
}

impl RequestZkNymSuccess {
    pub fn new(id: ZkNymId) -> Self {
        RequestZkNymSuccess { id }
    }
}
