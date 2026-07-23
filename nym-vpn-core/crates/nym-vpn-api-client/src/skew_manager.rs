// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use time::{Duration as TimeDuration, OffsetDateTime};
use tokio::{
    sync::{Mutex, RwLock},
    time::Instant,
};

use crate::{
    client::NYM_VPN_API_TIMEOUT,
    error::{Result, VpnApiClientError},
    types::{VpnApiTime, VpnApiTimeSynced},
};

// Number of retries for remote time synchronization (not including the initial attempt)
const REMOTE_TIME_MAX_RETRIES: u8 = 2;

// Wait delay between retries for remote time synchronization
const REMOTE_TIME_WAIT_DELAY: Duration = Duration::from_secs(1);

const SKEW_CACHE_TTL: Duration = Duration::from_secs(4 * 60 * 60); // 4 hours

/// Type providing access to the current device time.
pub trait DeviceTimeProvider: std::fmt::Debug {
    /// Returns the current device time in UTC.
    fn device_time(&self) -> OffsetDateTime;
}

/// Type providing access to the current device time of API server.
#[async_trait::async_trait]
pub trait RemoteTimeProvider: std::fmt::Debug {
    /// Returns remote time at the API server
    ///
    /// Internally, this call should not implement retry logic to
    /// prevent caller from making wrong assumptions about how much time passed.
    async fn request_remote_time(&self) -> Result<OffsetDateTime>;
}

