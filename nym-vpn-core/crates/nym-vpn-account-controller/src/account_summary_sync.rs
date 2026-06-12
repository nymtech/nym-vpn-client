use nym_vpn_lib_types::VpnAccountSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountSummaryFetchFailure {
    MissingAccount,
    MissingDevice,
    Retryable(String),
}

pub fn resolve_account_summary_with_cache_fallback(
    sync_result: Result<VpnAccountSummary, AccountSummaryFetchFailure>,
    cached: Option<VpnAccountSummary>,
) -> Result<Option<VpnAccountSummary>, AccountSummaryFetchFailure> {
    match sync_result {
        Ok(summary) => Ok(Some(summary)),
        Err(
            err @ (AccountSummaryFetchFailure::MissingAccount
            | AccountSummaryFetchFailure::MissingDevice),
        ) => Err(err),
        Err(err) => match cached {
            Some(summary) => Ok(Some(summary)),
            None => Err(err),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{AccountSummaryFetchFailure, resolve_account_summary_with_cache_fallback};
    use nym_vpn_lib_types::{StoredAccountMode, VpnAccountStatus, VpnAccountSummary};
    use time::OffsetDateTime;

    fn sample_summary() -> VpnAccountSummary {
        VpnAccountSummary {
            traffic_used_gb: 0,
            traffic_limit_gb: 2000,
            traffic_reset_time: None,
            fair_usage_data_unavailable: false,
            account_addr: "n1test".into(),
            canonical_account_addr: None,
            auth_methods: vec![],
            account_mode: Some(StoredAccountMode::Api),
            subscription: None,
            is_subscription_stacked: false,
            account_status: VpnAccountStatus::Active,
            remaining_devices: 1,
            is_device_active: true,
            time_synced: true,
            stale: false,
            last_synced_utc: OffsetDateTime::now_utc(),
        }
    }

    #[test]
    fn network_failure_returns_cached_summary() {
        let cached = sample_summary();
        let resolved = resolve_account_summary_with_cache_fallback(
            Err(AccountSummaryFetchFailure::Retryable("network down".into())),
            Some(cached.clone()),
        )
        .expect("resolved");
        assert_eq!(resolved, Some(cached));
    }

    #[test]
    fn network_failure_without_cache_propagates_error() {
        let err = AccountSummaryFetchFailure::Retryable("network down".into());
        let resolved = resolve_account_summary_with_cache_fallback(Err(err.clone()), None);
        assert_eq!(resolved, Err(err));
    }

    #[test]
    fn missing_account_does_not_fall_back_to_cache() {
        let cached = sample_summary();
        let resolved = resolve_account_summary_with_cache_fallback(
            Err(AccountSummaryFetchFailure::MissingAccount),
            Some(cached),
        );
        assert_eq!(resolved, Err(AccountSummaryFetchFailure::MissingAccount));
    }
}
