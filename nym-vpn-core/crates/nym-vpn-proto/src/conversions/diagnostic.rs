// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use crate::{conversions::ConversionError, proto};

impl TryFrom<nym_vpn_lib_types::DiagnosticReport> for proto::DiagnosticReport {
    type Error = ConversionError;
    fn try_from(value: nym_vpn_lib_types::DiagnosticReport) -> Result<Self, Self::Error> {
        Ok(Self {
            json: serde_json::to_string(&value).map_err(|e| {
                ConversionError::generic(format!("failed to convert Diagnostic Report : {e}"))
            })?,
        })
    }
}

impl TryFrom<proto::DiagnosticReport> for nym_vpn_lib_types::DiagnosticReport {
    type Error = ConversionError;
    fn try_from(value: proto::DiagnosticReport) -> Result<Self, Self::Error> {
        serde_json::from_str(&value.json).map_err(|e| {
            ConversionError::generic(format!("failed to convert Diagnostic Report : {e}"))
        })
    }
}

impl From<proto::DiagnosticRunParams> for nym_vpn_lib_types::DiagnosticRunParams {
    fn from(value: proto::DiagnosticRunParams) -> Self {
        Self {
            gateway: value.gateway.map(|g| g.id),
            skip_dns: value.skip_dns,
            skip_http: value.skip_http,
            skip_hybrid_transport: value.skip_hybrid_transport,
        }
    }
}

impl From<nym_vpn_lib_types::DiagnosticRunParams> for proto::DiagnosticRunParams {
    fn from(value: nym_vpn_lib_types::DiagnosticRunParams) -> Self {
        Self {
            gateway: value.gateway.map(|id| proto::GatewayId { id }),
            skip_dns: value.skip_dns,
            skip_http: value.skip_http,
            skip_hybrid_transport: value.skip_hybrid_transport,
        }
    }
}

impl TryFrom<nym_vpn_lib_types::RegistrationReport> for proto::RegistrationReport {
    type Error = ConversionError;
    fn try_from(value: nym_vpn_lib_types::RegistrationReport) -> Result<Self, Self::Error> {
        Ok(Self {
            json: serde_json::to_string(&value).map_err(|e| {
                ConversionError::generic(format!("failed to convert Registration Report : {e}"))
            })?,
        })
    }
}

impl TryFrom<proto::RegistrationReport> for nym_vpn_lib_types::RegistrationReport {
    type Error = ConversionError;
    fn try_from(value: proto::RegistrationReport) -> Result<Self, Self::Error> {
        serde_json::from_str(&value.json).map_err(|e| {
            ConversionError::generic(format!("failed to convert Registration Report : {e}"))
        })
    }
}

impl TryFrom<proto::DiagnosticRegisterParams> for nym_vpn_lib_types::DiagnosticRegisterParams {
    type Error = ConversionError;
    fn try_from(value: proto::DiagnosticRegisterParams) -> Result<Self, Self::Error> {
        Ok(Self {
            gateway: value
                .gateway
                .map(|g| g.id)
                .ok_or_else(|| ConversionError::generic("missing gateway id"))?,
            storage_path: value.storage_path.map(PathBuf::from),
            skip_wireguard: value.skip_wireguard,
            registration_mode: nym_vpn_lib_types::RegistrationMode::from_cli_flags(
                value.mixnet,
                value.lp,
            ),
        })
    }
}

impl From<nym_vpn_lib_types::DiagnosticRegisterParams> for proto::DiagnosticRegisterParams {
    fn from(value: nym_vpn_lib_types::DiagnosticRegisterParams) -> Self {
        Self {
            gateway: Some(proto::GatewayId { id: value.gateway }),
            storage_path: value
                .storage_path
                .and_then(|p| p.to_str().map(str::to_string)),
            skip_wireguard: value.skip_wireguard,
            mixnet: value.registration_mode.is_mixnet(),
            lp: value.registration_mode.is_lp(),
        }
    }
}
