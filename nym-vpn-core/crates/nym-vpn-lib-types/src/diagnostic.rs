// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::ApiUrl;

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
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct DiagnosticResult<T> {
    #[cfg_attr(feature = "cbor", n(0))]
    pub ok: bool,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "cbor", n(1))]
    pub value: Option<T>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "cbor", n(2))]
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
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct DiagnosticReport {
    #[cfg_attr(feature = "cbor", n(0))]
    pub dns: Option<CompleteDnsReport>,
    #[cfg_attr(feature = "cbor", n(1))]
    pub http: Option<DiagnosticResult<HttpReport>>,
    #[cfg_attr(feature = "cbor", n(2))]
    pub gateway: Option<GatewayReport>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    #[cfg_attr(feature = "cbor", n(3))]
    pub hybrid_transport: Option<DiagnosticResult<HybridTransportReport>>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct HybridTransportReport {
    #[cfg_attr(feature = "cbor", n(0))]
    pub routing_id: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub handshake_duration_ms: u128,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct CompleteDnsReport {
    #[cfg_attr(feature = "cbor", n(0))]
    pub system: DiagnosticResult<Vec<DnsResolution>>,
    #[cfg_attr(feature = "cbor", n(1))]
    pub by_nameserver: Vec<DiagnosticResult<Vec<DnsResolution>>>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct DnsResolution {
    #[cfg_attr(feature = "cbor", n(0))]
    pub nameservers: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub hostname: String,
    #[cfg_attr(feature = "cbor", n(2))]
    pub resolution: DiagnosticResult<Vec<IpAddr>>,
    #[cfg_attr(feature = "cbor", n(3))]
    pub resolution_duration_ms: u128,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct HttpReport {
    #[cfg_attr(feature = "cbor", n(0))]
    pub remote_time: DiagnosticResult<ApiTimeSkew>,
    #[cfg_attr(feature = "cbor", n(1))]
    pub health_response: DiagnosticResult<DiagnosticHealthResponse>,
    #[cfg_attr(feature = "cbor", n(2))]
    pub nb_nymnodes: DiagnosticResult<usize>,
    #[cfg_attr(feature = "cbor", n(3))]
    pub by_endpoint: Vec<DiagnosticResult<DiagnosticEndpointResponse>>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct DiagnosticEndpointResponse {
    #[cfg_attr(feature = "cbor", n(0))]
    pub status: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub url: ApiUrl,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct DiagnosticHealthResponse {
    #[cfg_attr(feature = "cbor", n(0))]
    pub status: String,
    #[cfg_attr(feature = "serde", serde(with = "time::serde::rfc3339"))]
    #[cfg_attr(feature = "cbor", n(1))]
    #[cfg_attr(feature = "cbor", cbor(with = "crate::cbor::offset_date_time"))]
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
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct ApiTimeSkew {
    // The local time on the client.
    #[cfg_attr(feature = "cbor", n(0))]
    #[cfg_attr(feature = "cbor", cbor(with = "crate::cbor::offset_date_time"))]
    pub local_time: OffsetDateTime,

    // The estimated time on the remote server. Based on RTT, it's not guaranteed to be accurate.
    #[cfg_attr(feature = "cbor", n(1))]
    #[cfg_attr(feature = "cbor", cbor(with = "crate::cbor::offset_date_time"))]
    pub estimated_remote_time: OffsetDateTime,

    #[cfg_attr(feature = "cbor", n(2))]
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
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct GatewayReport {
    #[cfg_attr(feature = "cbor", n(0))]
    pub gateway: DiagnosticResult<Option<gateway::Gateway>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "cbor", n(1))]
    pub tcp: Option<DiagnosticResult<()>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "cbor", n(2))]
    pub websocket: Option<DiagnosticResult<()>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "cbor", n(3))]
    pub websocket_request: Option<DiagnosticResult<String>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "cbor", n(4))]
    pub lp_handshake: Option<DiagnosticResult<()>>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct RegistrationReport {
    #[cfg_attr(feature = "cbor", n(0))]
    pub mixnet_client_build: DiagnosticResult<()>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "cbor", n(1))]
    pub mixnet_client_start: Option<DiagnosticResult<()>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "cbor", n(2))]
    pub mixnet_ipr_connect: Option<DiagnosticResult<()>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "cbor", n(3))]
    pub mixnet_based_dvpn_registration: Option<DiagnosticResult<GatewayDataReport>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "cbor", n(4))]
    pub lp_handshake: Option<DiagnosticResult<()>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "cbor", n(5))]
    pub lp_based_dvpn_registration: Option<DiagnosticResult<GatewayDataReport>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "cbor", n(6))]
    pub wireguard_pings: Option<DiagnosticResult<Vec<PingReport>>>,
}

