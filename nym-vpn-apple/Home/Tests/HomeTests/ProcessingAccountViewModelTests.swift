import Foundation
import Testing
import AccountPrefetchGates
@testable import Home

@MainActor
private final class FakeProcessing: AccountProcessing {
    enum Call: Equatable, Sendable {
        case ensure
        case prepare
        case sync
        case isActive
        case prefetch
        case syncPayment
    }

    var prepareError: Error?
    var syncPaymentError: Error?
    var accountActive = true
    var prefetchResult: ZkNymPrefetchResult = .fetchedTickets
    private(set) var calls: [Call] = []

    func ensureCredentialImportResolved() async {
        calls.append(.ensure)
    }

    func prepareRegisteredAccount() async throws {
        calls.append(.prepare)
        if let prepareError {
            throw prepareError
        }
    }

    func updateAccountSummary(force: Bool, untilActive: Bool) async {
        calls.append(.sync)
    }

    func isAccountActive() -> Bool {
        calls.append(.isActive)
        return accountActive
    }

    func prefetchZkNyms(timeout: TimeInterval) async -> ZkNymPrefetchResult {
        calls.append(.prefetch)
        return prefetchResult
    }

    func handleSubscriptionPayment() async throws {
        calls.append(.syncPayment)
        if let syncPaymentError {
            throw syncPaymentError
        }
    }
}

@MainActor
private final class FakeCoordinator: AppSessionCoordinating {
    private(set) var actions: [CoordinatorAction] = []

    func handle(_ action: CoordinatorAction) {
        actions.append(action)
    }
}

@MainActor
private func makeViewModel(
    flow: ProcessingFlow,
    processing: FakeProcessing,
    coordinator: FakeCoordinator
) -> ProcessingAccountViewModel {
    let viewModel = ProcessingAccountViewModel(processing: processing, flow: flow)
    viewModel.sessionCoordinator = coordinator
    viewModel.finalMessageDuration = 0
    return viewModel
}

@MainActor
struct ProcessingAccountViewModelTests {
    @Test func loginRunsImportPrepSyncPrefetchThenAwaitsAdvance() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        await viewModel.run()

        #expect(processing.calls == [.ensure, .prepare, .sync, .isActive, .prefetch])
        #expect(viewModel.phase == .awaitingAdvance)
        #expect(coordinator.actions.isEmpty)
    }

    @Test func createAccountSkipsPrepare() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .createAccount, processing: processing, coordinator: coordinator)

        await viewModel.run()

        #expect(processing.calls == [.ensure, .sync, .isActive, .prefetch])
        #expect(viewModel.phase == .awaitingAdvance)
    }

    @Test func postPurchaseStaticFinishesWithoutAnimation() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .postPurchase, processing: processing, coordinator: coordinator)

        await viewModel.run()

        #expect(viewModel.phase == .finished)
        #expect(coordinator.actions == [.session(.processingFinished)])
    }

    @Test func inactiveAccountSkipsPrefetchButStillAdvances() async {
        let processing = FakeProcessing()
        processing.accountActive = false
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .createAccount, processing: processing, coordinator: coordinator)

        await viewModel.run()

        #expect(processing.calls == [.ensure, .sync, .isActive])
        #expect(!processing.calls.contains(.prefetch))
        #expect(viewModel.phase == .awaitingAdvance)
    }

    @Test func prepareFailurePublishesFailedAndNotifiesCoordinator() async {
        struct Boom: Error {}
        let processing = FakeProcessing()
        processing.prepareError = Boom()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        await viewModel.run()

        guard case .failed = viewModel.phase else {
            Issue.record("expected .failed, got \(viewModel.phase)")
            return
        }
        #expect(coordinator.actions.count == 1)
        guard case .session(.processingFailed) = coordinator.actions.first else {
            Issue.record("expected .session(.processingFailed)")
            return
        }
    }

    @Test func advanceWaitsForBothWorkAndAnimation() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .createAccount, processing: processing, coordinator: coordinator)

        await viewModel.run()
        #expect(viewModel.phase == .awaitingAdvance)
        #expect(coordinator.actions.isEmpty)

        viewModel.animationDidFinish()
        #expect(viewModel.phase == .finalizing)

        await viewModel.awaitFinalMessage()
        #expect(viewModel.phase == .finished)
        #expect(coordinator.actions == [.session(.processingFinished)])
    }

    @Test func cancellationDoesNotFailOrNotify() async {
        let processing = FakeProcessing()
        processing.prepareError = CancellationError()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        await viewModel.run()

        if case .failed = viewModel.phase {
            Issue.record("cancellation should not mark failed")
        }
        #expect(coordinator.actions.isEmpty)
    }

    @Test func postPurchaseSyncsPaymentThenFinishesWhenActive() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .postPurchase, processing: processing, coordinator: coordinator)

        await viewModel.run()

        #expect(processing.calls == [.syncPayment, .isActive, .prefetch])
        #expect(viewModel.phase == .finished)
        #expect(coordinator.actions == [.session(.processingFinished)])
    }

    @Test func postPurchasePaymentSyncFailureFails() async {
        struct Boom: Error {}
        let processing = FakeProcessing()
        processing.syncPaymentError = Boom()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .postPurchase, processing: processing, coordinator: coordinator)

        await viewModel.run()

        guard case .failed = viewModel.phase else {
            Issue.record("expected .failed, got \(viewModel.phase)")
            return
        }
        #expect(!processing.calls.contains(.prefetch))
        guard case .session(.processingFailed) = coordinator.actions.first else {
            Issue.record("expected .session(.processingFailed)")
            return
        }
    }

    @Test func postPurchaseInactiveAfterPaymentFails() async {
        let processing = FakeProcessing()
        processing.accountActive = false
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .postPurchase, processing: processing, coordinator: coordinator)

        await viewModel.run()

        guard case .failed = viewModel.phase else {
            Issue.record("expected .failed, got \(viewModel.phase)")
            return
        }
        #expect(!processing.calls.contains(.prefetch))
    }
}
