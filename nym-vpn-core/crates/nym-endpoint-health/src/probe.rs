// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::{Duration, Instant};

use url::Url;

use crate::FailureKind;

pub(crate) const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct ProbeFailure {
    pub kind: FailureKind,
    /// The endpoint can never become valid this session (e.g. wrong chain).
    pub permanent: bool,
    pub message: String,
}

impl ProbeFailure {
    fn transient(kind: FailureKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            permanent: false,
            message: message.into(),
        }
    }
}

fn classify_reqwest_error(err: &reqwest::Error) -> FailureKind {
    if err.is_timeout() {
        FailureKind::Timeout
    } else if err.is_connect() {
        FailureKind::Connect
    } else {
        FailureKind::BadResponse
    }
}

/// Probe a Tendermint/CometBFT RPC endpoint: `GET {url}/status` must return 200,
/// report `catching_up: false`, and (when given) the expected chain-id.
pub async fn probe_nyxd(
    client: &reqwest::Client,
    url: &Url,
    expected_chain_id: Option<&str>,
) -> Result<Duration, ProbeFailure> {
    let status_url = url
        .join("status")
        .map_err(|e| ProbeFailure::transient(FailureKind::BadResponse, e.to_string()))?;

    let started = Instant::now();
    let response = client
        .get(status_url)
        .timeout(PROBE_TIMEOUT)
        .send()
        .await
        .map_err(|e| ProbeFailure::transient(classify_reqwest_error(&e), e.to_string()))?;

    if !response.status().is_success() {
        return Err(ProbeFailure::transient(
            FailureKind::Http,
            format!("status code {}", response.status()),
        ));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| ProbeFailure::transient(FailureKind::BadResponse, e.to_string()))?;
    let latency = started.elapsed();

    // Tendermint RPC over GET wraps the payload in "result".
    let result = body.get("result").unwrap_or(&body);

    let chain_id = result
        .pointer("/node_info/network")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ProbeFailure::transient(FailureKind::BadResponse, "missing node_info.network")
        })?;
    if let Some(expected) = expected_chain_id
        && chain_id != expected
    {
        return Err(ProbeFailure {
            kind: FailureKind::BadResponse,
            permanent: true,
            message: format!("wrong chain-id: expected {expected}, got {chain_id}"),
        });
    }

    let catching_up = result
        .pointer("/sync_info/catching_up")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| {
            ProbeFailure::transient(FailureKind::BadResponse, "missing sync_info.catching_up")
        })?;
    if catching_up {
        return Err(ProbeFailure::transient(
            FailureKind::BadResponse,
            "node is catching up",
        ));
    }

    Ok(latency)
}

/// Probe a nym-api endpoint (base url ends in `/api/`): a cheap contract-data path.
pub async fn probe_nym_api(client: &reqwest::Client, url: &Url) -> Result<Duration, ProbeFailure> {
    let probe_url = url
        .join("v1/epoch/reward_params")
        .map_err(|e| ProbeFailure::transient(FailureKind::BadResponse, e.to_string()))?;

    let started = Instant::now();
    let response = client
        .get(probe_url)
        .timeout(PROBE_TIMEOUT)
        .send()
        .await
        .map_err(|e| ProbeFailure::transient(classify_reqwest_error(&e), e.to_string()))?;

    if !response.status().is_success() {
        return Err(ProbeFailure::transient(
            FailureKind::Http,
            format!("status code {}", response.status()),
        ));
    }

    Ok(started.elapsed())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    fn nyxd_status_body(chain_id: &str, catching_up: bool) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": -1,
            "result": {
                "node_info": { "network": chain_id },
                "sync_info": { "catching_up": catching_up }
            }
        })
    }

    #[tokio::test]
    async fn nyxd_probe_ok() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(nyxd_status_body("nyx", false)))
            .mount(&server)
            .await;
        let client = reqwest::Client::new();
        let url = server.uri().parse().unwrap();
        assert!(probe_nyxd(&client, &url, Some("nyx")).await.is_ok());
    }

    #[tokio::test]
    async fn nyxd_probe_wrong_chain_is_permanent() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/status"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(nyxd_status_body("cosmoshub-4", false)),
            )
            .mount(&server)
            .await;
        let client = reqwest::Client::new();
        let url = server.uri().parse().unwrap();
        let err = probe_nyxd(&client, &url, Some("nyx")).await.unwrap_err();
        assert!(err.permanent);
    }

    #[tokio::test]
    async fn nyxd_probe_catching_up_fails_not_permanent() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(nyxd_status_body("nyx", true)))
            .mount(&server)
            .await;
        let client = reqwest::Client::new();
        let url = server.uri().parse().unwrap();
        let err = probe_nyxd(&client, &url, Some("nyx")).await.unwrap_err();
        assert!(!err.permanent);
        assert_eq!(err.kind, FailureKind::BadResponse);
    }

    #[tokio::test]
    async fn nyxd_probe_no_expected_chain_skips_check() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/status"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(nyxd_status_body("whatever", false)),
            )
            .mount(&server)
            .await;
        let client = reqwest::Client::new();
        let url = server.uri().parse().unwrap();
        assert!(probe_nyxd(&client, &url, None).await.is_ok());
    }

    #[tokio::test]
    async fn nyxd_probe_connect_error() {
        let client = reqwest::Client::new();
        // Reserved unroutable-ish port on localhost: bind a listener then drop it.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let url: url::Url = format!("http://127.0.0.1:{port}/").parse().unwrap();
        let err = probe_nyxd(&client, &url, Some("nyx")).await.unwrap_err();
        assert_eq!(err.kind, FailureKind::Connect);
    }

    #[tokio::test]
    async fn nym_api_probe_ok_and_http_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/epoch/reward_params"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;
        let client = reqwest::Client::new();
        let ok_url: url::Url = format!("{}/api/", server.uri()).parse().unwrap();
        assert!(probe_nym_api(&client, &ok_url).await.is_ok());

        let server2 = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/epoch/reward_params"))
            .respond_with(ResponseTemplate::new(501))
            .mount(&server2)
            .await;
        let bad_url: url::Url = format!("{}/api/", server2.uri()).parse().unwrap();
        let err = probe_nym_api(&client, &bad_url).await.unwrap_err();
        assert_eq!(err.kind, FailureKind::Http);
    }
}
