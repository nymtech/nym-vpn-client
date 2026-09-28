import Foundation
import Testing
import AccountPrefetchGates

struct AccountSummaryRefreshPolicyTests {
    @Test func networkRefreshFollowsForceAndActivity() {
        #expect(AccountSummaryRefreshPolicy.shouldForceNetworkRefresh(force: true, isAccountActive: true))
        #expect(AccountSummaryRefreshPolicy.shouldForceNetworkRefresh(force: true, isAccountActive: false))
        #expect(AccountSummaryRefreshPolicy.shouldForceNetworkRefresh(force: false, isAccountActive: false))
        #expect(!AccountSummaryRefreshPolicy.shouldForceNetworkRefresh(force: false, isAccountActive: true))
    }

    @Test func pollDelaysIncludeImmediateFirstAttempt() {
        let delays = AccountSummaryRefreshPolicy.pollDelays(untilActive: true)
        #expect(delays.first == .zero)
        #expect(delays.count == 7)
    }

    @Test func manualRefreshPollWaitsAppliesAndGivesUp() {
        let timeout = AccountSummaryRefreshPolicy.manualRefreshTimeoutSeconds
        #expect(decision(hasSummary: true, stale: true, sawStale: true, last: 10, before: 10, elapsed: 2) == .keepWaiting)
        #expect(decision(hasSummary: true, stale: false, sawStale: false, last: 10, before: 10, elapsed: 2) == .keepWaiting)
        #expect(decision(hasSummary: true, stale: false, sawStale: true, last: 10, before: 10, elapsed: 0.4) == .apply)
        #expect(decision(hasSummary: true, stale: false, sawStale: false, last: 20, before: 10, elapsed: 0.4) == .apply)
        #expect(decision(hasSummary: true, stale: false, sawStale: false, last: 20, before: nil, elapsed: 0.4) == .apply)
        #expect(decision(hasSummary: false, stale: false, sawStale: false, last: nil, before: 10, elapsed: timeout) == .giveUp)
        #expect(decision(hasSummary: true, stale: true, sawStale: true, last: 10, before: 10, elapsed: timeout) == .giveUp)
    }

    @Test func vpnApiSummaryIsFreshInsideTheSyncWindow() {
        let maxAge = AccountSummaryRefreshPolicy.manualRefreshMaxSyncAgeSeconds
        #expect(AccountSummaryRefreshPolicy.isFreshVpnApiSummary(lastSyncedUnixSeconds: 1_000, nowUnixSeconds: 1_000))
        #expect(AccountSummaryRefreshPolicy.isFreshVpnApiSummary(
            lastSyncedUnixSeconds: 1_000,
            nowUnixSeconds: 1_000 + maxAge
        ))
        #expect(!AccountSummaryRefreshPolicy.isFreshVpnApiSummary(
            lastSyncedUnixSeconds: 1_000,
            nowUnixSeconds: 1_000 + maxAge + 1
        ))
        #expect(AccountSummaryRefreshPolicy.isFreshVpnApiSummary(
            lastSyncedUnixSeconds: 1_000 + maxAge,
            nowUnixSeconds: 1_000
        ))
        #expect(!AccountSummaryRefreshPolicy.isFreshVpnApiSummary(
            lastSyncedUnixSeconds: 1_000 + maxAge + 1,
            nowUnixSeconds: 1_000
        ))
    }

    private func decision(
        hasSummary: Bool,
        stale: Bool,
        sawStale: Bool,
        last: Int64?,
        before: Int64?,
        elapsed: TimeInterval
    ) -> AccountSummaryRefreshPolicy.ManualRefreshPoll {
        AccountSummaryRefreshPolicy.manualRefreshPoll(
            hasSummary: hasSummary,
            stale: stale,
            sawStale: sawStale,
            lastSyncedUnixSeconds: last,
            syncedBeforeRefresh: before,
            elapsedSeconds: elapsed
        )
    }
}
