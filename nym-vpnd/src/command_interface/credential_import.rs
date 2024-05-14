// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_proto::{
    import_error::ImportErrorType, ImportError as ProtoImportError, ImportUserCredentialResponse,
};

use crate::service::{ImportCredentialError, VpnServiceImportUserCredentialResult};

impl From<VpnServiceImportUserCredentialResult> for ImportUserCredentialResponse {
    fn from(result: VpnServiceImportUserCredentialResult) -> Self {
        match result {
            VpnServiceImportUserCredentialResult::Success => ImportUserCredentialResponse {
                success: true,
                error: None,
            },
            VpnServiceImportUserCredentialResult::Fail(reason) => ImportUserCredentialResponse {
                success: false,
                error: Some(ProtoImportError::from(reason)),
            },
        }
    }
}

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
            ImportCredentialError::DeserializationFailure { .. } => ProtoImportError {
                kind: ImportErrorType::DeserializationFailure as i32,
                message: err.to_string(),
                details: Default::default(),
            },
            ImportCredentialError::CredentialExpired { .. } => ProtoImportError {
                kind: ImportErrorType::CredentialExpired as i32,
                message: err.to_string(),
                details: Default::default(),
            },
            ImportCredentialError::FreepassExpired { .. } => ProtoImportError {
                kind: ImportErrorType::CredentialExpired as i32,
                message: err.to_string(),
                details: Default::default(),
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
