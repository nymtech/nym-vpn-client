import Foundation
import Testing
import AccountPrefetchGates
@testable import Home

@MainActor
final class FakeProcessing: AccountProcessing {
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
    var preparePhaseScript: [OnboardingAccountPreparationPolicy.AccountStatePhase] = []
    var holdPrepareUntilReleased = false
    private var prepareRelease: (() -> Void)?
    var syncPaymentError: Error?
    var storeDeeplinkError: Error?
    var registerError: Error?
    var accountActive = true
    var becomesActiveAfterSync = false
    var prefetchDelay: Duration = .zero
    var prefetchResult: ZkNymPrefetchResult = .fetchedTickets
    private(set) var calls: [Call] = []

    func releasePrepare() {
        prepareRelease?()
        prepareRelease = nil
    }

    func ensureCredentialImportResolved() async {
        calls.append(.ensure)
    }

    func prepareRegisteredAccount(
        onAccountPhaseChange: (@MainActor (OnboardingAccountPreparationPolicy.AccountStatePhase) -> Void)?
    ) async throws {
        calls.append(.prepare)
        for phase in preparePhaseScript {
            onAccountPhaseChange?(phase)
        }
        if holdPrepareUntilReleased {
            await withCheckedContinuation { continuation in
                prepareRelease = { continuation.resume() }
            }
        }
        if let prepareError {
            throw prepareError
        }
    }

    func updateAccountSummary(force: Bool, untilActive: Bool) async {
        calls.append(.sync)
        if becomesActiveAfterSync {
            accountActive = true
        }
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
final class FakeCoordinator: AppSessionCoordinating {
    private(set) var actions: [CoordinatorAction] = []

    func handle(_ action: CoordinatorAction) {
        actions.append(action)
    }
}

@MainActor
func makeViewModel(
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
func finishSetupCarousel(_ viewModel: ProcessingAccountViewModel) async {
    viewModel.animationDidFinish()
    await viewModel.awaitFinalMessage()
}

@MainActor
struct ProcessingAccountViewModelTests {
    @Test func loginRunsImportPrepSyncPrefetchThenAdvances() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        await viewModel.run()

<<<<<<< Updated upstream
        #expect(processing.calls == [.ensure, .ensureDeviceRegistered, .prepare, .sync, .isActive, .prefetch])
=======
        #expect(
            processing.calls == [
                .ensure, .ensureDeviceRegistered, .prepare, .sync, .isActive, .prefetch
            ]
        )
        #expect(processing.lastSyncUntilActive == false)
>>>>>>> Stashed changes
        await finishSetupCarousel(viewModel)
        #expect(viewModel.phase == .finished)
        #expect(coordinator.actions == [.session(.processingFinished)])
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
            .ensureDeviceRegistered,
            .prepare,
            .sync,
            .isActive,
            .prefetch
        ])
        #expect(viewModel.phase == .awaitingAdvance)
        await finishSetupCarousel(viewModel)
        #expect(viewModel.phase == .finished)
        #expect(coordinator.actions == [.session(.processingFinished)])
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
        await finishSetupCarousel(viewModel)
        #expect(viewModel.phase == .finished)
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
        await finishSetupCarousel(viewModel)
        #expect(viewModel.phase == .finished)
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

    @Test func workCompleteBeforeCarousel_holdsFirstSegmentUntilSetupTicks() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .createAccount, processing: processing, coordinator: coordinator)

        await viewModel.run()

        #expect(!viewModel.didFinishSetupCarousel)
        #expect(!viewModel.didFinishAnimatingText)
        #expect(viewModel.currentStep == LoginProcessingUI.initialProgressStep)
        #expect(coordinator.actions.isEmpty)
        #expect(viewModel.credentialsDisplayPair == nil)
    }

    @Test func inactiveAccountHoldsSetupBarsUntilCarouselFinishes() async {
        let processing = FakeProcessing()
        processing.accountActive = false
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .createAccount, processing: processing, coordinator: coordinator)

        await viewModel.run()
        #expect(viewModel.currentStep == LoginProcessingUI.initialProgressStep)
        #expect(viewModel.credentialsDisplayPair == nil)
        #expect(!viewModel.didFinishSetupCarousel)

        await finishSetupCarousel(viewModel)
        #expect(viewModel.currentStep == 4)
        #expect(viewModel.phase == .finished)
