// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum UniffiConversionError {
    #[error("invalid byte length")]
    InvalidByteLength,

    #[error("invalid mixnet min performance percentage")]
    InvalidMixnetMinPerformancePercentage,

    #[error("invalid vpn min performance percentage")]
    InvalidVpnMinPerformancePercentage,
}
