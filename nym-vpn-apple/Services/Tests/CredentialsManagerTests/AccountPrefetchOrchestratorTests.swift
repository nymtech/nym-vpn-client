import Foundation
import Testing
import TunnelStatus
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

    func prepareRegisteredAccount(
        onAccountPhaseChange: (@MainActor (OnboardingAccountPreparationPolicy.AccountStatePhase) -> Void)?
    ) async throws {}

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

    func storeDeeplink(callbackURLString: String) async throws {}

    func registerAccountIfNeeded() async throws {}

    func ensureDeviceRegisteredForLogin() async throws {}
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

@MainActor
struct ProcessingFlowTests {
    @Test func processingFlowPrefetchesWhenActive() async {
        var didSync = false
        var didPrefetch = false

        let outcome = await AccountPrefetchOrchestrator.runProcessingFlow(
            isAccountActive: { true },
            updateAccountSummary: { didSync = true },
            prefetchZkNyms: {
                didPrefetch = true
                return .fetchedTickets
            }
        )

        #expect(didSync)
        #expect(didPrefetch)
        #expect(outcome.didSyncSummary)
        #expect(outcome.isAccountActive)
        #expect(outcome.prefetchResult == .fetchedTickets)
    }

    @Test func processingFlowSkipsPrefetchWhenInactive() async {
        var didPrefetch = false

        let outcome = await AccountPrefetchOrchestrator.runProcessingFlow(
            isAccountActive: { false },
            updateAccountSummary: {},
            prefetchZkNyms: {
                didPrefetch = true
                return .fetchedTickets
            }
        )

        #expect(!didPrefetch)
        #expect(outcome.didSyncSummary)
        #expect(!outcome.isAccountActive)
        #expect(outcome.prefetchResult == nil)
    }

    @Test func postPurchaseFlowPrefetchesWhenActive() async {
        var didSyncPayment = false
        var didPrefetch = false

        let outcome = await AccountPrefetchOrchestrator.runPostPurchaseProcessingFlow(
            syncSubscriptionPayment: { didSyncPayment = true },
            isAccountActive: { true },
            prefetchZkNyms: {
                didPrefetch = true
                return .sufficientBandwidth
            }
        )

        #expect(didSyncPayment)
        #expect(didPrefetch)
        #expect(outcome.didSyncSummary)
        #expect(outcome.isAccountActive)
        #expect(outcome.prefetchResult == .sufficientBandwidth)
    }

    @Test func postPurchaseFlowSkipsPrefetchWhenInactive() async {
        var didPrefetch = false

        let outcome = await AccountPrefetchOrchestrator.runPostPurchaseProcessingFlow(
            syncSubscriptionPayment: {},
            isAccountActive: { false },
            prefetchZkNyms: {
                didPrefetch = true
                return .fetchedTickets
            }
        )

        #expect(!didPrefetch)
        #expect(outcome.didSyncSummary)
        #expect(!outcome.isAccountActive)
        #expect(outcome.prefetchResult == nil)
    }

    @Test func postPurchaseFlowBailsWhenPaymentSyncThrows() async {
        struct SyncError: Error {}
        var didCheckActive = false
        var didPrefetch = false

        let outcome = await AccountPrefetchOrchestrator.runPostPurchaseProcessingFlow(
            syncSubscriptionPayment: { throw SyncError() },
            isAccountActive: {
                didCheckActive = true
                return true
            },
            prefetchZkNyms: {
                didPrefetch = true
                return .fetchedTickets
            }
        )

        #expect(!didCheckActive)
        #expect(!didPrefetch)
        #expect(!outcome.didSyncSummary)
        #expect(!outcome.isAccountActive)
        #expect(outcome.prefetchResult == nil)
    }
}

/// Covers gate/result surface not already exercised by `AccountZkNymPrefetchGateTests`:
/// the `postSummarySyncPlan` helper, the remaining ready results, and the remaining
/// tunnel statuses (restarting / offline / offlineReconnect / error / unknown).
struct AccountPrefetchGateCoverageTests {
    @Test func postSummarySyncPlanFollowsActiveFlag() {
        #expect(AccountZkNymPrefetchGate.postSummarySyncPlan(isAccountActive: true) == .syncAndPrefetch)
        #expect(AccountZkNymPrefetchGate.postSummarySyncPlan(isAccountActive: false) == .syncSummaryOnly)
    }

    @Test func remainingReadyResultsReportReady() {
        #expect(ZkNymPrefetchResult.sufficientBandwidth.isReady)
        #expect(ZkNymPrefetchResult.upgradeMode.isReady)
    }

    @Test func remainingLiveTunnelStatusesAreActive() {
        let active: [TunnelStatus] = [.restarting, .offlineReconnect, .error]
        for status in active {
            #expect(AccountTunnelPrefetchGate.isTunnelActive(status: status))
        }
    }

    @Test func remainingIdleTunnelStatusesAreInactive() {
        let inactive: [TunnelStatus] = [.offline, .unknown]
        for status in inactive {
            #expect(!AccountTunnelPrefetchGate.isTunnelActive(status: status))
        }
    }
}
