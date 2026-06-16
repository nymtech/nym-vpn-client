import Testing
import AccountPrefetchGates

struct AccountZkNymPrefetchGateTests {
    @Test func prefetchSkippedWhenAccountInactiveAfterSummarySync() {
        #expect(!AccountZkNymPrefetchGate.shouldPrefetchAfterSummarySync(isAccountActive: false))
    }

    @Test func prefetchRunsWhenAccountActiveAfterSummarySync() {
        #expect(AccountZkNymPrefetchGate.shouldPrefetchAfterSummarySync(isAccountActive: true))
    }

    @Test func backgroundRefreshSkipsWithoutCredential() {
        let plan = AccountZkNymPrefetchGate.backgroundRefreshPlan(
            isCredentialImported: false,
            isAccountActiveAfterSummarySync: true
        )
        #expect(plan == .skipNoCredential)
    }

    @Test func backgroundRefreshSyncOnlyWhenInactiveAfterSummary() {
        let plan = AccountZkNymPrefetchGate.backgroundRefreshPlan(
            isCredentialImported: true,
            isAccountActiveAfterSummarySync: false
        )
        #expect(plan == .syncSummaryOnly)
    }

    @Test func backgroundRefreshPrefetchesWhenActiveAfterSummary() {
        let plan = AccountZkNymPrefetchGate.backgroundRefreshPlan(
            isCredentialImported: true,
            isAccountActiveAfterSummarySync: true
        )
        #expect(plan == .syncAndPrefetch)
    }
}

struct ZkNymPrefetchResultTests {
    @Test func storeBusyIsNotReady() {
        #expect(!ZkNymPrefetchResult.skippedStoreBusy.isReady)
    }

    @Test func fetchedTicketsIsReady() {
        #expect(ZkNymPrefetchResult.fetchedTickets.isReady)
    }

    @Test func failedIsNotReady() {
        #expect(!ZkNymPrefetchResult.failed.isReady)
    }

    @Test func skippedIsNotReady() {
        #expect(!ZkNymPrefetchResult.skipped.isReady)
    }
}