<<<<<<< Updated upstream
=======
        #expect(coordinator.actions == [.session(.processingFinished)])
    }

    @Test func inactiveLoginSkipsSetupCarousel() async {
        let processing = FakeProcessing()
        processing.accountActive = false
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        await viewModel.run()
        #expect(viewModel.usesStaticCopy)
        #expect(processing.lastSyncUntilActive == false)
        #expect(viewModel.phase == .finished)
        #expect(coordinator.actions == [.session(.processingFinished)])
    }

    @Test func staleInactiveCacheDoesNotSkipCarouselUntilAfterSync() async {
        let processing = FakeProcessing()
        processing.accountActive = false
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        #expect(!viewModel.usesStaticCopy)
        #expect(ProcessingUIPolicy.showsOnboardingProgressBar(usesStaticCopy: viewModel.usesStaticCopy))

        await viewModel.run()

        #expect(viewModel.usesStaticCopy)
        #expect(viewModel.phase == .finished)
    }

    @Test func staleInactiveCacheDoesNotFlipStaticCopyWhenSyncActivates() async {
        let processing = FakeProcessing()
        processing.accountActive = false
        processing.becomesActiveAfterSync = true
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        #expect(!viewModel.usesStaticCopy)
        await viewModel.run()
        #expect(!viewModel.usesStaticCopy)
>>>>>>> Stashed changes
    }

    @Test func navigationWaitsForCarouselAfterWorkCompletes() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .createAccount, processing: processing, coordinator: coordinator)

        await viewModel.run()
        #expect(coordinator.actions.isEmpty)

        await finishSetupCarousel(viewModel)
        #expect(viewModel.didFinishSetupCarousel)
        #expect(viewModel.didFinishAnimatingText)
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

    @Test func prefetchPhaseHoldsStepFourUntilCallbackReturns() async {
        let processing = FakeProcessing()
        processing.prefetchDelay = .milliseconds(100)
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .createAccount, processing: processing, coordinator: coordinator)
        viewModel.animationDidFinish()

        await viewModel.run()

        #expect(processing.calls.contains(.prefetch))
        #expect(viewModel.currentStep == 4)
        #expect(viewModel.didFinishAnimatingText)
        #expect(viewModel.phase == .finalizing)
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

    @Test func prefetchCompletingBeforeCarousel_sequencesBarsThenHoldsFourth() async {
        let processing = FakeProcessing()
        processing.prefetchDelay = .zero
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        await viewModel.run()

        #expect(viewModel.hasReachedPrefetchPhase)
        #expect(viewModel.currentStep == LoginProcessingUI.initialProgressStep)
        #expect(!viewModel.didFinishSetupCarousel)
        #expect(viewModel.credentialsDisplayPair == nil)

        viewModel.noteSetupCarouselStepBarTick(atIndex: 1)
        #expect(viewModel.currentStep == 2)

        viewModel.noteSetupCarouselStepBarTick(atIndex: 2)
        #expect(viewModel.currentStep == 3)

        viewModel.animationDidFinish()
        #expect(viewModel.currentStep == 4)
        #expect(viewModel.didFinishSetupCarousel)

        viewModel.noteSetupCarouselStepBarTick(atIndex: 0)
        #expect(viewModel.currentStep == 4)

        await viewModel.awaitFinalMessage()
        #expect(viewModel.phase == .finished)
        #expect(coordinator.actions == [.session(.processingFinished)])
    }

    @Test func startDoesNotRestartWhenWorkAlreadyCompleted() async {
        let processing = FakeProcessing()
        processing.prefetchDelay = .zero
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        await viewModel.run()
        let callsAfterRun = processing.calls

        viewModel.start()

        #expect(processing.calls == callsAfterRun)
    }

    @Test func prepareBackendPhase_showsPrefetchBeforePrepareReturns() async {
        let processing = FakeProcessing()
        processing.preparePhaseScript = [.requestingZkNyms]
        processing.holdPrepareUntilReleased = true
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        let runTask = Task { await viewModel.run() }
        await Task.yield()
        try? await Task.sleep(for: .milliseconds(20))

        #expect(viewModel.phase == .prefetching)
        #expect(viewModel.hasReachedPrefetchPhase)
        #expect(viewModel.currentStep == LoginProcessingUI.initialProgressStep)
        #expect(viewModel.setupCarouselIndex == 0)
        #expect(!viewModel.didFinishSetupCarousel)
        #expect(viewModel.credentialsDisplayPair == nil)

        processing.releasePrepare()
        await runTask.value
        await finishSetupCarousel(viewModel)
        #expect(viewModel.phase == .finished)
    }

    @Test func syncSummaryRefresh_keepsPrefetchPhaseAfterSetupFinishes() async {
        let processing = FakeProcessing()
        processing.preparePhaseScript = [.requestingZkNyms]
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        await viewModel.run()

        #expect(viewModel.hasReachedPrefetchPhase)
        #expect(viewModel.currentStep == LoginProcessingUI.initialProgressStep)
        viewModel.animationDidFinish()
        #expect(viewModel.currentStep == 4)
    }
}
