import Foundation
import Testing
import AccountPrefetchGates

@MainActor
private final class FakeAccountProcessing: AccountProcessing {
    enum Event: Equatable, Sendable {
        case syncSummary
        case readActiveAfterSync
        case prefetch
    }

    var accountActiveAfterSync = true
    var prefetchResult: ZkNymPrefetchResult = .fetchedTickets
    private(set) var events: [Event] = []

    func ensureCredentialImportResolved() async {}

    func prepareRegisteredAccount() async throws {}

    func updateAccountSummary(force: Bool, untilActive: Bool) async {
        events.append(.syncSummary)
    }

    func isAccountActive() -> Bool {
        events.append(.readActiveAfterSync)
        return accountActiveAfterSync
    }

    func prefetchZkNyms(timeout: TimeInterval) async -> ZkNymPrefetchResult {
        events.append(.prefetch)
        return prefetchResult
    }

    func handleSubscriptionPayment() async throws {}
}

@MainActor
struct AccountPrefetchOrchestratorTests {
    @Test func backgroundRefreshSkipsWithoutCredential() async {
        let fake = FakeAccountProcessing()

        let outcome = await AccountPrefetchOrchestrator.runBackgroundRefresh(
            isCredentialImported: false,
            processing: fake,
            timeout: 25
        )

        #expect(outcome.skipReason == .noCredential)
        #expect(!outcome.didSyncSummary)
        #expect(outcome.prefetchResult == nil)
        #expect(fake.events.isEmpty)
    }

    @Test func backgroundRefreshSyncsOnlyWhenInactiveAfterSummary() async {
        let fake = FakeAccountProcessing()
        fake.accountActiveAfterSync = false

        let outcome = await AccountPrefetchOrchestrator.runBackgroundRefresh(
            isCredentialImported: true,
            processing: fake,
            timeout: 25
        )

        #expect(outcome.skipReason == .inactiveAfterSummarySync)
        #expect(outcome.didSyncSummary)
        #expect(outcome.prefetchResult == nil)
        #expect(fake.events == [.syncSummary, .readActiveAfterSync])
    }

    @Test func backgroundRefreshPrefetchesAfterSummaryWhenActive() async {
        let fake = FakeAccountProcessing()
        fake.prefetchResult = .skippedStoreBusy

        let outcome = await AccountPrefetchOrchestrator.runBackgroundRefresh(
            isCredentialImported: true,
            processing: fake,
            timeout: 25
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

    @Test func e5StaticPostPurchaseAdvancesWhenPrepDoneWithoutCarousel() {
        #expect(
            ProcessingAccountReadiness.canAdvanceNavigation(
                didCompleteAccountPrep: true,
                didFinishAnimatingText: false,
                requiresCarousel: false
            )
        )
    }

    @Test func e6StaticPostPurchaseBlockedUntilPrepDone() {
        #expect(
            !ProcessingAccountReadiness.canAdvanceNavigation(
                didCompleteAccountPrep: false,
                didFinishAnimatingText: false,
                requiresCarousel: false
            )
        )
    }
}
