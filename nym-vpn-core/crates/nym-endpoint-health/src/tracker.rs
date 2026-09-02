// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use url::Url;

/// Healthy endpoints whose measured latency is within this factor of the best
/// one form the "fast tier" that shares load round-robin.
const FAST_TIER_LATENCY_FACTOR: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EndpointClass {
    NyxdRpc,
    NymApi,
}

impl std::fmt::Display for EndpointClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NyxdRpc => write!(f, "nyxd-rpc"),
            Self::NymApi => write!(f, "nym-api"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureKind {
    Connect,
    Timeout,
    Http,
    BadResponse,
}

impl std::fmt::Display for FailureKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect => write!(f, "connect"),
            Self::Timeout => write!(f, "timeout"),
            Self::Http => write!(f, "http"),
            Self::BadResponse => write!(f, "bad-response"),
        }
    }
}

/// Health policy for endpoint failure tracking.
///
/// # Invariants
///
/// - `cooldowns` must not be empty; if empty, it will be replaced with the default cooldowns
/// - `failure_threshold` must not be zero; if zero, it will be set to 1
///
/// These invariants are enforced automatically in `EndpointHealthTracker::with_policy`.
/// A panic-free fallback ensures the tracker never fails to initialize even with invalid input.
#[derive(Clone, Debug)]
pub struct HealthPolicy {
    pub failure_threshold: u32,
    pub cooldowns: Vec<Duration>,
}

impl Default for HealthPolicy {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            cooldowns: vec![
                Duration::from_secs(5 * 60),
                Duration::from_secs(15 * 60),
                Duration::from_secs(60 * 60),
            ],
        }
    }
}

impl HealthPolicy {
    /// Validates and normalizes the policy: ensures non-empty cooldowns and non-zero threshold.
    /// If either invariant is violated, falls back to a sensible default while preserving
    /// the caller's intent where possible.
    fn normalize(&self) -> Self {
        let cooldowns = if self.cooldowns.is_empty() {
            HealthPolicy::default().cooldowns
        } else {
            self.cooldowns.clone()
        };
        let failure_threshold = if self.failure_threshold == 0 {
            1
        } else {
            self.failure_threshold
        };
        Self {
            failure_threshold,
            cooldowns,
        }
    }
}

#[derive(Debug)]
struct EndpointState {
    url: Url,
    consecutive_failures: u32,
    blacklist_generation: u32,
    blacklisted_until: Option<Instant>,
    permanently_failed: bool,
    last_latency: Option<Duration>,
}

impl EndpointState {
    fn new(url: Url) -> Self {
        Self {
            url,
            consecutive_failures: 0,
            blacklist_generation: 0,
            blacklisted_until: None,
            permanently_failed: false,
            last_latency: None,
        }
    }

    fn is_blacklisted(&self, now: Instant) -> bool {
        self.blacklisted_until.is_some_and(|until| until > now)
    }
}

#[derive(Debug, Default)]
struct ClassState {
    endpoints: Vec<EndpointState>,
    cursor: usize,
}

/// Tracks per-endpoint health for a class of API endpoints, temporarily
/// blacklisting endpoints that keep failing so callers stop hammering them,
/// while guaranteeing selection never comes back empty (fail-open).
#[derive(Debug)]
pub struct EndpointHealthTracker {
    inner: Mutex<Inner>,
}

#[derive(Debug)]
struct Inner {
    policy: HealthPolicy,
    classes: HashMap<EndpointClass, ClassState>,
}

impl Default for EndpointHealthTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl EndpointHealthTracker {
    pub fn new() -> Self {
        Self::with_policy(HealthPolicy::default())
    }

    pub fn with_policy(policy: HealthPolicy) -> Self {
        Self {
            inner: Mutex::new(Inner {
                policy: policy.normalize(),
                classes: HashMap::new(),
            }),
        }
    }

    /// Add endpoints to a class, preserving order; already-known URLs keep their state.
    pub fn register(&self, class: EndpointClass, urls: Vec<Url>) {
        let mut inner = self.lock();
        let state = inner.classes.entry(class).or_default();
        for url in urls {
            if !state.endpoints.iter().any(|e| e.url == url) {
                state.endpoints.push(EndpointState::new(url));
            }
        }
    }

