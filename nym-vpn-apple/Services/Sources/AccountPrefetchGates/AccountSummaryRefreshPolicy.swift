import Foundation

public enum AccountSummaryRefreshPolicy {
    public static let manualRefreshTimeoutSeconds: TimeInterval = 15
    public static let manualRefreshMaxSyncAgeSeconds: Int64 = 120

    public enum ManualRefreshPoll: Equatable, Sendable { case keepWaiting, apply, giveUp }

    public static func shouldForceNetworkRefresh(force: Bool, isAccountActive: Bool) -> Bool {
        force || !isAccountActive
    }

    public static func manualRefreshPoll(
        hasSummary: Bool, stale: Bool, sawStale: Bool,
        lastSyncedUnixSeconds: Int64?, syncedBeforeRefresh: Int64?,
        elapsedSeconds: TimeInterval, followUpMissing: Bool
    ) -> ManualRefreshPoll {
        if followUpMissing { return .giveUp }
        let syncAdvanced = if let lastSyncedUnixSeconds, let syncedBeforeRefresh {
            lastSyncedUnixSeconds > syncedBeforeRefresh
        } else { false }
        let appeared = hasSummary && !stale && syncedBeforeRefresh == nil && lastSyncedUnixSeconds != nil
        if hasSummary && !stale && (sawStale || syncAdvanced || appeared) { return .apply }
        return elapsedSeconds >= manualRefreshTimeoutSeconds ? .giveUp : .keepWaiting
    }

    public static func isFreshVpnApiSummary(lastSyncedUnixSeconds: Int64, nowUnixSeconds: Int64) -> Bool {
        abs(nowUnixSeconds - lastSyncedUnixSeconds) <= manualRefreshMaxSyncAgeSeconds
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
}