/// Default device time provider
#[derive(Debug)]
struct DefaultDeviceTimeProvider;
impl DeviceTimeProvider for DefaultDeviceTimeProvider {
    fn device_time(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

/// Shared component tracking the clock skew between the local device and the VPN API server.
///
/// Cheaply `Clone`-able: every clone shares the same underlying cache (via an internal `Arc`),
/// so multiple owners can hold a handle to the same skew state without needing an actor/task of
/// its own. `VpnApiClient` holds one and opportunistically refreshes it from response headers,
/// but it can just as well refresh itself on demand via the remote time provider (e.g. the
/// health endpoint) when nobody has updated it recently.
#[derive(Clone, Debug)]
pub struct SkewManager {
    inner: Arc<SkewManagerInner>,
}

#[derive(Debug)]
struct SkewManagerInner {
    skew_state: RwLock<Option<SkewState>>,
    refresh_lock: Mutex<()>,
    device_time_provider: Box<dyn DeviceTimeProvider + Send + Sync>,
    remote_time_provider: Box<dyn RemoteTimeProvider + Send + Sync>,
}

impl SkewManager {
    pub fn new(remote_time_provider: impl RemoteTimeProvider + Send + Sync + 'static) -> Self {
        Self::with_device_time_provider(DefaultDeviceTimeProvider, remote_time_provider)
    }

    pub fn new_for_testing(
        device_time_provider: impl DeviceTimeProvider + Send + Sync + 'static,
        remote_time_provider: impl RemoteTimeProvider + Send + Sync + 'static,
    ) -> Self {
        Self::with_device_time_provider(device_time_provider, remote_time_provider)
    }

    fn with_device_time_provider(
        device_time_provider: impl DeviceTimeProvider + Send + Sync + 'static,
        remote_time_provider: impl RemoteTimeProvider + Send + Sync + 'static,
    ) -> Self {
        Self {
            inner: Arc::new(SkewManagerInner {
                skew_state: RwLock::new(None),
                refresh_lock: Mutex::new(()),
                device_time_provider: Box::new(device_time_provider),
                remote_time_provider: Box::new(remote_time_provider),
            }),
        }
    }

    pub async fn current_remote_time(&self) -> Result<Option<VpnApiTime>> {
        let cached_remote_time = match self.skew_status(Instant::now()).await {
            Some(SkewStatus::Valid(skew)) => {
                tracing::debug!("Valid VPN API time skew");
                self.estimate_remote_time(skew)
            }
            Some(SkewStatus::Expired) | None => {
                tracing::debug!("VPN API time skew expired or not present, refreshing");

                self.refresh_skew().await?
            }
        };

        Ok(if Self::use_remote_time(cached_remote_time) {
            Some(cached_remote_time)
        } else {
            None
        })
    }

    pub async fn sync_with_remote_time(&self) -> Result<Option<VpnApiTime>> {
        let remote_time = self.refresh_skew().await?;

        if Self::use_remote_time(remote_time) {
            Ok(Some(remote_time))
        } else {
            Ok(None)
        }
    }

    pub async fn sync_with_response_timestamp(
        &self,
        time_before: OffsetDateTime,
        remote_timestamp: OffsetDateTime,
        time_after: OffsetDateTime,
    ) -> Result<Option<VpnApiTime>> {
        let request_time = time_after - time_before;

        // Detect sleep or time travel, in which case the timestamp cannot be trusted
        if request_time.is_negative() {
            tracing::warn!("Request time is negative. Time traveling?");
            return Err(VpnApiClientError::TimeTravelTooMuch);
        }
        if request_time > NYM_VPN_API_TIMEOUT {
            tracing::warn!("Request time exceeds the timeout. Device fell asleep?");
            return Err(VpnApiClientError::TimeTravelTooMuch);
        }

        let remote_time =
            VpnApiTime::from_remote_timestamp(time_before, remote_timestamp, time_after);
        self.store_skew(remote_time).await;

        if Self::use_remote_time(remote_time) {
            Ok(Some(remote_time))
        } else {
            Ok(None)
        }
    }

    pub async fn get_remote_time(&self) -> Result<VpnApiTime> {
        let mut last_error: Option<VpnApiClientError> = None;

        for retry in 0..=REMOTE_TIME_MAX_RETRIES {
            let time_before = self.device_time();
            match self.inner.remote_time_provider.request_remote_time().await {
                Ok(remote_timestamp) => {
                    let time_after = self.device_time();
                    let request_time = time_after - time_before;

                    // Detect sleep or time travel and retry the request
                    if request_time.is_negative() {
                        tracing::warn!(
                            "Request time is negative. Time traveling? ({}/{REMOTE_TIME_MAX_RETRIES})",
                            retry + 1
                        );
                        tokio::time::sleep(REMOTE_TIME_WAIT_DELAY).await;
                    } else if request_time > NYM_VPN_API_TIMEOUT {
                        tracing::warn!(
                            "Request time exceeds the timeout. Device fell asleep? ({}/{REMOTE_TIME_MAX_RETRIES})",
                            retry + 1
                        );
                        tokio::time::sleep(REMOTE_TIME_WAIT_DELAY).await;
                    } else {
                        return Ok(VpnApiTime::from_remote_timestamp(
                            time_before,
                            remote_timestamp,
                            time_after,
                        ));
                    }
                }
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }
        Err(last_error.unwrap_or(VpnApiClientError::TimeTravelTooMuch))
    }

    fn use_remote_time(remote_time: VpnApiTime) -> bool {
        match remote_time.is_synced() {
            VpnApiTimeSynced::AlmostSame => {
                tracing::debug!("{remote_time}");
                false
            }
            VpnApiTimeSynced::AcceptableSynced => {
                tracing::info!("{remote_time}");
                false
            }
            VpnApiTimeSynced::NotSynced => {
                tracing::warn!(
                    "The time skew between the local and remote time is too large, we'll use remote instead for JWT ({remote_time})."
                );
                true
            }
        }
    }

    async fn refresh_skew(&self) -> Result<VpnApiTime> {
        // Serialize concurrent refreshes: only the first caller to acquire this lock actually
        // hits the remote time endpoint. Everyone else re-checks the cache once they get in and
        // reuses whatever was just stored (refreshed moments ago, so just as trustworthy),
        // instead of each issuing their own redundant network request.
        let _refresh_guard = self.inner.refresh_lock.lock().await;

        if let Some(SkewStatus::Valid(skew)) = self.skew_status(Instant::now()).await {
            return Ok(self.estimate_remote_time(skew));
        }

        let remote_time = self.get_remote_time().await?;
        self.store_skew(remote_time).await;

        Ok(remote_time)
    }

    async fn skew_status(&self, now: Instant) -> Option<SkewStatus> {
        self.inner
            .skew_state
            .read()
            .await
            .as_ref()
            .map(|state| state.status(now))
    }

    fn estimate_remote_time(&self, skew: TimeDuration) -> VpnApiTime {
        let local_time = self.device_time();
        let estimated_remote_time = local_time - skew;
        VpnApiTime::from_estimated_remote_time(local_time, estimated_remote_time)
    }

    async fn store_skew(&self, remote_time: VpnApiTime) {
        let skew = remote_time.local_time_ahead_skew();
        let now = Instant::now();

        self.inner
            .skew_state
            .write()
            .await
            .replace(SkewState::new(skew, now));
        tracing::debug!(skew = ?skew, "Refreshed VPN API time skew");
    }

    /// Returns the current device time.
    pub fn device_time(&self) -> OffsetDateTime {
        self.inner.device_time_provider.device_time()
    }

    /// Returns a timestamp corrected for the skew between the local device clock and the VPN API
    /// server clock, if a valid cached skew is available right now - or `None` otherwise (skew
    /// not yet known, expired, not significant, or the cache is momentarily locked for writing).
    ///
    /// Synchronous and non-blocking: this never performs network I/O and never waits on a lock
    /// (a locked cache is simply treated as "not available"), so it's safe to call from
    /// time-sensitive paths that must not be delayed. Callers decide what "not available" means
    /// for them - typically, falling back to `device_time()`.
    ///
    /// Note this samples the device clock *now* - if the result is going to be used significantly
    /// later (e.g. after a slow async operation), prefer [`Self::cached_skew`] instead and apply
    /// it to a clock reading taken at the point of actual use, or this correction will itself
    /// become stale by the time it's used.
    pub fn cached_skew_corrected_time(&self) -> Option<OffsetDateTime> {
        self.cached_skew().map(|skew| self.device_time() - skew)
    }

    /// Returns the currently cached clock skew (how far ahead of the VPN API server's clock the
    /// local device clock is) - or `None` otherwise (skew not yet known, expired, not
    /// significant, or the cache is momentarily locked for writing).
    ///
    /// Unlike [`Self::cached_skew_corrected_time`], this doesn't sample the device clock itself:
    /// it returns just the offset, so callers can apply it to a clock reading taken right before
    /// it's actually needed (e.g. `OffsetDateTime::now_utc() - skew`) instead of one taken well
    /// before, which would let unrelated async work (network round-trips, retries, ...) make the
    /// correction stale before it's used.
    pub fn cached_skew(&self) -> Option<TimeDuration> {
        let skew = match self
            .inner
            .skew_state
            .try_read()
            .ok()?
            .as_ref()?
            .status(Instant::now())
        {
            SkewStatus::Valid(skew) => skew,
            SkewStatus::Expired => return None,
        };

        Self::use_remote_time(self.estimate_remote_time(skew)).then_some(skew)
    }
}

#[derive(Debug)]
struct SkewState {
    skew: TimeDuration,
    expires_at: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SkewStatus {
    Expired,
    Valid(TimeDuration),
}

impl SkewState {
    fn new(skew: TimeDuration, now: Instant) -> Self {
        Self {
            skew,
            expires_at: now + SKEW_CACHE_TTL,
        }
    }

    fn status(&self, now: Instant) -> SkewStatus {
        if self.expires_at > now {
            SkewStatus::Valid(self.skew)
        } else {
            SkewStatus::Expired
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_concurrent_refreshes_on_cold_cache_share_one_remote_time_request() {
        let calls = Arc::new(AtomicUsize::new(0));
        let remote_time_provider = CountingDelayedTimeProvider {
            calls: calls.clone(),
            delay: Duration::from_millis(20),
        };
        let skew_manager =
            SkewManager::new_for_testing(MockDeviceTimeProvider::new(vec![]), remote_time_provider);

        // Several concurrent lookups race against a cold (empty) cache; they must share one
        // refresh instead of each hitting the remote time endpoint.
        let (a, b, c) = tokio::join!(
            skew_manager.current_remote_time(),
            skew_manager.current_remote_time(),
            skew_manager.current_remote_time(),
        );
        a.unwrap();
        b.unwrap();
        c.unwrap();

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_cached_skew_corrected_time_uses_significant_cached_skew() {
        // Fixed so the later `device_time()` reads inside `cached_skew_corrected_time` (there
        // are two) return exactly this instant rather than drifting a few microseconds past it.
        let time_before = OffsetDateTime::now_utc();
        let remote_timestamp = time_before - Duration::from_hours(2);
        let skew_manager = SkewManager::new_for_testing(
            MockDeviceTimeProvider::new(vec![time_before; 4]),
            PanickingTimeProvider,
        );

        skew_manager
            .sync_with_response_timestamp(time_before, remote_timestamp, time_before)
            .await
            .unwrap()
            .expect("skew is significant");

        // Reads the cache synchronously: no network request (which would panic)
        let corrected = skew_manager
            .cached_skew_corrected_time()
            .expect("cached skew is significant");
        assert_eq!((time_before - corrected).whole_hours(), 2);
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_cached_skew_corrected_time_none_when_skew_negligible_or_missing() {
        let skew_manager = SkewManager::new_for_testing(
            MockDeviceTimeProvider::new(vec![]),
            PanickingTimeProvider,
        );

        // No skew has been recorded yet
        assert!(skew_manager.cached_skew_corrected_time().is_none());

        let time_before = OffsetDateTime::now_utc();
        let remote_timestamp = time_before + Duration::from_secs(30);
        skew_manager
            .sync_with_response_timestamp(time_before, remote_timestamp, time_before)
            .await
            .unwrap();

        // A negligible skew is cached, but not significant enough to correct for
        assert!(skew_manager.cached_skew_corrected_time().is_none());
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_time_travel() {
        let device_time_provider = MockDeviceTimeProvider::new(
            [
                OffsetDateTime::now_utc(),
                OffsetDateTime::now_utc() - Duration::from_hours(1),
            ]
            .repeat(3),
        );
        let skew_manager = SkewManager::new_for_testing(device_time_provider, MockTimeProvider);
        let time_result = skew_manager.current_remote_time().await;
        assert!(matches!(
            time_result,
            Err(VpnApiClientError::TimeTravelTooMuch)
        ));
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_device_sleep() {
        let device_time_provider = MockDeviceTimeProvider::new(
            [
                OffsetDateTime::now_utc(),
                OffsetDateTime::now_utc() + NYM_VPN_API_TIMEOUT + Duration::from_secs(1),
            ]
            .repeat(3),
        );
        let skew_manager = SkewManager::new_for_testing(device_time_provider, MockTimeProvider);
        let time_result = skew_manager.current_remote_time().await;
        assert!(matches!(
            time_result,
            Err(VpnApiClientError::TimeTravelTooMuch)
        ));
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_sync_with_response_timestamp_updates_cache() {
        let skew_manager = SkewManager::new_for_testing(
            MockDeviceTimeProvider::new(vec![]),
            PanickingTimeProvider,
        );

        let time_before = OffsetDateTime::now_utc();
        let remote_timestamp = time_before - Duration::from_hours(2);
        let time_after = time_before + Duration::from_secs(1);

        let remote_time = skew_manager
            .sync_with_response_timestamp(time_before, remote_timestamp, time_after)
            .await
            .unwrap()
            .expect("skew is significant");
        assert_eq!(remote_time.local_time_ahead_skew().whole_hours(), 2);

        // The derived skew is cached and reused without a remote time request
        // (which would panic)
        let cached = skew_manager
            .current_remote_time()
            .await
            .unwrap()
            .expect("cached skew is significant");
        assert_eq!(cached.local_time_ahead_skew().whole_hours(), 2);
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_sync_with_response_timestamp_negligible_skew() {
        let skew_manager = SkewManager::new_for_testing(
            MockDeviceTimeProvider::new(vec![]),
            PanickingTimeProvider,
        );

        let time_before = OffsetDateTime::now_utc();
        let remote_timestamp = time_before + Duration::from_secs(30);

        let remote_time = skew_manager
            .sync_with_response_timestamp(time_before, remote_timestamp, time_before)
            .await
            .unwrap();
        assert!(remote_time.is_none());

        // The negligible skew is still cached: no remote time request is made
        // (which would panic) and no override is returned
        let cached = skew_manager.current_remote_time().await.unwrap();
        assert!(cached.is_none());
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_sync_with_response_timestamp_time_travel() {
        let skew_manager = SkewManager::new_for_testing(
            MockDeviceTimeProvider::new(vec![]),
            PanickingTimeProvider,
        );

        let time_before = OffsetDateTime::now_utc();
        let time_after = time_before - Duration::from_secs(1);

        let result = skew_manager
            .sync_with_response_timestamp(time_before, time_before, time_after)
            .await;
        assert!(matches!(result, Err(VpnApiClientError::TimeTravelTooMuch)));
    }

    #[derive(Debug)]
    struct MockTimeProvider;

    #[async_trait::async_trait]
    impl RemoteTimeProvider for MockTimeProvider {
        async fn request_remote_time(&self) -> Result<OffsetDateTime> {
            Ok(OffsetDateTime::now_utc())
        }
    }

    #[derive(Debug)]
    struct CountingDelayedTimeProvider {
        calls: Arc<AtomicUsize>,
        delay: Duration,
    }

    #[async_trait::async_trait]
    impl RemoteTimeProvider for CountingDelayedTimeProvider {
        async fn request_remote_time(&self) -> Result<OffsetDateTime> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(self.delay).await;
            Ok(OffsetDateTime::now_utc())
        }
    }

    #[derive(Debug)]
    struct PanickingTimeProvider;

    #[async_trait::async_trait]
    impl RemoteTimeProvider for PanickingTimeProvider {
        async fn request_remote_time(&self) -> Result<OffsetDateTime> {
            panic!("remote time must not be requested");
        }
    }

    #[derive(Debug)]
    struct MockDeviceTimeProvider {
        timestamps: std::sync::Mutex<Vec<OffsetDateTime>>,
    }

    impl MockDeviceTimeProvider {
        fn new(timestamps: Vec<OffsetDateTime>) -> Self {
            Self {
                timestamps: std::sync::Mutex::new(timestamps),
            }
        }
    }

    impl DeviceTimeProvider for MockDeviceTimeProvider {
        fn device_time(&self) -> OffsetDateTime {
            let mut timestamps = self.timestamps.lock().unwrap();
            if timestamps.is_empty() {
                OffsetDateTime::now_utc()
            } else {
                timestamps.remove(0)
            }
        }
    }
}
