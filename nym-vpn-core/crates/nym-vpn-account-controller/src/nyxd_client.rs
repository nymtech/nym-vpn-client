// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_endpoint_health::{EndpointClass, EndpointHealthTracker, FailureKind};
use nym_validator_client::{
    DirectSigningHttpRpcNyxdClient,
    nyxd::{Coin, Config, CosmWasmClient, cosmwasm_client::types::Account, error::NyxdError},
};
use nym_vpn_lib_types::{AccountCommandError, Mnemonic};
use nym_vpn_network_config::Network;
use std::{str::FromStr, sync::Arc};

pub struct NyxdClient {
    network: Network,
    tracker: Arc<EndpointHealthTracker>,

    // TODO: does it need locking or are we guaranteed sequential access?
    client: Option<(url::Url, Arc<DirectSigningHttpRpcNyxdClient>)>,
}

/// Outcome of a failed connection attempt against a single candidate.
enum ConnectError {
    /// Ordinary failure: the tracker's normal failure-counting/backoff should run.
    Failed(String),
    /// The candidate is permanently unsuitable (e.g. wrong chain-id). The
    /// caller has already recorded this via `mark_permanent_failure`, so the
    /// tracker's ordinary failure-counting must not also run for it.
    PermanentlyExcluded(String),
}

/// Try candidates in order until `connect` succeeds, reporting each outcome
/// to the tracker. Returns the winning url and value, or the last error.
async fn connect_first_healthy<T, F, Fut>(
    candidates: Vec<url::Url>,
    tracker: &EndpointHealthTracker,
    mut connect: F,
) -> Result<(url::Url, T), String>
where
    F: FnMut(&url::Url) -> Fut,
    Fut: Future<Output = Result<T, ConnectError>>,
{
    let mut last_error = "no nyxd endpoints available".to_string();
    for url in candidates {
        match connect(&url).await {
            Ok(value) => {
                tracker.report_success(EndpointClass::NyxdRpc, &url, None);
                return Ok((url, value));
            }
            Err(ConnectError::Failed(err)) => {
                tracing::warn!(endpoint = %url, "nyxd endpoint failed, trying next: {err}");
                tracker.report_failure(EndpointClass::NyxdRpc, &url, FailureKind::Connect);
                last_error = err;
            }
            Err(ConnectError::PermanentlyExcluded(err)) => {
                tracing::warn!(endpoint = %url, "nyxd endpoint permanently excluded, trying next: {err}");
                last_error = err;
            }
        }
    }
    Err(last_error)
}

/// Only connection-class errors should count as endpoint failures — RPC
/// transport failures and timeouts. Application-level errors (ABCI errors
/// such as "account not found", deserialization of an otherwise valid
/// response, and other domain errors) do not indicate an unhealthy endpoint
/// and must not affect endpoint health tracking or trigger a retry/rotation.
fn is_connection_error(err: &NyxdError) -> bool {
    matches!(err, NyxdError::TendermintErrorRpc(_))
}

/// Pick a `FailureKind` for a connection-class error, for reporting purposes.
fn failure_kind_for(err: &NyxdError) -> FailureKind {
    if err.is_tendermint_response_timeout() {
        FailureKind::Timeout
    } else {
        FailureKind::Connect
    }
}

impl NyxdClient {
    pub fn new(network: &Network, tracker: Arc<EndpointHealthTracker>) -> Self {
        NyxdClient {
            network: network.clone(),
            tracker,
            client: None,
        }
    }