    /// Candidate list, preferred first. Never empty if anything is registered.
    ///
    /// Ordering policy (load spreading + latency preference): healthy
    /// endpoints with no recent failures come first — those within
    /// [`FAST_TIER_LATENCY_FACTOR`] of the best measured latency form a fast
    /// tier that is rotated round-robin across calls, spreading load over the
    /// validators; slower endpoints follow by ascending latency, then
    /// endpoints without a latency measurement yet. Recently-failed (but not
    /// yet blacklisted) endpoints come next, expired-blacklist ones last.
    /// Fail-open still applies when nothing else is eligible.
    pub fn select(&self, class: EndpointClass) -> Vec<Url> {
        let now = Instant::now();
        let mut inner = self.lock();
        let Some(state) = inner.classes.get_mut(&class) else {
            return Vec::new();
        };

        let mut clean_known: Vec<(Duration, Url)> = Vec::new();
        let mut clean_unknown: Vec<Url> = Vec::new();
        let mut suspect: Vec<Url> = Vec::new();
        let mut expired: Vec<Url> = Vec::new();
        for e in &state.endpoints {
            if e.permanently_failed || e.is_blacklisted(now) {
                continue;
            }
            if e.blacklisted_until.is_some() {
                expired.push(e.url.clone());
            } else if e.consecutive_failures > 0 {
                suspect.push(e.url.clone());
            } else if let Some(latency) = e.last_latency {
                clean_known.push((latency, e.url.clone()));
            } else {
                clean_unknown.push(e.url.clone());
            }
        }

        clean_known.sort_by_key(|(latency, _)| *latency);
        let (mut fast, slow): (Vec<Url>, Vec<Url>) = match clean_known.first() {
            Some((best, _)) => {
                let cutoff = *best * FAST_TIER_LATENCY_FACTOR;
                let split = clean_known
                    .iter()
                    .take_while(|(latency, _)| *latency <= cutoff)
                    .count();
                let slow = clean_known.split_off(split);
                (
                    clean_known.into_iter().map(|(_, url)| url).collect(),
                    slow.into_iter().map(|(_, url)| url).collect(),
                )
            }
            None => (Vec::new(), Vec::new()),
        };

        // Spread load: rotate the preferred group round-robin across calls.
        let preferred = if !fast.is_empty() {
            &mut fast
        } else {
            &mut clean_unknown
        };
        if !preferred.is_empty() {
            let offset = state.cursor % preferred.len();
            preferred.rotate_left(offset);
            state.cursor = state.cursor.wrapping_add(1);
        }

        let mut result = fast;
        result.extend(slow);
        result.extend(clean_unknown);
        result.extend(suspect);
        result.extend(expired);
        if !result.is_empty() {
            return result;
        }

        // Fail-open: all eligible endpoints are blacklisted.
        let mut fallback: Vec<&EndpointState> = state
            .endpoints
            .iter()
            .filter(|e| !e.permanently_failed)
            .collect();
        if fallback.is_empty() {
            return state.endpoints.iter().map(|e| e.url.clone()).collect();
        }
        fallback.sort_by_key(|e| e.blacklisted_until);
        fallback.into_iter().map(|e| e.url.clone()).collect()
    }

    pub fn report_failure(&self, class: EndpointClass, url: &Url, kind: FailureKind) {
        let now = Instant::now();
        let mut inner = self.lock();
        let policy = inner.policy.clone();
        let Some(state) = inner.classes.get_mut(&class) else {
            return;
        };
        let Some(endpoint) = state.endpoints.iter_mut().find(|e| e.url == *url) else {
            return;
        };

        endpoint.consecutive_failures += 1;
        let should_blacklist = endpoint.consecutive_failures >= policy.failure_threshold
            || endpoint.blacklist_generation > 0;
        if should_blacklist && !endpoint.is_blacklisted(now) {
            let tier = (endpoint.blacklist_generation as usize).min(policy.cooldowns.len() - 1);
            let cooldown = policy.cooldowns[tier];
            endpoint.blacklisted_until = Some(now + cooldown);
            endpoint.blacklist_generation += 1;
            endpoint.consecutive_failures = 0;
            tracing::warn!(
                endpoint = %endpoint.url,
                class = %class,
                failure_kind = %kind,
                cooldown_secs = cooldown.as_secs(),
                generation = endpoint.blacklist_generation,
                "endpoint blacklisted after repeated failures"
            );
        }
    }

