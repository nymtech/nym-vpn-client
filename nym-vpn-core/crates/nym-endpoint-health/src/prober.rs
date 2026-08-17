// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashSet, sync::Arc, time::Duration};

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use nym_offline_monitor::ConnectivityMonitor;
use url::Url;

use crate::{
    EndpointClass, EndpointHealthTracker,
    probe::{probe_nym_api, probe_nyxd},
};

const CHECK_INTERVAL: Duration = Duration::from_secs(60);
/// Every this-many ticks the probed-set is cleared so all endpoints get
/// re-probed, keeping the latency measurements that drive selection ordering
/// fresh (and letting dead endpoints accrue failures toward blacklisting).
const LATENCY_REFRESH_TICKS: u32 = 15;

/// Background task that probes endpoint health: every registered endpoint is
/// probed once after it appears (at startup or registered later, e.g. by
/// on-chain signer discovery), blacklisted endpoints are re-probed once
/// their cooldown expires so recoveries are noticed without waiting for live
/// traffic to hit them, and everything is re-probed every
/// [`LATENCY_REFRESH_TICKS`] minutes to keep latency data current.
pub struct EndpointProber {
    tracker: Arc<EndpointHealthTracker>,
    expected_chain_id: Option<String>,
    http: reqwest::Client,
}

impl EndpointProber {
    pub fn spawn(
        tracker: Arc<EndpointHealthTracker>,
        expected_chain_id: Option<String>,
        connectivity_monitor: impl ConnectivityMonitor + 'static,
        cancel_token: CancellationToken,
    ) -> JoinHandle<()> {
        let prober = Self {
            tracker,
            expected_chain_id,
            http: reqwest::Client::new(),
        };
        tokio::spawn(prober.run(connectivity_monitor, cancel_token))
    }

    async fn run(
        self,
        mut connectivity_monitor: impl ConnectivityMonitor + 'static,
        cancel_token: CancellationToken,
    ) {
        tracing::debug!("Endpoint prober started");

        let mut interval = tokio::time::interval(CHECK_INTERVAL);
        let mut current_connectivity = connectivity_monitor.connectivity().await;
        let mut probed: HashSet<(EndpointClass, Url)> = HashSet::new();
        let mut ticks_until_refresh = LATENCY_REFRESH_TICKS;

        loop {
            tokio::select! {
                Some(connectivity) = connectivity_monitor.next() => {
                    current_connectivity = connectivity;
                }
                _ = interval.tick(), if current_connectivity.is_online() => {
                    ticks_until_refresh = ticks_until_refresh.saturating_sub(1);
                    if ticks_until_refresh == 0 {
                        probed.clear();
                        ticks_until_refresh = LATENCY_REFRESH_TICKS;
                    }
                    for (class, url) in probe_targets(&self.tracker, &probed) {
                        self.probe_one(class, &url).await;
                        probed.insert((class, url));
                    }
                }
                _ = cancel_token.cancelled() => {
                    tracing::debug!("Endpoint prober cancelled");
                    break;
                }
            }
        }

        tracing::debug!("Endpoint prober exiting");
    }

    async fn probe_one(&self, class: EndpointClass, url: &Url) {
        let result = match class {
            EndpointClass::NyxdRpc => {
                probe_nyxd(&self.http, url, self.expected_chain_id.as_deref()).await
            }
            EndpointClass::NymApi => probe_nym_api(&self.http, url).await,
        };
        match result {
            Ok(latency) => {
                tracing::debug!(endpoint = %url, %class, latency_ms = latency.as_millis(), "endpoint probe ok");
                self.tracker.report_success(class, url, Some(latency));
            }
            Err(failure) if failure.permanent => {
                self.tracker
                    .mark_permanent_failure(class, url, &failure.message);
            }
            Err(failure) => {
                tracing::debug!(endpoint = %url, %class, failure = %failure.message, "endpoint probe failed");
                self.tracker.report_failure(class, url, failure.kind);
            }
        }
    }
}

/// Endpoints to probe this tick: everything registered that has never been
/// probed (covers late registration, e.g. on-chain signer discovery), plus
/// blacklisted endpoints whose cooldown has expired.
fn probe_targets(
    tracker: &EndpointHealthTracker,
    probed: &HashSet<(EndpointClass, Url)>,
) -> Vec<(EndpointClass, Url)> {
    let mut targets = Vec::new();
    for class in [EndpointClass::NyxdRpc, EndpointClass::NymApi] {
        for url in tracker.all_endpoints(class) {
            if !probed.contains(&(class, url.clone())) {
                targets.push((class, url));
            }
        }
        for url in tracker.due_for_reprobe(class) {
            if !targets.contains(&(class, url.clone())) {
                targets.push((class, url));
            }
        }
    }
    targets
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FailureKind, HealthPolicy};

    fn u(s: &str) -> Url {
        s.parse().unwrap()
    }

    #[test]
    fn unprobed_endpoints_are_targets_once() {
        let tracker = EndpointHealthTracker::new();
        tracker.register(
            EndpointClass::NymApi,
            vec![u("https://a.example/"), u("https://b.example/")],
        );

        let mut probed = HashSet::new();
        let first = probe_targets(&tracker, &probed);
        assert_eq!(first.len(), 2);
        probed.extend(first);

        assert!(probe_targets(&tracker, &probed).is_empty());
    }

    #[test]
    fn late_registered_endpoints_become_targets() {
        let tracker = EndpointHealthTracker::new();
        tracker.register(EndpointClass::NymApi, vec![u("https://a.example/")]);

        let mut probed = HashSet::new();
        probed.extend(probe_targets(&tracker, &probed));

        // Simulates on-chain signer discovery registering endpoints later.
        tracker.register(EndpointClass::NymApi, vec![u("https://signer.example/")]);
        assert_eq!(
            probe_targets(&tracker, &probed),
            vec![(EndpointClass::NymApi, u("https://signer.example/"))]
        );
    }

    #[test]
    fn expired_blacklist_is_reprobed_without_duplicates() {
        let tracker = EndpointHealthTracker::with_policy(HealthPolicy {
            failure_threshold: 1,
            cooldowns: vec![std::time::Duration::from_millis(10)],
        });
        tracker.register(EndpointClass::NyxdRpc, vec![u("https://a.example/")]);

        let mut probed = HashSet::new();
        probed.extend(probe_targets(&tracker, &probed));

        tracker.report_failure(
            EndpointClass::NyxdRpc,
            &u("https://a.example/"),
            FailureKind::Connect,
        );
        assert!(
            probe_targets(&tracker, &probed).is_empty(),
            "still cooling down"
        );

        std::thread::sleep(std::time::Duration::from_millis(20));
        let targets = probe_targets(&tracker, &probed);
        assert_eq!(
            targets,
            vec![(EndpointClass::NyxdRpc, u("https://a.example/"))]
        );
    }
}
