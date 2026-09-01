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
        #expect(delays.count == 7)
    }

    @Test func untilActiveDoesNotStopWhenInactive() {
        #expect(
            !AccountSummaryRefreshPolicy.shouldStopUntilActivePoll(isSubscriptionActive: false)
        )
    }

    @Test func untilActiveStopsWhenSubscriptionActive() {
        #expect(
            AccountSummaryRefreshPolicy.shouldStopUntilActivePoll(isSubscriptionActive: true)
        )
    }

    @Test func loginPollDoesNotFinishOnEmptySuccessBeforeLastAttempt() {
        #expect(
            !AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                untilActive: false,
                isSubscriptionActive: false,
                hasAccountSummary: false,
                lastFetchFailed: false,
                attemptIndex: 0
            )
        )
        #expect(
            !AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                untilActive: false,
                isSubscriptionActive: false,
                hasAccountSummary: false,
                lastFetchFailed: false,
                attemptIndex: 2
            )
        )
        let lastBeforeTerminal = AccountSummaryRefreshPolicy.loginEmptySuccessMinAttemptIndex - 1
        #expect(lastBeforeTerminal >= 0)
        #expect(
            !AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                untilActive: false,
                isSubscriptionActive: false,
                hasAccountSummary: false,
                lastFetchFailed: false,
                attemptIndex: lastBeforeTerminal
            )
        )
    }

    @Test func loginPollFinishesOnEmptySuccessfulSummaryOnLastAttempt() {
        #expect(
            AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                untilActive: false,
                isSubscriptionActive: false,
                hasAccountSummary: false,
                lastFetchFailed: false,
                attemptIndex: AccountSummaryRefreshPolicy.loginEmptySuccessMinAttemptIndex
            )
        )
    }

    @Test func loginPollFinishesImmediatelyWhenAccountKnownInactiveWithoutSummary() {
        #expect(
            AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                untilActive: false,
                isSubscriptionActive: false,
                hasAccountSummary: false,
                lastFetchFailed: false,
                attemptIndex: 0,
                isAccountKnownInactive: true
            )
        )
    }

    @Test func loginPollDoesNotFinishOnEmptyWhileStillSyncing() {
        #expect(
            !AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                untilActive: false,
                isSubscriptionActive: false,
                hasAccountSummary: false,
                lastFetchFailed: false,
                attemptIndex: 0,
                isAccountKnownInactive: false
            )
        )
    }

    @Test func iapPollDoesNotFinishWhenKnownInactiveWithoutActiveSubscription() {
        #expect(
            !AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                untilActive: true,
                isSubscriptionActive: false,
                hasAccountSummary: false,
                lastFetchFailed: false,
                attemptIndex: 0,
                isAccountKnownInactive: true
            )
        )
    }

    @Test func loginPollFinishesImmediatelyWhenInactiveSummaryExists() {
        #expect(
            AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                untilActive: false,
                isSubscriptionActive: false,
                hasAccountSummary: true,
                lastFetchFailed: false,
                attemptIndex: 0
            )
        )
    }

    @Test func loginPollRetriesWhenFetchFailed() {
        #expect(
            !AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                untilActive: false,
                isSubscriptionActive: false,
                hasAccountSummary: false,
                lastFetchFailed: true
            )
        )
    }

    @Test func iapPollDoesNotFinishOnEmptyInactiveSummary() {
        #expect(
            !AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                untilActive: true,
                isSubscriptionActive: false,
                hasAccountSummary: false,
                lastFetchFailed: false
            )
        )
    }

    @Test func iapPollDoesNotFinishWhenInactiveSummaryExists() {
        #expect(
            !AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                untilActive: true,
                isSubscriptionActive: false,
                hasAccountSummary: true,
                lastFetchFailed: false
            )
        )
    }

    @Test func iapPollFinishesWhenSubscriptionActive() {
        #expect(
            AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                untilActive: true,
                isSubscriptionActive: true,
                hasAccountSummary: true,
                lastFetchFailed: false
            )
        )
    }

    @Test func recheckInactiveStateOnlyOnLaterEmptyLoginPolls() {
        #expect(
            !AccountSummaryRefreshPolicy.shouldRecheckLoginInactiveState(
                untilActive: false,
                hasAccountSummary: false,
                attemptIndex: 0,
                alreadyKnownInactive: false
            )
        )
        #expect(
            AccountSummaryRefreshPolicy.shouldRecheckLoginInactiveState(
                untilActive: false,
                hasAccountSummary: false,
                attemptIndex: 2,
                alreadyKnownInactive: false
            )
        )
        #expect(
            !AccountSummaryRefreshPolicy.shouldRecheckLoginInactiveState(
                untilActive: false,
                hasAccountSummary: true,
                attemptIndex: 2,
                alreadyKnownInactive: false
            )
        )
        #expect(
            !AccountSummaryRefreshPolicy.shouldRecheckLoginInactiveState(
                untilActive: true,
                hasAccountSummary: false,
                attemptIndex: 2,
                alreadyKnownInactive: false
            )
        )
        #expect(
            !AccountSummaryRefreshPolicy.shouldRecheckLoginInactiveState(
                untilActive: false,
                hasAccountSummary: false,
                attemptIndex: 2,
                alreadyKnownInactive: true
            )
        )
    }
}