    pub(crate) async fn ensure_connected(
        &mut self,
        mnemonic: &str,
    ) -> Result<(), AccountCommandError> {
        if self.client.is_some() {
            return Ok(());
        }
        let network_details = self.network.nym_network_details();
        let client_config =
            Config::try_from_nym_network_details(network_details).map_err(|err| {
                AccountCommandError::NyxdConnectionFailure(format!(
                    "invalid network information: {err}"
                ))
            })?;
        let mnemonic = Mnemonic::from_str(mnemonic)
            .map_err(|err| AccountCommandError::InvalidMnemonic(err.to_string()))?;

        let mut candidates = self.tracker.select(EndpointClass::NyxdRpc);
        if candidates.is_empty() {
            candidates = vec![self.network.nyxd_url()];
        }

        let expected_chain_id = self.network.expected_chain_id();
        let tracker_for_probe = self.tracker.clone();

        let (url, client) = connect_first_healthy(candidates, &self.tracker, |url| {
            let client_config = client_config.clone();
            let mnemonic = mnemonic.clone();
            let url = url.clone();
            let tracker = tracker_for_probe.clone();
            let expected_chain_id = expected_chain_id.clone();
            async move {
                let client = DirectSigningHttpRpcNyxdClient::connect_with_mnemonic(
                    client_config,
                    url.as_str(),
                    mnemonic,
                )
                .map_err(|err| ConnectError::Failed(err.to_string()))?;
                // connect_with_mnemonic does no I/O: verify the endpoint answers
                // with an account-independent read, so a valid-but-never-funded
                // (fresh) account doesn't spuriously fail this check.
                let chain_id = client
                    .get_chain_id()
                    .await
                    .map_err(|err| ConnectError::Failed(err.to_string()))?;
                if let Some(expected) = expected_chain_id
                    && chain_id.as_str() != expected
                {
                    let msg =
                        format!("wrong chain-id at {url}: expected {expected}, got {chain_id}");
                    tracker.mark_permanent_failure(EndpointClass::NyxdRpc, &url, &msg);
                    return Err(ConnectError::PermanentlyExcluded(msg));
                }
                Ok(client)
            }
        })
        .await
        .map_err(AccountCommandError::NyxdConnectionFailure)?;

        self.client = Some((url, Arc::new(client)));
        Ok(())
    }

    pub(crate) fn disconnect(&mut self) {
        self.client = None;
    }

    /// Drop the cached client and report its endpoint as failed so the next
    /// connection rotates away from it.
    fn invalidate_after_failure(&mut self, kind: FailureKind) {
        if let Some((url, _)) = self.client.take() {
            self.tracker
                .report_failure(EndpointClass::NyxdRpc, &url, kind);
        }
    }

    pub(crate) async fn get_account_details(
        &mut self,
        mnemonic: &str,
    ) -> Result<Option<Account>, AccountCommandError> {
        self.query_with_failover(mnemonic, |client| async move {
            let address = client.address();
            client.get_account(&address).await
        })
        .await
    }

    pub(crate) async fn account_balance(
        &mut self,
        mnemonic: &str,
    ) -> Result<Vec<Coin>, AccountCommandError> {
        self.query_with_failover(mnemonic, |client| async move {
            let address = client.address();
            client.get_all_balances(&address).await
        })
        .await
    }

    /// Run a read query; on failure rotate to the next healthy endpoint and retry once.
    async fn query_with_failover<T, F, Fut>(
        &mut self,
        mnemonic: &str,
        query: F,
    ) -> Result<T, AccountCommandError>
    where
        F: Fn(Arc<DirectSigningHttpRpcNyxdClient>) -> Fut,
        Fut: Future<Output = Result<T, NyxdError>>,
    {
        self.ensure_connected(mnemonic).await?;
        // SAFETY: we just connected
        #[allow(clippy::unwrap_used)]
        let client = self.client.as_ref().unwrap().1.clone();

        match query(client).await {
            Ok(value) => Ok(value),
            Err(first_err) => {
                // Application-level errors (e.g. ABCI "not found", deserialization
                // of a valid response) don't indicate an unhealthy endpoint: return
                // them directly without reporting a failure or rotating/retrying.
                if !is_connection_error(&first_err) {
                    return Err(AccountCommandError::NyxdQueryFailure(first_err.to_string()));
                }
                self.invalidate_after_failure(failure_kind_for(&first_err));
                self.ensure_connected(mnemonic).await?;
                // SAFETY: we just connected
                #[allow(clippy::unwrap_used)]
                let client = self.client.as_ref().unwrap().1.clone();
                match query(client).await {
                    Ok(value) => Ok(value),
                    Err(err) => {
                        if is_connection_error(&err) {
                            self.invalidate_after_failure(failure_kind_for(&err));
                        }
                        Err(AccountCommandError::NyxdQueryFailure(format!(
                            "{err} (after retry; first attempt: {first_err})"
                        )))
                    }
                }
            }
        }
    }

