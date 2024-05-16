// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use maplit::hashmap;
use nym_vpn_proto::{
    error::ErrorType, import_error::ImportErrorType, Error as ProtoError,
    ImportError as ProtoImportError,
};

use crate::service::{ConnectionFailedError, ImportCredentialError};

impl From<ImportCredentialError> for ProtoImportError {
    fn from(err: ImportCredentialError) -> Self {
        match err {
            ImportCredentialError::VpnRunning => ProtoImportError {
                kind: ImportErrorType::VpnRunning as i32,
                message: err.to_string(),
                details: Default::default(),
            },
            ImportCredentialError::CredentialAlreadyImported => ProtoImportError {
                kind: ImportErrorType::CredentialAlreadyImported as i32,
                message: err.to_string(),
                details: Default::default(),
            },
            ImportCredentialError::StorageError {
                ref path,
                ref error,
            } => ProtoImportError {
                kind: ImportErrorType::StorageError as i32,
                message: err.to_string(),
                details: hashmap! {
                    "path".to_string() => path.to_string_lossy().to_string(),
                    "error".to_string() => error.to_string()
                },
            },
            ImportCredentialError::DeserializationFailure {
                ref reason,
                ref location,
            } => ProtoImportError {
                kind: ImportErrorType::DeserializationFailure as i32,
                message: err.to_string(),
                details: hashmap! {
                    "location".to_string() => location.to_string_lossy().to_string(),
                    "reason".to_string() => reason.clone(),
                },
            },
            ImportCredentialError::CredentialExpired {
                expiration,
                ref location,
            } => ProtoImportError {
                kind: ImportErrorType::CredentialExpired as i32,
                message: err.to_string(),
                details: hashmap! {
                    "location".to_string() => location.to_string_lossy().to_string(),
                    "expiration".to_string() => expiration.to_string(),
                },
            },
        }
    }
}

impl From<ConnectionFailedError> for ProtoError {
    fn from(err: ConnectionFailedError) -> Self {
        match err {
            ConnectionFailedError::InvalidCredential {
                reason,
                location,
                gateway_id,
            } => ProtoError {
                kind: ErrorType::NoValidCredentials as i32,
                message: reason,
                details: [
                    ("location".to_string(), location),
                    ("gateway_id".to_string(), gateway_id),
                ]
                .into_iter()
                .collect(),
            },
            ConnectionFailedError::Generic(reason) => ProtoError {
                kind: ErrorType::Generic as i32,
                message: reason,
                details: Default::default(),
            },
        }
    }
}
