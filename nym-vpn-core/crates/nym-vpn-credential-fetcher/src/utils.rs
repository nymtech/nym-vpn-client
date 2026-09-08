// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Shared helpers for the VPN-API credential fetcher.

use std::{future::Future, time::Duration};

use crate::error::VpnApiFetcherError;

/// Backoff bounds for retrying transient fetch failures. The fetcher retries such failures
/// indefinitely (the BandwidthController does not retry — see its `CredentialFetcher` trait doc);
/// the caller-side readiness timeout in the tunnel state machine bounds how long anything waits on a
/// fetch.
const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(5);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(60);

/// Issuance being briefly unavailable upstream is paced on its own, far slower schedule, mirroring
/// the `Retry-After: 300` the api sends with it. Every attempt spends a token from the api's
/// per-device bucket (burst 9, refilling one per 15 minutes) before the credential proxy is even
/// consulted, so the schedule above would empty the bucket within the first few minutes of a
/// condition that takes tens of minutes to clear, and the retries would then fail on rate limiting
/// rather than on the original condition.
const UPSTREAM_UNAVAILABLE_RETRY_DELAY: Duration = Duration::from_secs(300);

/// Run `op` — a fresh fetch attempt each call — retrying indefinitely on
/// [retryable](VpnApiFetcherError::is_retryable) errors with capped exponential backoff (from
/// [`INITIAL_RETRY_DELAY`], doubling up to [`MAX_RETRY_DELAY`]). A terminal error or a success
/// returns immediately. Issuance that is briefly unavailable upstream is also retried, but on the
/// slower fixed schedule of [`UPSTREAM_UNAVAILABLE_RETRY_DELAY`].
///
/// This helper does not itself watch for pause/cancellation; wrap it in
/// [`run_while_active`](crate::fetcher::VpnApiCredentialFetcher::run_while_active) so a pause cancels
/// the in-flight attempt and any backoff wait, restarting `op` from scratch on resume.
pub(crate) async fn with_retries<Fut, T>(op: impl Fn() -> Fut) -> Result<T, VpnApiFetcherError>
where
    Fut: Future<Output = Result<T, VpnApiFetcherError>>,
{
    let mut delay = INITIAL_RETRY_DELAY;
    loop {
        match op().await {
            Ok(value) => return Ok(value),
            // A dedicated arm rather than a member of `is_retryable()`: the pacing is the point,
            // and a bool cannot carry it. Folding this into the backoff arm would put it on the
            // 5s-doubling schedule, which drains the api's per-device rate bucket mid-ceremony.
            Err(err @ VpnApiFetcherError::UpstreamUnavailable) => {
                tracing::info!(
                    "Issuance is briefly unavailable, retrying in {}s: {err}",
                    UPSTREAM_UNAVAILABLE_RETRY_DELAY.as_secs()
                );
                tokio::time::sleep(UPSTREAM_UNAVAILABLE_RETRY_DELAY).await;
            }
            Err(err) if err.is_retryable() => {
                tracing::warn!("Transient fetch error, retrying in {delay:?}: {err}");
                tokio::time::sleep(delay).await;
                delay = (delay * 2).min(MAX_RETRY_DELAY);
            }
            Err(err) => {
                tracing::error!("Non-retryable fetch error: {err}");
                return Err(err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tokio::time::Instant;

    use super::*;

    #[tokio::test(start_paused = true)]
    async fn issuance_briefly_unavailable_upstream_is_retried_on_the_slow_schedule() {
        let attempts = AtomicUsize::new(0);
        let started = Instant::now();

        let result = with_retries(|| async {
            if attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                Err(VpnApiFetcherError::UpstreamUnavailable)
            } else {
                Ok(())
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        // stated independently of the constant: this mirrors the server's `Retry-After: 300`, and
        // the pace is what keeps the per-device rate bucket alive for the whole of a ceremony
        assert_eq!(started.elapsed(), Duration::from_secs(300));
    }

    #[tokio::test(start_paused = true)]
    async fn a_503_without_the_marker_still_fails_fast() {
        let attempts = AtomicUsize::new(0);

        let result: Result<(), _> = with_retries(|| async {
            attempts.fetch_add(1, Ordering::SeqCst);
            Err(VpnApiFetcherError::ApiStatusCodeError {
                endpoint: "request_zk_nym".to_string(),
                msg: "service unavailable".to_string(),
                status_code: 503,
            })
        })
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }
}
