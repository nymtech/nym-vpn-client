// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::response::NymVpnZkNymStatus;
use serde::{Deserialize, Serialize};

use crate::VpnApiEndpointFailure;

use super::ZkNymId;

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestZkNymError {
    #[error("failed to get zk-nyms available for download: {source}")]
    GetZkNymsAvailableForDownloadEndpointFailure { source: VpnApiEndpointFailure },

    #[error("failed to create ecash keypair: {0}")]
    CreateEcashKeyPair(String),

    #[error("failed to construct withdrawal request: {0}")]
    ConstructWithdrawalRequest(String),

    #[error("failed to request zk-nym endpoint for {ticket_type}: {source}")]
    RequestZkNymEndpointFailure {
        ticket_type: String,
        source: VpnApiEndpointFailure,
    },

    #[error("response contains invalid ticketbook type: {0}")]
    InvalidTicketTypeInResponse(String),

    #[error("ticket type mismatch")]
    TicketTypeMismatch,

    #[error("error polling for zknym result: {source}")]
    PollZkNymEndpointFailure { source: VpnApiEndpointFailure },

    #[error("polling task failed")]
    PollingTaskError,

    #[error("timeout polling for zknym {id}")]
    PollingTimeout { id: ZkNymId },

    #[error("polling for zknym {id} finished with error for ticket type: {ticket_type}")]
    FinishedWithError {
        id: ZkNymId,
        ticket_type: String,
        status: NymVpnZkNymStatus,
    },

    #[error("response is missing blinded shares")]
    MissingBlindedShares,

    #[error("response contains invalid master verification key: {0}")]
    ResponseHasInvalidMasterVerificationKey(String),

    #[error("epoch id mismatch")]
    EpochIdMismatch,

    #[error("expiration date mismatch")]
    ExpirationDateMismatch,

    #[error("failed to request partial verification keys for epoch {epoch_id}: {source}")]
    GetPartialVerificationKeysEndpointFailure {
        epoch_id: u64,
        source: VpnApiEndpointFailure,
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

    #[error("failed to confirm zknym {id} download: {source}")]
    ConfirmZkNymDownloadEndpointFailure {
        id: ZkNymId,
        source: VpnApiEndpointFailure,
    },

    #[error("missing pending request: {0}")]
    MissingPendingRequest(ZkNymId),

    #[error("failed to remove pending zk-nym request {id}: {error}")]
    RemovePendingRequest { id: String, error: String },

    #[error("credential storage error: {0}")]
    CredentialStorage(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("unexpected error response: {0}")]
    UnexpectedErrorResponse(String),
}

impl RequestZkNymError {
    pub fn internal(message: impl ToString) -> Self {
        RequestZkNymError::Internal(message.to_string())
    }

    pub fn unexpected_response(message: impl ToString) -> Self {
        RequestZkNymError::UnexpectedErrorResponse(message.to_string())
    }

    pub fn message(&self) -> String {
        match self {
            RequestZkNymError::RequestZkNymEndpointFailure {
                source,
                ticket_type: _,
            }
            | RequestZkNymError::PollZkNymEndpointFailure { source } => source.message.clone(),
            other => other.to_string(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        match self {
            RequestZkNymError::RequestZkNymEndpointFailure {
                source,
                ticket_type: _,
            }
            | RequestZkNymError::PollZkNymEndpointFailure { source } => source.message_id.clone(),
            _ => None,
        }
    }

    pub fn ticket_type(&self) -> Option<String> {
        match self {
            RequestZkNymError::RequestZkNymEndpointFailure {
                source: _,
                ticket_type,
            } => Some(ticket_type.clone()),
            RequestZkNymError::FinishedWithError {
                id: _,
                ticket_type,
                status: _,
            }
            | RequestZkNymError::ImportZkNym {
                ticket_type,
                error: _,
            } => Some(ticket_type.clone()),
            _ => None,
        }
    }
}