    pub(crate) async fn inner_client(
        &mut self,
        mnemonic: &str,
    ) -> Result<Arc<DirectSigningHttpRpcNyxdClient>, AccountCommandError> {
        self.ensure_connected(mnemonic).await?;
        // SAFETY: we just connected
        #[allow(clippy::unwrap_used)]
        let client = self.client.clone().unwrap().1;
        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nym_endpoint_health::{EndpointClass, EndpointHealthTracker};
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn u(s: &str) -> url::Url {
        s.parse().unwrap()
    }

    #[tokio::test]
    async fn connects_to_first_healthy_candidate() {
        let tracker = Arc::new(EndpointHealthTracker::new());
        tracker.register(
            EndpointClass::NyxdRpc,
            vec![u("https://a.example/"), u("https://b.example/")],
        );
        let attempts = AtomicUsize::new(0);
        let result = connect_first_healthy(
            vec![u("https://a.example/"), u("https://b.example/")],
            &tracker,
            |url| {
                attempts.fetch_add(1, Ordering::SeqCst);
                let url = url.clone();
                async move {
                    if url.as_str() == "https://a.example/" {
                        Err(ConnectError::Failed("a is down".to_string()))
                    } else {
                        Ok(42u32)
                    }
                }
            },
        )
        .await;
        let (url, value) = result.unwrap();
        assert_eq!(url, u("https://b.example/"));
        assert_eq!(value, 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        // failure was reported: next selection starts at b
        assert_eq!(
            tracker.select(EndpointClass::NyxdRpc)[0],
            u("https://b.example/")
        );
    }

    /// Live smoke test: real network I/O against the mainnet nyxd validator pool,
    /// with the primary endpoint replaced by a dead localhost port so failover
    /// must rotate to a third-party validator. Run manually:
    /// `cargo test -p nym-vpn-account-controller --lib -- --ignored --nocapture nyxd_live`
    #[tokio::test]
    #[ignore = "live network test; run manually"]
    async fn nyxd_live_failover_rotates_off_dead_primary() {
        let dead = u("http://127.0.0.1:1/");
        let mut network = nym_vpn_network_config::Network::mainnet_default().unwrap();
        network.nyxd_url = dead.clone();
        network.nyxd_urls = std::iter::once(dead.clone())
            .chain(
                network
                    .nyxd_urls
                    .into_iter()
                    .filter(|url| url.host_str() != Some("rpc.nymtech.net")),
            )
            .collect();
        assert!(
            network.nyxd_urls.len() >= 2,
            "need at least one live fallback validator in the bundled pool"
        );

        let tracker = Arc::new(EndpointHealthTracker::new());
        tracker.register(EndpointClass::NyxdRpc, network.nyxd_urls.clone());

        let mut client = NyxdClient::new(&network, tracker.clone());
        // Well-known BIP-39 test vector; derives a real but unused, unfunded account.
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon \
                        abandon abandon abandon abandon abandon abandon abandon abandon \
                        abandon abandon abandon abandon abandon abandon abandon art";

        let balances = client
            .account_balance(mnemonic)
            .await
            .expect("dead primary should rotate to a live third-party validator");
        println!("live failover OK; fresh-account balances: {balances:?}");

        let preferred = &tracker.select(EndpointClass::NyxdRpc)[0];
        println!("preferred endpoint after failover: {preferred}");
        assert_ne!(
            *preferred, dead,
            "rotation should have moved off the dead primary"
        );
    }

    #[tokio::test]
    async fn returns_last_error_when_all_fail() {
        let tracker = Arc::new(EndpointHealthTracker::new());
        tracker.register(EndpointClass::NyxdRpc, vec![u("https://a.example/")]);
        let result =
            connect_first_healthy(vec![u("https://a.example/")], &tracker, |_url| async move {
                Err::<u32, _>(ConnectError::Failed("boom".to_string()))
            })
            .await;
        assert_eq!(result.unwrap_err(), "boom");
    }

    #[tokio::test]
    async fn chain_mismatch_is_permanently_excluded_not_reported_as_failure() {
        let tracker = Arc::new(EndpointHealthTracker::new());
        tracker.register(EndpointClass::NyxdRpc, vec![u("https://a.example/")]);
        let result = connect_first_healthy(vec![u("https://a.example/")], &tracker, |url| {
            let url = url.clone();
            let tracker = tracker.clone();
            async move {
                tracker.mark_permanent_failure(EndpointClass::NyxdRpc, &url, "wrong chain-id");
                Err::<u32, _>(ConnectError::PermanentlyExcluded(
                    "wrong chain-id".to_string(),
                ))
            }
        })
        .await;
        assert_eq!(result.unwrap_err(), "wrong chain-id");
        // Permanently excluded, not merely blacklisted: fail-open would still
        // return it if it had only been reported as an ordinary failure, but
        // permanent exclusion drops it even from fail-open (no other endpoint
        // registered, so selection now falls back to returning it anyway --
        // what matters here is that the state is `permanently_failed`, which
        // `all_endpoints` doesn't directly expose, so we assert indirectly via
        // an additional healthy endpoint).
        tracker.register(EndpointClass::NyxdRpc, vec![u("https://b.example/")]);
        assert_eq!(
            tracker.select(EndpointClass::NyxdRpc),
            vec![u("https://b.example/")]
        );
    }
}
