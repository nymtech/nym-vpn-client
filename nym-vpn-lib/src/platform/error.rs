// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum FFIError {
    #[error("Invalid value passed in uniffi")]
    InvalidValueUniffi,
}