impl RegistrationReport {
    pub fn from_err(error: impl ToString) -> Self {
        Self {
            mixnet_client_build: DiagnosticResult::from_err(error),
            mixnet_client_start: None,
            mixnet_ipr_connect: None,
            mixnet_based_dvpn_registration: None,
            lp_handshake: None,
            lp_based_dvpn_registration: None,
            wireguard_pings: None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct GatewayDataReport {
    #[cfg_attr(feature = "cbor", n(0))]
    pub public_key: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub endpoint: SocketAddr,
    #[cfg_attr(feature = "cbor", n(2))]
    pub psk: Option<String>,
    #[cfg_attr(feature = "cbor", n(3))]
    pub private_ipv4: Ipv4Addr,
    #[cfg_attr(feature = "cbor", n(4))]
    pub private_ipv6: Ipv6Addr,
}

#[cfg(feature = "nym-type-conversions")]
impl From<&nym_registration_common::WireguardConfiguration> for GatewayDataReport {
    fn from(value: &nym_registration_common::WireguardConfiguration) -> Self {
        Self {
            public_key: value.public_key.to_base58_string(),
            endpoint: value.endpoint,
            // While this is technically copying a PSK, it's only for display and in the diagnostic which uses ephemeral stuff anyway
            psk: value.psk.as_ref().map(|psk| {
                psk.as_bytes()
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect()
            }),
            private_ipv4: value.private_ipv4,
            private_ipv6: value.private_ipv6,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct PingReport {
    #[cfg_attr(feature = "cbor", n(0))]
    pub dst: IpAddr,
    #[cfg_attr(feature = "cbor", n(1))]
    pub delay_ms: DiagnosticResult<u128>,
}

#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct DiagnosticRunParams {
    #[cfg_attr(feature = "cbor", n(0))]
    pub gateway: Option<String>,
    #[cfg_attr(feature = "cbor", n(1))]
    pub skip_dns: bool,
    #[cfg_attr(feature = "cbor", n(2))]
    pub skip_http: bool,
    #[cfg_attr(feature = "cbor", n(3))]
    pub skip_hybrid_transport: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct DiagnosticRegisterParams {
    #[cfg_attr(feature = "cbor", n(0))]
    pub gateway: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub storage_path: Option<PathBuf>,
    #[cfg_attr(feature = "cbor", n(2))]
    pub registration_mode: RegistrationMode,
    #[cfg_attr(feature = "cbor", n(3))]
    pub skip_wireguard: bool,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
#[cfg_attr(feature = "cbor", cbor(index_only))]
pub enum RegistrationMode {
    #[default]
    #[cfg_attr(feature = "cbor", n(0))]
    Mixnet,
    #[cfg_attr(feature = "cbor", n(1))]
    Lp,
}

impl RegistrationMode {
    pub fn from_cli_flags(mixnet: bool, lp: bool) -> Self {
        if lp {
            Self::Lp
        } else if mixnet {
            Self::Mixnet
        } else {
            Self::default()
        }
    }

    pub fn is_mixnet(&self) -> bool {
        matches!(self, RegistrationMode::Mixnet)
    }
    pub fn is_lp(&self) -> bool {
        matches!(self, RegistrationMode::Lp)
    }
}
