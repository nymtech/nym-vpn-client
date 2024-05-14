// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use maplit::hashmap;
use nym_vpn_proto::{import_error::ImportErrorType, ImportError as ProtoImportError};

use crate::service::ImportCredentialError;

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
            ImportCredentialError::StorageError(_) => ProtoImportError {
                kind: ImportErrorType::StorageError as i32,
                message: err.to_string(),
                details: Default::default(),
            },
            ImportCredentialError::Generic(_) => ProtoImportError {
                kind: ImportErrorType::Generic as i32,
                message: err.to_string(),
                details: Default::default(),
            },
            ImportCredentialError::DeserializationFailure {
                reason: _,
                ref location,
            } => ProtoImportError {
                kind: ImportErrorType::DeserializationFailure as i32,
                message: err.to_string(),
                details: hashmap! { "location".to_string() => location.to_string_lossy().to_string() },
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
            ImportCredentialError::FreepassExpired { ref expiration } => ProtoImportError {
                kind: ImportErrorType::CredentialExpired as i32,
                message: err.to_string(),
                details: [("expiration".to_string(), expiration.to_string())]
                    .into_iter()
                    .collect(),
            },
            ImportCredentialError::VerificationFailed => ProtoImportError {
                kind: ImportErrorType::VerificationFailed as i32,
                message: err.to_string(),
                details: Default::default(),
            },
            ImportCredentialError::FailedToQueryContract => ProtoImportError {
                kind: ImportErrorType::FailedToQueryContract as i32,
                message: err.to_string(),
                details: Default::default(),
            },
        }
    }
}
