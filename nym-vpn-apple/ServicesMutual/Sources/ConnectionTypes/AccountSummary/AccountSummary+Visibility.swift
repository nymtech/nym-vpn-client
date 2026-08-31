import Foundation

public extension AccountSummary {
    /// Account-status renew row: shown when active-but-not-auto-renewing, or inactive.
    /// Mirrors `AccountAndDevicesView+AccountStatus.accountStatusSection`.
    var shouldShowRenewRow: Bool { !isActive || !isAutoRenewEnabled }

    /// "Account on nym.com" link row: shown while the account is not yet linked.
    var shouldShowLinkAccountRow: Bool { !isLinked }

    /// Daily allowance is spent: usage has met or passed the limit. Skipped when the
    /// account is inactive or the API could not return fair-usage data (fail-open).
    var isDailyAllowanceReached: Bool {
        guard isActive, !dataUnavailable,
              let used = trafficUsedGb,
              let limit = trafficLimitGb,
              limit > 0
        else {
            return false
        }
        return used >= limit
    }
}
