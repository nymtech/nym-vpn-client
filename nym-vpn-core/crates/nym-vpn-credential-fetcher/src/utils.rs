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

/// Run `op` — a fresh fetch attempt each call — retrying indefinitely on
/// [retryable](VpnApiFetcherError::is_retryable) errors with capped exponential backoff (from
/// [`INITIAL_RETRY_DELAY`], doubling up to [`MAX_RETRY_DELAY`]). A terminal error or a success
/// returns immediately.
///
/// This helper does not itself watch for pause/cancellation; wrap it in
/// [`run_while_active`](crate::fetcher::VpnApiCredentialFetcher::run_while_active) so a pause cancels
/// the in-flight attempt and any backoff wait, restarting `op` from scratch on resume.
pub(crate) async fn with_exponential_backoff<Fut, T>(
    op: impl Fn() -> Fut,
) -> Result<T, VpnApiFetcherError>
where
    Fut: Future<Output = Result<T, VpnApiFetcherError>>,
{
    let mut delay = INITIAL_RETRY_DELAY;
    loop {
        match op().await {
            Ok(value) => return Ok(value),
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
