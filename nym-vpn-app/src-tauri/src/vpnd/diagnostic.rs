use nym_vpn_lib_types as lib;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TDiagnosticResult")]
pub struct DiagnosticResult<T: TS> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: TS> DiagnosticResult<T> {
    fn from_lib<U>(result: lib::DiagnosticResult<U>, map_fn: impl FnOnce(U) -> T) -> Self {
        Self {
            ok: result.ok,
            value: result.value.map(map_fn),
            error: result.error,
        }
    }
}

impl<T: TS> From<lib::DiagnosticResult<T>> for DiagnosticResult<T> {
    fn from(r: lib::DiagnosticResult<T>) -> Self {
        Self {
            ok: r.ok,
            value: r.value,
            error: r.error,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TDiagnosticReport")]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<CompleteDnsReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<DiagnosticResult<HttpReport>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<GatewayReport>,
}

impl From<lib::DiagnosticReport> for DiagnosticReport {
    fn from(report: lib::DiagnosticReport) -> Self {
        Self {
            dns: report.dns.map(CompleteDnsReport::from),
            http: report
                .http
                .map(|r| DiagnosticResult::from_lib(r, HttpReport::from)),
            gateway: report.gateway.map(GatewayReport::from),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TCompleteDnsReport")]
#[serde(rename_all = "camelCase")]
pub struct CompleteDnsReport {
    pub system: DiagnosticResult<Vec<DnsResolution>>,
    pub by_nameserver: Vec<DiagnosticResult<Vec<DnsResolution>>>,
}

impl From<lib::CompleteDnsReport> for CompleteDnsReport {
    fn from(report: lib::CompleteDnsReport) -> Self {
        Self {
            system: DiagnosticResult::from_lib(report.system, |resolutions| {
                resolutions.into_iter().map(DnsResolution::from).collect()
            }),
            by_nameserver: report
                .by_nameserver
                .into_iter()
                .map(|r| {
                    DiagnosticResult::from_lib(r, |resolutions| {
                        resolutions.into_iter().map(DnsResolution::from).collect()
                    })
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TDnsResolution")]
#[serde(rename_all = "camelCase")]
pub struct DnsResolution {
    pub nameservers: String,
    pub hostname: String,
    pub resolution: DiagnosticResult<Vec<String>>,
    pub resolution_duration_ms: u64,
}

impl From<lib::DnsResolution> for DnsResolution {
    fn from(r: lib::DnsResolution) -> Self {
        Self {
            nameservers: r.nameservers,
            hostname: r.hostname,
            resolution: DiagnosticResult::from_lib(r.resolution, |ips| {
                ips.into_iter().map(|ip| ip.to_string()).collect()
            }),
            resolution_duration_ms: r.resolution_duration_ms as u64,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts", rename = "THttpReport")]
#[serde(rename_all = "camelCase")]
pub struct HttpReport {
    pub remote_time: DiagnosticResult<ApiTimeSkew>,
    pub health_response: DiagnosticResult<DiagnosticHealthResponse>,
    pub nb_nymnodes: DiagnosticResult<usize>,
}

impl From<lib::HttpReport> for HttpReport {
    fn from(report: lib::HttpReport) -> Self {
        Self {
            remote_time: DiagnosticResult::from_lib(report.remote_time, ApiTimeSkew::from),
            health_response: DiagnosticResult {
                ok: report.health_response.ok,
                value: report
                    .health_response
                    .value
                    .map(|h| DiagnosticHealthResponse {
                        status: h.status,
                        timestamp_utc: h.timestamp_utc.to_string(),
                    }),
                error: report.health_response.error,
            },
            nb_nymnodes: report.nb_nymnodes.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TDiagnosticHealthResponse")]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticHealthResponse {
    pub status: String,
    pub timestamp_utc: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TApiTimeSkew")]
#[serde(rename_all = "camelCase")]
pub struct ApiTimeSkew {
    pub local_time: String,
    pub estimated_remote_time: String,
    pub acceptably_synced: bool,
}

impl From<lib::ApiTimeSkew> for ApiTimeSkew {
    fn from(skew: lib::ApiTimeSkew) -> Self {
        Self {
            local_time: skew.local_time.to_string(),
            estimated_remote_time: skew.estimated_remote_time.to_string(),
            acceptably_synced: skew.accetably_synced,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TGatewayReport")]
#[serde(rename_all = "camelCase")]
pub struct GatewayReport {
    pub gateway: DiagnosticResult<Option<DiagnosticGateway>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp: Option<DiagnosticResult<()>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub websocket: Option<DiagnosticResult<()>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub websocket_request: Option<DiagnosticResult<String>>,
}

impl From<lib::GatewayReport> for GatewayReport {
    fn from(report: lib::GatewayReport) -> Self {
        Self {
            gateway: DiagnosticResult::from_lib(report.gateway, |opt_gw| {
                opt_gw.map(DiagnosticGateway::from)
            }),
            tcp: report.tcp.map(DiagnosticResult::from),
            websocket: report.websocket.map(DiagnosticResult::from),
            websocket_request: report.websocket_request.map(DiagnosticResult::from),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TDiagnosticGateway")]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticGateway {
    pub identity_key: String,
    pub name: String,
    pub description: Option<String>,
}

impl From<lib::Gateway> for DiagnosticGateway {
    fn from(gw: lib::Gateway) -> Self {
        Self {
            identity_key: gw.identity_key,
            name: gw.name,
            description: gw.description,
        }
    }
}

#[derive(Deserialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TDiagnosticRunParams")]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticRunParams {
    pub gateway: Option<String>,
    pub skip_dns: bool,
    pub skip_http: bool,
    pub skip_hybrid_transport: bool,
}

impl From<DiagnosticRunParams> for lib::DiagnosticRunParams {
    fn from(params: DiagnosticRunParams) -> Self {
        Self {
            gateway: params.gateway,
            skip_dns: params.skip_dns,
            skip_http: params.skip_http,
            skip_hybrid_transport: params.skip_hybrid_transport,
        }
    }
}
