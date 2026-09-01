import Foundation

public enum AccountSummaryRefreshPolicy {
    public static func shouldForceNetworkRefresh(force: Bool, isAccountActive: Bool) -> Bool {
        force || !isAccountActive
    }

    public static func pollDelays(untilActive: Bool) -> [Duration] {
        _ = untilActive
        return [
            .zero,
            .seconds(1),
            .seconds(2),
            .seconds(3),
            .seconds(4),
            .seconds(6),
            .seconds(10)
        ]
    }

    /// untilActive waits for a paid plan. Inactive is not terminal - login that
    /// must not wait passes untilActive: false instead.
    public static func shouldStopUntilActivePoll(isSubscriptionActive: Bool) -> Bool {
        isSubscriptionActive
    }

    /// Empty success is still-syncing until the last login poll. A paid summary
    /// often arrives after the third attempt (~3s).
    public static var loginEmptySuccessMinAttemptIndex: Int {
        max(0, pollDelays(untilActive: false).count - 1)
    }

    /// Login uses untilActive false. A real summary (active or inactive) is
    /// terminal. Empty success is terminal only on the last poll attempt.
    public static func shouldFinishSummaryPoll(
        untilActive: Bool,
        isSubscriptionActive: Bool,
        hasAccountSummary: Bool,
        lastFetchFailed: Bool,
        attemptIndex: Int = 0
    ) -> Bool {
        if untilActive {
            return shouldStopUntilActivePoll(isSubscriptionActive: isSubscriptionActive)
        }
        if hasAccountSummary {
            return true
        }
        if lastFetchFailed {
            return false
        }
        return attemptIndex >= loginEmptySuccessMinAttemptIndex
    }
}
