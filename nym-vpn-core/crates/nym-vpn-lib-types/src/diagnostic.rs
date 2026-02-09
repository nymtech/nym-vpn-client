// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
};
use time::OffsetDateTime;

use crate::gateway;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct DiagnosticResult<T> {
    pub ok: bool,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub value: Option<T>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub error: Option<String>,
}

impl<T> DiagnosticResult<T> {
    pub const SUCCESS: DiagnosticResult<()> = DiagnosticResult {
        ok: true,
        value: None,
        error: None,
    };

    pub fn from_err(error: impl ToString) -> DiagnosticResult<T> {
        Self {
            ok: false,
            value: None,
            error: Some(error.to_string()),
        }
    }

    pub fn from_value(value: T) -> DiagnosticResult<T> {
        Self {
            ok: true,
            value: Some(value),
            error: None,
        }
    }
}

impl<T, E> From<Result<T, E>> for DiagnosticResult<T>
where
    E: ToString,
{
    fn from(value: Result<T, E>) -> Self {
        let result = value.map_err(|e| e.to_string());
        DiagnosticResult {
            ok: result.is_ok(),
            error: result.as_ref().err().cloned(),
            value: result.ok(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    pub dns: Option<CompleteDnsReport>,
    pub http: Option<DiagnosticResult<HttpReport>>,
    pub gateway: Option<GatewayReport>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct CompleteDnsReport {
    pub system: DiagnosticResult<Vec<DnsResolution>>,
    pub by_nameserver: Vec<DnsResolution>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct DnsResolution {
    pub nameservers: String,
    pub hostname: String,
    pub resolution: DiagnosticResult<Vec<IpAddr>>,
    pub resolution_duration_ms: u128,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct HttpReport {
    pub remote_time: DiagnosticResult<ApiTimeSkew>,
    pub health_response: DiagnosticResult<DiagnosticHealthResponse>,
    pub nb_nymnodes: DiagnosticResult<usize>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct DiagnosticHealthResponse {
    pub status: String,
    #[cfg_attr(feature = "serde", serde(with = "time::serde::rfc3339"))]
    pub timestamp_utc: OffsetDateTime,
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::response::NymVpnHealthResponse> for DiagnosticHealthResponse {
    fn from(value: nym_vpn_api_client::response::NymVpnHealthResponse) -> Self {
        Self {
            status: value.status,
            timestamp_utc: value.timestamp_utc,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct ApiTimeSkew {
    // The local time on the client.
    pub local_time: OffsetDateTime,

    // The estimated time on the remote server. Based on RTT, it's not guaranteed to be accurate.
    pub estimated_remote_time: OffsetDateTime,

    pub accetably_synced: bool,
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::types::VpnApiTime> for ApiTimeSkew {
    fn from(value: nym_vpn_api_client::types::VpnApiTime) -> Self {
        Self {
            local_time: value.local_time,
            estimated_remote_time: value.estimated_remote_time,
            accetably_synced: value.is_acceptable_synced(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct GatewayReport {
    pub gateway: DiagnosticResult<Option<gateway::Gateway>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub tcp: Option<DiagnosticResult<()>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub websocket: Option<DiagnosticResult<()>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub websocket_request: Option<DiagnosticResult<String>>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct RegistrationReport {
    pub mixnet_client_build: DiagnosticResult<()>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub mixnet_client_start: Option<DiagnosticResult<()>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub wireguard_registration: Option<DiagnosticResult<GatewayDataReport>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub wireguard_pings: Option<DiagnosticResult<Vec<PingReport>>>,
}

impl RegistrationReport {
    pub fn from_err(error: impl ToString) -> Self {
        Self {
            mixnet_client_build: DiagnosticResult::from_err(error),
            mixnet_client_start: None,
            wireguard_registration: None,
            wireguard_pings: None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct GatewayDataReport {
    pub public_key: String,
    pub endpoint: SocketAddr,
    pub private_ipv4: Ipv4Addr,
    pub private_ipv6: Ipv6Addr,
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_registration_common::WireguardConfiguration> for GatewayDataReport {
    fn from(value: nym_registration_common::WireguardConfiguration) -> Self {
        Self {
            public_key: value.public_key.to_base58_string(),
            endpoint: value.endpoint,
            private_ipv4: value.private_ipv4,
            private_ipv6: value.private_ipv6,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct PingReport {
    pub dst: IpAddr,
    pub delay_ms: DiagnosticResult<u128>,
}

#[derive(Debug, Clone)]
pub struct DiagnosticRunParams {
    pub gateway: Option<String>,
    pub skip_dns: bool,
    pub skip_http: bool,
}

#[derive(Debug, Clone)]
pub struct DiagnosticRegisterParams {
    pub gateway: String,
    pub storage_path: Option<PathBuf>,
    pub skip_wireguard: bool,
}