    pub fn report_success(&self, class: EndpointClass, url: &Url, latency: Option<Duration>) {
        let mut inner = self.lock();
        let Some(state) = inner.classes.get_mut(&class) else {
            return;
        };
        let Some(endpoint) = state.endpoints.iter_mut().find(|e| e.url == *url) else {
            return;
        };
        if endpoint.blacklist_generation > 0 {
            tracing::info!(endpoint = %endpoint.url, class = %class, "endpoint recovered");
        }
        endpoint.consecutive_failures = 0;
        endpoint.blacklist_generation = 0;
        endpoint.blacklisted_until = None;
        if latency.is_some() {
            endpoint.last_latency = latency;
        }
    }

    /// Remove an endpoint from rotation for the rest of the session
    /// (e.g. it serves the wrong chain).
    pub fn mark_permanent_failure(&self, class: EndpointClass, url: &Url, reason: &str) {
        let mut inner = self.lock();
        let Some(state) = inner.classes.get_mut(&class) else {
            return;
        };
        if let Some(endpoint) = state.endpoints.iter_mut().find(|e| e.url == *url) {
            endpoint.permanently_failed = true;
            tracing::error!(endpoint = %endpoint.url, class = %class, reason, "endpoint permanently removed from rotation");
        }
    }

    /// Blacklisted endpoints whose cooldown has expired: candidates for a recovery probe.
    pub fn due_for_reprobe(&self, class: EndpointClass) -> Vec<Url> {
        let now = Instant::now();
        let inner = self.lock();
        inner
            .classes
            .get(&class)
            .map(|state| {
                state
                    .endpoints
                    .iter()
                    .filter(|e| {
                        !e.permanently_failed
                            && e.blacklisted_until.is_some_and(|until| until <= now)
                    })
                    .map(|e| e.url.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn all_endpoints(&self, class: EndpointClass) -> Vec<Url> {
        let inner = self.lock();
        inner
            .classes
            .get(&class)
            .map(|state| state.endpoints.iter().map(|e| e.url.clone()).collect())
            .unwrap_or_default()
    }

    #[allow(clippy::unwrap_used)]
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        // Mutex poisoning cannot happen: no code path panics while holding the lock.
        self.inner.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn u(s: &str) -> url::Url {
        s.parse().unwrap()
    }

    fn test_policy() -> HealthPolicy {
        HealthPolicy {
            failure_threshold: 3,
            cooldowns: vec![Duration::from_millis(50), Duration::from_millis(100)],
        }
    }

    fn tracker_with(urls: &[&str]) -> EndpointHealthTracker {
        let t = EndpointHealthTracker::with_policy(test_policy());
        t.register(EndpointClass::NyxdRpc, urls.iter().map(|s| u(s)).collect());
        t
    }

    #[test]
    fn select_spreads_load_round_robin_when_latency_unknown() {
        let t = tracker_with(&["https://a.example/", "https://b.example/"]);
        assert_eq!(
            t.select(EndpointClass::NyxdRpc),
            vec![u("https://a.example/"), u("https://b.example/")]
        );
        // consecutive selections rotate the preferred group to spread load
        assert_eq!(
            t.select(EndpointClass::NyxdRpc),
            vec![u("https://b.example/"), u("https://a.example/")]
        );
        assert_eq!(t.select(EndpointClass::NyxdRpc)[0], u("https://a.example/"));
    }

    #[test]
    fn select_prefers_lowest_latency_fast_tier_and_keeps_slow_last() {
        let t = tracker_with(&[
            "https://a.example/",
            "https://b.example/",
            "https://c.example/",
        ]);
        // a is slow (beyond 2x of best), b and c form the fast tier
        t.report_success(
            EndpointClass::NyxdRpc,
            &u("https://a.example/"),
            Some(Duration::from_millis(400)),
        );
        t.report_success(
            EndpointClass::NyxdRpc,
            &u("https://b.example/"),
            Some(Duration::from_millis(50)),
        );
        t.report_success(
            EndpointClass::NyxdRpc,
            &u("https://c.example/"),
            Some(Duration::from_millis(60)),
        );

        assert_eq!(
            t.select(EndpointClass::NyxdRpc),
            vec![
                u("https://b.example/"),
                u("https://c.example/"),
                u("https://a.example/")
            ]
        );
        // fast tier rotates across calls; the slow endpoint stays last but is never dropped
        assert_eq!(
            t.select(EndpointClass::NyxdRpc),
            vec![
                u("https://c.example/"),
                u("https://b.example/"),
                u("https://a.example/")
            ]
        );
    }

    #[test]
    fn endpoints_without_latency_sort_after_measured_fast_tier() {
        let t = tracker_with(&["https://a.example/", "https://b.example/"]);
        t.report_success(
            EndpointClass::NyxdRpc,
            &u("https://b.example/"),
            Some(Duration::from_millis(40)),
        );
        // b has a measurement, a does not: b leads on every call
        assert_eq!(
            t.select(EndpointClass::NyxdRpc),
            vec![u("https://b.example/"), u("https://a.example/")]
        );
        assert_eq!(
            t.select(EndpointClass::NyxdRpc),
            vec![u("https://b.example/"), u("https://a.example/")]
        );
    }

    #[test]
    fn register_is_idempotent_merge() {
        let t = tracker_with(&["https://a.example/"]);
        t.register(
            EndpointClass::NyxdRpc,
            vec![u("https://a.example/"), u("https://b.example/")],
        );
        assert_eq!(t.all_endpoints(EndpointClass::NyxdRpc).len(), 2);
    }

    #[test]
    fn recently_failed_endpoint_sorts_after_clean_ones() {
        let t = tracker_with(&["https://a.example/", "https://b.example/"]);
        t.report_failure(
            EndpointClass::NyxdRpc,
            &u("https://a.example/"),
            FailureKind::Connect,
        );
        // one failure doesn't blacklist, but it demotes a behind the clean b
        assert_eq!(
            t.select(EndpointClass::NyxdRpc),
            vec![u("https://b.example/"), u("https://a.example/")]
        );
    }

    #[test]
    fn three_consecutive_failures_blacklist() {
        let t = tracker_with(&["https://a.example/", "https://b.example/"]);
        for _ in 0..3 {
            t.report_failure(
                EndpointClass::NyxdRpc,
                &u("https://a.example/"),
                FailureKind::Timeout,
            );
        }
        // a is blacklisted: only b eligible
        assert_eq!(
            t.select(EndpointClass::NyxdRpc),
            vec![u("https://b.example/")]
        );
    }

    #[test]
    fn success_resets_failure_count() {
        let t = tracker_with(&["https://a.example/", "https://b.example/"]);
        t.report_failure(
            EndpointClass::NyxdRpc,
            &u("https://a.example/"),
            FailureKind::Connect,
        );
        t.report_failure(
            EndpointClass::NyxdRpc,
            &u("https://a.example/"),
            FailureKind::Connect,
        );
        t.report_success(EndpointClass::NyxdRpc, &u("https://a.example/"), None);
        t.report_failure(
            EndpointClass::NyxdRpc,
            &u("https://a.example/"),
            FailureKind::Connect,
        );
        // 1 failure after reset: still eligible
        assert!(
            t.select(EndpointClass::NyxdRpc)
                .contains(&u("https://a.example/"))
        );
    }

    #[test]
    fn cooldown_expiry_makes_endpoint_eligible_again_but_last() {
        let t = tracker_with(&["https://a.example/", "https://b.example/"]);
        for _ in 0..3 {
            t.report_failure(
                EndpointClass::NyxdRpc,
                &u("https://a.example/"),
                FailureKind::Connect,
            );
        }
        std::thread::sleep(Duration::from_millis(60));
        let sel = t.select(EndpointClass::NyxdRpc);
        assert_eq!(sel, vec![u("https://b.example/"), u("https://a.example/")]);
    }

    #[test]
    fn reblacklist_after_expiry_is_immediate_and_escalates() {
        let t = tracker_with(&["https://a.example/", "https://b.example/"]);
        for _ in 0..3 {
            t.report_failure(
                EndpointClass::NyxdRpc,
                &u("https://a.example/"),
                FailureKind::Connect,
            );
        }
        std::thread::sleep(Duration::from_millis(60));
        // one more failure re-blacklists immediately (generation > 0), now with 100ms cooldown
        t.report_failure(
            EndpointClass::NyxdRpc,
            &u("https://a.example/"),
            FailureKind::Connect,
        );
        assert_eq!(
            t.select(EndpointClass::NyxdRpc),
            vec![u("https://b.example/")]
        );
        std::thread::sleep(Duration::from_millis(60));
        // 60ms < 100ms escalated cooldown: still blacklisted
        assert_eq!(
            t.select(EndpointClass::NyxdRpc),
            vec![u("https://b.example/")]
        );
    }

    #[test]
    fn fail_open_when_all_blacklisted() {
        let t = tracker_with(&["https://a.example/", "https://b.example/"]);
        for url in ["https://a.example/", "https://b.example/"] {
            for _ in 0..3 {
                t.report_failure(EndpointClass::NyxdRpc, &u(url), FailureKind::Connect);
            }
        }
        let sel = t.select(EndpointClass::NyxdRpc);
        assert_eq!(
            sel.len(),
            2,
            "fail-open must return all non-permanent endpoints"
        );
    }

    #[test]
    fn permanent_failure_excluded_even_in_fail_open_unless_nothing_else() {
        let t = tracker_with(&["https://a.example/", "https://b.example/"]);
        t.mark_permanent_failure(
            EndpointClass::NyxdRpc,
            &u("https://a.example/"),
            "wrong chain",
        );
        assert_eq!(
            t.select(EndpointClass::NyxdRpc),
            vec![u("https://b.example/")]
        );
        for _ in 0..3 {
            t.report_failure(
                EndpointClass::NyxdRpc,
                &u("https://b.example/"),
                FailureKind::Connect,
            );
        }
        // b blacklisted, a permanent -> fail-open returns b (non-permanent) only
        assert_eq!(
            t.select(EndpointClass::NyxdRpc),
            vec![u("https://b.example/")]
        );
    }

    #[test]
    fn due_for_reprobe_lists_expired_blacklisted() {
        let t = tracker_with(&["https://a.example/", "https://b.example/"]);
        for _ in 0..3 {
            t.report_failure(
                EndpointClass::NyxdRpc,
                &u("https://a.example/"),
                FailureKind::Connect,
            );
        }
        assert!(t.due_for_reprobe(EndpointClass::NyxdRpc).is_empty());
        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(
            t.due_for_reprobe(EndpointClass::NyxdRpc),
            vec![u("https://a.example/")]
        );
    }

    #[test]
    fn classes_are_independent() {
        let t = tracker_with(&["https://a.example/"]);
        t.register(EndpointClass::NymApi, vec![u("https://a.example/")]);
        for _ in 0..3 {
            t.report_failure(
                EndpointClass::NyxdRpc,
                &u("https://a.example/"),
                FailureKind::Connect,
            );
        }
        assert_eq!(
            t.select(EndpointClass::NymApi),
            vec![u("https://a.example/")]
        );
    }

    #[test]
    fn unknown_url_reports_are_ignored() {
        let t = tracker_with(&["https://a.example/"]);
        // must not panic or create entries
        t.report_failure(
            EndpointClass::NyxdRpc,
            &u("https://ghost.example/"),
            FailureKind::Connect,
        );
        t.report_success(EndpointClass::NyxdRpc, &u("https://ghost.example/"), None);
        assert_eq!(t.all_endpoints(EndpointClass::NyxdRpc).len(), 1);
    }

    #[test]
    fn invalid_policy_is_normalized_and_doesnt_panic() {
        // Construct a tracker with an invalid policy: empty cooldowns and zero threshold.
        // The tracker should normalize it to a sensible default without panicking.
        let invalid_policy = HealthPolicy {
            failure_threshold: 0,
            cooldowns: vec![],
        };
        let t = EndpointHealthTracker::with_policy(invalid_policy);
        t.register(EndpointClass::NyxdRpc, vec![u("https://a.example/")]);

        // Report a failure. This must not panic even though the input policy was invalid.
        t.report_failure(
            EndpointClass::NyxdRpc,
            &u("https://a.example/"),
            FailureKind::Connect,
        );

        // Select should return non-empty (fail-open) and definitely not panic.
        let sel = t.select(EndpointClass::NyxdRpc);
        assert!(
            !sel.is_empty(),
            "selection must never be empty with fail-open"
        );
    }
}
