import Foundation

public enum ZkNymPrefetchResult: Equatable, Sendable {
    case sufficientBandwidth
    case fetchedTickets
    case upgradeMode
    case skippedStoreBusy
    case skipped
    case failed

    /// True when local zk-nyms are sufficient or were fetched successfully.
    public var isReady: Bool {
        switch self {
        case .sufficientBandwidth, .fetchedTickets, .upgradeMode:
            return true
        case .skippedStoreBusy, .skipped, .failed:
            return false
        }
    }
}

/// Pure gates for when the app should prefetch zk-nyms (processing, background refresh).
public enum AccountZkNymPrefetchGate: Equatable, Sendable {
    public enum BackgroundRefreshPlan: Equatable, Sendable {
        case skipNoCredential
        case syncSummaryOnly
        case syncAndPrefetch
    }

    /// Prefetch only when the freshly synced summary reports an active subscription.
    public static func shouldPrefetchAfterSummarySync(isAccountActive: Bool) -> Bool {
        isAccountActive
    }

    /// Plan after summary sync (call with post-sync `isAccountActive`).
    public static func postSummarySyncPlan(isAccountActive: Bool) -> BackgroundRefreshPlan {
        isAccountActive ? .syncAndPrefetch : .syncSummaryOnly
    }

    /// Pre-sync plan: credential guard only; active status must be read after summary sync.
    public static func backgroundRefreshPlan(
        isCredentialImported: Bool,
        isAccountActiveAfterSummarySync: Bool
    ) -> BackgroundRefreshPlan {
        guard isCredentialImported else { return .skipNoCredential }
        return postSummarySyncPlan(isAccountActive: isAccountActiveAfterSummarySync)
    }
}
