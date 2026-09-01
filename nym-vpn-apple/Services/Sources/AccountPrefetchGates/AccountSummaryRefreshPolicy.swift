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

    /// Empty success while still syncing is not unregistered. Stop on the last
    /// attempt only if the controller never reports inactive.
    public static var loginEmptySuccessMinAttemptIndex: Int {
        max(0, pollDelays(untilActive: false).count - 1)
    }

    /// Login may query controller state before the loop and on later empty polls, not on every tick.
    public static func shouldRecheckLoginInactiveState(
        untilActive: Bool,
        hasAccountSummary: Bool,
        attemptIndex: Int,
        alreadyKnownInactive: Bool
    ) -> Bool {
        guard !untilActive, !hasAccountSummary, !alreadyKnownInactive else {
            return false
        }
        return attemptIndex > 0
    }

    /// Login uses untilActive false. A real summary, or a known-inactive
    /// controller state, is terminal. Empty success while still syncing waits.
    public static func shouldFinishSummaryPoll(
        untilActive: Bool,
        isSubscriptionActive: Bool,
        hasAccountSummary: Bool,
        lastFetchFailed: Bool,
        attemptIndex: Int = 0,
        isAccountKnownInactive: Bool = false
    ) -> Bool {
        if untilActive {
            return shouldStopUntilActivePoll(isSubscriptionActive: isSubscriptionActive)
        }
        if hasAccountSummary {
            return true
        }
        if isAccountKnownInactive {
            return true
        }
        if lastFetchFailed {
            return false
        }
        return attemptIndex >= loginEmptySuccessMinAttemptIndex
    }
}
