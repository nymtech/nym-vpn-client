import Testing
import AccountPrefetchGates
import TunnelStatus

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

    @Test func tunnelIdleAllowsAccountController() {
        #expect(!AccountTunnelPrefetchGate.isTunnelActive(status: .disconnected))
        #expect(!AccountTunnelPrefetchGate.isTunnelActive(status: nil))
    }

    @Test func tunnelConnectingBlocksAccountController() {
        #expect(AccountTunnelPrefetchGate.isTunnelActive(status: .connecting))
        #expect(AccountTunnelPrefetchGate.isTunnelActive(status: .connected))
        #expect(AccountTunnelPrefetchGate.isTunnelActive(status: .disconnecting))
        #expect(AccountTunnelPrefetchGate.isTunnelActive(status: .reasserting))
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
