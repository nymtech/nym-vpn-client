import Foundation
import Testing
import AccountPrefetchGates

private final class FakeAccountPrefetchClient: @unchecked Sendable {
    enum Event: Equatable, Sendable {
        case syncSummary
        case readActiveAfterSync
        case prefetch
    }

    var isCredentialImported = true
    var accountActiveAfterSync = true
    var prefetchResult: ZkNymPrefetchResult = .fetchedTickets
    private(set) var events: [Event] = []

    func resetEvents() {
        events = []
    }

    func recordSync() {
        events.append(.syncSummary)
    }

    func recordActiveRead() {
        events.append(.readActiveAfterSync)
    }

    func recordPrefetch() {
        events.append(.prefetch)
    }
}

struct AccountPrefetchOrchestratorTests {
    @Test func processingFlowSyncsThenPrefetchesWhenActive() async {
        let fake = FakeAccountPrefetchClient()
        fake.accountActiveAfterSync = true

        let outcome = await AccountPrefetchOrchestrator.runProcessingFlow(
            isAccountActive: {
                fake.recordActiveRead()
                return fake.accountActiveAfterSync
            },
            updateAccountSummary: {
                fake.recordSync()
            },
            prefetchZkNyms: {
                fake.recordPrefetch()
                return fake.prefetchResult
            }
        )

        #expect(outcome.didSyncSummary)
        #expect(outcome.prefetchResult == .fetchedTickets)
        #expect(fake.events == [.syncSummary, .readActiveAfterSync, .prefetch])
    }

    @Test func processingFlowSkipsPrefetchWhenInactiveAfterSync() async {
        let fake = FakeAccountPrefetchClient()
        fake.accountActiveAfterSync = false

        let outcome = await AccountPrefetchOrchestrator.runProcessingFlow(
            isAccountActive: {
                fake.recordActiveRead()
                return fake.accountActiveAfterSync
            },
            updateAccountSummary: {
                fake.recordSync()
            },
            prefetchZkNyms: {
                fake.recordPrefetch()
                return fake.prefetchResult
            }
        )

        #expect(outcome.didSyncSummary)
        #expect(outcome.prefetchResult == nil)
        #expect(fake.events == [.syncSummary, .readActiveAfterSync])
    }

    @Test func backgroundRefreshSkipsWithoutCredential() async {
        let fake = FakeAccountPrefetchClient()
        fake.isCredentialImported = false

        let outcome = await AccountPrefetchOrchestrator.runBackgroundRefresh(
            isCredentialImported: fake.isCredentialImported,
            isAccountActive: {
                fake.recordActiveRead()
                return fake.accountActiveAfterSync
            },
            updateAccountSummary: {
                fake.recordSync()
            },
            prefetchZkNyms: {
                fake.recordPrefetch()
                return fake.prefetchResult
            }
        )

        #expect(outcome.skipReason == .noCredential)
        #expect(!outcome.didSyncSummary)
        #expect(outcome.prefetchResult == nil)
        #expect(fake.events.isEmpty)
    }

    @Test func backgroundRefreshSyncsOnlyWhenInactiveAfterSummary() async {
        let fake = FakeAccountPrefetchClient()
        fake.accountActiveAfterSync = false

        let outcome = await AccountPrefetchOrchestrator.runBackgroundRefresh(
            isCredentialImported: fake.isCredentialImported,
            isAccountActive: {
                fake.recordActiveRead()
                return fake.accountActiveAfterSync
            },
            updateAccountSummary: {
                fake.recordSync()
            },
            prefetchZkNyms: {
                fake.recordPrefetch()
                return fake.prefetchResult
            }
        )

        #expect(outcome.skipReason == .inactiveAfterSummarySync)
        #expect(outcome.didSyncSummary)
        #expect(outcome.prefetchResult == nil)
        #expect(fake.events == [.syncSummary, .readActiveAfterSync])
    }

    @Test func backgroundRefreshPrefetchesAfterSummaryWhenActive() async {
        let fake = FakeAccountPrefetchClient()
        fake.prefetchResult = .skippedStoreBusy

        let outcome = await AccountPrefetchOrchestrator.runBackgroundRefresh(
            isCredentialImported: fake.isCredentialImported,
            isAccountActive: {
                fake.recordActiveRead()
                return fake.accountActiveAfterSync
            },
            updateAccountSummary: {
                fake.recordSync()
            },
            prefetchZkNyms: {
                fake.recordPrefetch()
                return fake.prefetchResult
            }
        )

        #expect(outcome.skipReason == nil)
        #expect(outcome.didSyncSummary)
        #expect(outcome.prefetchResult == .skippedStoreBusy)
        #expect(fake.events == [.syncSummary, .readActiveAfterSync, .prefetch])
    }
}

struct ProcessingAccountReadinessTests {
    @Test func navigationRequiresAccountPrepAndAnimation() {
        #expect(
            !ProcessingAccountReadiness.canAdvanceNavigation(
                didCompleteAccountPrep: false,
                didFinishAnimatingText: true
            )
        )
        #expect(
            !ProcessingAccountReadiness.canAdvanceNavigation(
                didCompleteAccountPrep: true,
                didFinishAnimatingText: false
            )
        )
        #expect(
            ProcessingAccountReadiness.canAdvanceNavigation(
                didCompleteAccountPrep: true,
                didFinishAnimatingText: true
            )
        )
    }
}
