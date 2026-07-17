// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use time::{Duration as TimeDuration, OffsetDateTime};
use tokio::time::Instant;

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

/// Type managing skew between device time and API server time.
#[derive(Debug)]
pub struct SkewManager {
    skew_state: Option<SkewState>,
    device_time_provider: Box<dyn DeviceTimeProvider + Send + Sync>,
    remote_time_provider: Box<dyn RemoteTimeProvider + Send + Sync>,
}

impl SkewManager {
    pub fn new(remote_time_provider: impl RemoteTimeProvider + Send + Sync + 'static) -> Self {
        Self {
            skew_state: None,
            device_time_provider: Box::new(DefaultDeviceTimeProvider),
            remote_time_provider: Box::new(remote_time_provider),
        }
    }

    #[cfg(test)]
    pub fn new_for_testing(
        device_time_provider: impl DeviceTimeProvider + Send + Sync + 'static,
        remote_time_provider: impl RemoteTimeProvider + Send + Sync + 'static,
    ) -> Self {
        Self {
            skew_state: None,
            device_time_provider: Box::new(device_time_provider),
            remote_time_provider: Box::new(remote_time_provider),
        }
    }

    pub async fn current_remote_time(&mut self) -> Result<Option<VpnApiTime>> {
        let now = Instant::now();
        let status = self.skew_state.as_ref().map(|state| state.status(now));

        let cached_remote_time = match status {
            Some(SkewStatus::Valid(skew)) => {
                tracing::debug!("Valid VPN API time skew");
                let local_time = self.device_time();
                let estimated_remote_time = local_time - skew;

                VpnApiTime::from_estimated_remote_time(local_time, estimated_remote_time)
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

    pub async fn sync_with_remote_time(&mut self) -> Result<Option<VpnApiTime>> {
        let remote_time = self.refresh_skew().await?;

        if Self::use_remote_time(remote_time) {
            Ok(Some(remote_time))
        } else {
            Ok(None)
        }
    }

    pub fn sync_with_response_timestamp(
        &mut self,
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
        self.store_skew(remote_time);

        if Self::use_remote_time(remote_time) {
            Ok(Some(remote_time))
        } else {
            Ok(None)
        }
    }

    pub async fn get_remote_time(&mut self) -> Result<VpnApiTime> {
        let mut last_error: Option<VpnApiClientError> = None;

        for retry in 0..=REMOTE_TIME_MAX_RETRIES {
            let time_before = self.device_time();
            match self.remote_time_provider.request_remote_time().await {
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

    async fn refresh_skew(&mut self) -> Result<VpnApiTime> {
        let remote_time = self.get_remote_time().await?;
        self.store_skew(remote_time);

        Ok(remote_time)
    }

    fn store_skew(&mut self, remote_time: VpnApiTime) {
        let skew = remote_time.local_time_ahead_skew();
        let now = Instant::now();

        self.skew_state.replace(SkewState::new(skew, now));
        tracing::debug!(skew = ?skew, "Refreshed VPN API time skew");
    }

    /// Returns the current device time.
    pub fn device_time(&self) -> OffsetDateTime {
        self.device_time_provider.device_time()
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
    use super::*;

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
        let mut skew_manager = SkewManager::new_for_testing(device_time_provider, MockTimeProvider);
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
        let mut skew_manager = SkewManager::new_for_testing(device_time_provider, MockTimeProvider);
        let time_result = skew_manager.current_remote_time().await;
        assert!(matches!(
            time_result,
            Err(VpnApiClientError::TimeTravelTooMuch)
        ));
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_sync_with_response_timestamp_updates_cache() {
        let mut skew_manager = SkewManager::new_for_testing(
            MockDeviceTimeProvider::new(vec![]),
            PanickingTimeProvider,
        );

        let time_before = OffsetDateTime::now_utc();
        let remote_timestamp = time_before - Duration::from_hours(2);
        let time_after = time_before + Duration::from_secs(1);

        let remote_time = skew_manager
            .sync_with_response_timestamp(time_before, remote_timestamp, time_after)
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
        let mut skew_manager = SkewManager::new_for_testing(
            MockDeviceTimeProvider::new(vec![]),
            PanickingTimeProvider,
        );

        let time_before = OffsetDateTime::now_utc();
        let remote_timestamp = time_before + Duration::from_secs(30);

        let remote_time = skew_manager
            .sync_with_response_timestamp(time_before, remote_timestamp, time_before)
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
        let mut skew_manager = SkewManager::new_for_testing(
            MockDeviceTimeProvider::new(vec![]),
            PanickingTimeProvider,
        );

        let time_before = OffsetDateTime::now_utc();
        let time_after = time_before - Duration::from_secs(1);

        let result =
            skew_manager.sync_with_response_timestamp(time_before, time_before, time_after);
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
