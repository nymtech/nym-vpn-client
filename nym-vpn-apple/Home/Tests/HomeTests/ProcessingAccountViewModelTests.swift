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
        case storeDeeplink(String)
        case register
        case ensureDeviceRegistered
    }

    var prepareError: Error?
    var syncPaymentError: Error?
    var storeDeeplinkError: Error?
    var registerError: Error?
    var accountActive = true
    var prefetchDelay: Duration = .zero
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
        if prefetchDelay > .zero {
            try? await Task.sleep(for: prefetchDelay)
        }
        return prefetchResult
    }

    func handleSubscriptionPayment() async throws {
        calls.append(.syncPayment)
        if let syncPaymentError {
            throw syncPaymentError
        }
    }

    func storeDeeplink(callbackURLString: String) async throws {
        calls.append(.storeDeeplink(callbackURLString))
        if let storeDeeplinkError {
            throw storeDeeplinkError
        }
    }

    func registerAccountIfNeeded() async throws {
        calls.append(.register)
        if let registerError {
            throw registerError
        }
    }

    func ensureDeviceRegisteredForLogin() async throws {
        calls.append(.ensureDeviceRegistered)
        if let registerError {
            throw registerError
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
    coordinator: FakeCoordinator,
    deeplinkLoginCallbackURL: String? = nil
) -> ProcessingAccountViewModel {
    let viewModel = ProcessingAccountViewModel(
        processing: processing,
        flow: flow,
        deeplinkLoginCallbackURL: deeplinkLoginCallbackURL
    )
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

    @Test func deeplinkLoginStoresAndRegistersBeforePrepare() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(
            flow: .login,
            processing: processing,
            coordinator: coordinator,
            deeplinkLoginCallbackURL: "nymvpn://auth/privy/privateKey?x=1"
        )

        await viewModel.run()

        #expect(processing.calls == [
            .storeDeeplink("nymvpn://auth/privy/privateKey?x=1"),
            .register,
            .ensure,
            .prepare,
            .sync,
            .isActive,
            .prefetch
        ])
        #expect(viewModel.phase == .awaitingAdvance)
        #expect(coordinator.actions.isEmpty)
    }

    @Test func deeplinkLoginStoreFailureFailsBeforePrepare() async {
        struct Boom: Error {}
        let processing = FakeProcessing()
        processing.storeDeeplinkError = Boom()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(
            flow: .login,
            processing: processing,
            coordinator: coordinator,
            deeplinkLoginCallbackURL: "nymvpn://auth/privy/privateKey"
        )

        await viewModel.run()

        #expect(!processing.calls.contains(.prepare))
        guard case .failed = viewModel.phase else {
            Issue.record("expected .failed, got \(viewModel.phase)")
            return
        }
        guard case .session(.processingFailed) = coordinator.actions.first else {
            Issue.record("expected .session(.processingFailed)")
            return
        }
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
        #expect(viewModel.currentStep == 4)
        #expect(coordinator.actions.isEmpty)

        viewModel.animationDidFinish()
        #expect(viewModel.didFinishAnimatingText)
        #expect(viewModel.phase == .finalizing)

        await viewModel.awaitFinalMessage()
        #expect(viewModel.phase == .finished)
        #expect(coordinator.actions == [.session(.processingFinished)])
    }

    @Test func inactiveAccountAdvancesToStepFourWithoutPrefetch() async {
        let processing = FakeProcessing()
        processing.accountActive = false
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .createAccount, processing: processing, coordinator: coordinator)

        await viewModel.run()
        viewModel.animationDidFinish()

        #expect(viewModel.phase == .awaitingAdvance)
        #expect(viewModel.currentStep == 4)
        #expect(viewModel.credentialsDisplayPair == nil)
        #expect(viewModel.didFinishAnimatingText)
    }

    @Test func navigationBlockedUntilSetupCarouselCompletes() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .createAccount, processing: processing, coordinator: coordinator)

        await viewModel.run()
        #expect(viewModel.phase == .awaitingAdvance)
        #expect(!viewModel.didFinishAnimatingText)
        #expect(coordinator.actions.isEmpty)
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

    @Test func prefetchPhaseHoldsStepFourUntilCallbackReturns() async {
        let processing = FakeProcessing()
        processing.prefetchDelay = .milliseconds(100)
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .createAccount, processing: processing, coordinator: coordinator)
        viewModel.animationDidFinish()

        await viewModel.run()

        #expect(processing.calls.contains(.prefetch))
        #expect(viewModel.phase == .awaitingAdvance)
        #expect(viewModel.currentStep == 4)
        #expect(viewModel.didFinishAnimatingText)
        #expect(coordinator.actions.isEmpty)
    }

    @Test func setupCarouselStepBarTick_updatesIndexAndProgressTogether() {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        viewModel.noteSetupCarouselStepBarTick(atIndex: 1)

        #expect(viewModel.setupCarouselIndex == 1)
        #expect(viewModel.currentStep == 2)
    }

    @Test func setupCarouselIndexTwoAdvancesBarToStepThree() {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        viewModel.noteSetupCarouselStepBarTick(atIndex: 2)

        #expect(viewModel.currentStep == 3)
    }

    @Test func preparingAfterSetupHoldsThirdSegmentUntilPrefetch() {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        viewModel.animationDidFinish()

        #expect(viewModel.currentStep == 3)
        #expect(viewModel.credentialsDisplayPair == nil)
    }

    @Test func loginStartsAtFirstProgressSegment() {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        #expect(viewModel.currentStep == LoginProcessingUI.initialProgressStep)
    }

    @Test func loginEnsuresDeviceRegisteredBeforePrepare() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        await viewModel.run()

        guard
            let deviceIndex = processing.calls.firstIndex(of: .ensureDeviceRegistered),
            let prepareIndex = processing.calls.firstIndex(of: .prepare)
        else {
            Issue.record("expected device registration before prepare")
            return
        }
        #expect(deviceIndex < prepareIndex)
    }
}
