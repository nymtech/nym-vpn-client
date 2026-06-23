import Testing
import AccountPrefetchGates

struct AccountSummaryRefreshPolicyTests {
    @Test func forceFlagRequiresNetworkRefresh() {
        #expect(AccountSummaryRefreshPolicy.shouldForceNetworkRefresh(force: true, isAccountActive: true))
        #expect(AccountSummaryRefreshPolicy.shouldForceNetworkRefresh(force: true, isAccountActive: false))
    }

    @Test func inactiveAccountRequiresNetworkRefreshWithoutForce() {
        #expect(AccountSummaryRefreshPolicy.shouldForceNetworkRefresh(force: false, isAccountActive: false))
    }

    @Test func activeAccountSkipsNetworkRefreshWhenNotForced() {
        #expect(!AccountSummaryRefreshPolicy.shouldForceNetworkRefresh(force: false, isAccountActive: true))
    }

    @Test func pollDelaysIncludeImmediateFirstAttempt() {
        let delays = AccountSummaryRefreshPolicy.pollDelays(untilActive: true)
        #expect(delays.first == .zero)
        #expect(delays.count == 5)
    }
}
