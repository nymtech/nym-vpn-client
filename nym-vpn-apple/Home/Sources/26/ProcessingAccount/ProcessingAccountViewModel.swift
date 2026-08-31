import Foundation
import SwiftUI
import AccountPrefetchGates
import Theme
#if os(iOS)
import ErrorHandler
#endif

public enum ProcessingFlow: Sendable {
    case createAccount
    case login
    case postPurchase
}

/// Observable phase machine for the account-processing screen. Runs the account
/// side effects through an injected `AccountProcessing`, publishes its `phase`, and
/// advances navigation when both the work and the carousel animation have converged.
enum ProcessingPhase: Equatable {
    case preparing
    case syncing
    case prefetching
    case awaitingAdvance
    case finalizing
    case finished
    case failed(ProcessingFailure)
}

@MainActor
@Observable
public final class ProcessingAccountViewModel {
    private let processing: AccountProcessing
    @ObservationIgnored private var processingTask: Task<Void, Never>?
    @ObservationIgnored private var finalMessageTask: Task<Void, Never>?
    @ObservationIgnored public weak var sessionCoordinator: AppSessionCoordinating?

    /// Seconds the welcome message lingers before navigating (carousel flows only).
    /// Settable so tests can drive the finalize transition without a real delay.
    @ObservationIgnored var finalMessageDuration: Double = 2

    let flow: ProcessingFlow
    @ObservationIgnored private let deeplinkLoginCallbackURL: String?
    private(set) var phase: ProcessingPhase = .preparing
    var currentStep: Int = 1
    private(set) var didFinishSetupCarousel = false
    private(set) var setupCarouselIndex = 0
    /// Set when backend prefetch begins; keeps bar segment 4 until navigation advances.
    private(set) var hasReachedPrefetchPhase = false
    @ObservationIgnored private var isCarouselInterrupted = false
    private var skipsSetupCarousel = false

    var didFinishAnimatingText = false {
        didSet { evaluateAdvance() }
    }

    @ObservationIgnored private var workCompleted = false {
        didSet { evaluateAdvance() }
    }

    var usesStaticCopy: Bool {
        flow == .postPurchase || skipsSetupCarousel
    }

    var didShowFinalMessage: Bool {
        switch phase {
        case .finalizing, .finished:
            return true
        default:
            return false
        }
    }

    private var holdsPrefetchCopyThroughAdvance: Bool {
        phase == .awaitingAdvance && hasReachedPrefetchPhase
    }

    var credentialsDisplayPair: (String, String)? {
        guard let keys = LoginProcessingProgressPolicy.credentialsCopyKeys(
            isSyncing: phase == .syncing,
            isPrefetching: phase == .prefetching,
            holdsPrefetchCopyThroughAdvance: holdsPrefetchCopyThroughAdvance,
            didFinishSetupCarousel: didFinishSetupCarousel
        ) else { return nil }
        return (keys.title.localizedString, keys.subtitle.localizedString)
    }

    public init(
        processing: AccountProcessing,
        flow: ProcessingFlow,
        deeplinkLoginCallbackURL: String? = nil
    ) {
        self.processing = processing
        self.flow = flow
        self.deeplinkLoginCallbackURL = deeplinkLoginCallbackURL
        switch flow {
        case .login, .createAccount:
            currentStep = LoginProcessingUI.initialProgressStep
        case .postPurchase:
            currentStep = PostPurchaseProcessingUI.progressStep
            didFinishAnimatingText = true
        }
    }

    func start() {
        switch phase {
        case .awaitingAdvance:
            updateAnimationReady()
            evaluateAdvance()
            return
        case .finalizing, .finished:
            return
        default:
            break
        }
        guard processingTask == nil else { return }
        processingTask = Task { @MainActor [weak self] in
            defer { self?.processingTask = nil }
            await self?.run()
        }
    }

    /// The processing sequence. Extracted from `start()` so tests can await it.
    func run() async {
        do {
            switch flow {
            case .login:
                phase = .preparing
                syncProgressStep()
                try await completeDeeplinkLoginIfNeeded()
                await processing.ensureCredentialImportResolved()
                try await processing.ensureDeviceRegisteredForLogin()
                try await processing.prepareRegisteredAccount { [weak self] accountPhase in
                    self?.applyBackendAccountPhase(accountPhase)
                }
                let isActive = try await syncSummaryThenPrefetch()
                if !isActive {
                    skipsSetupCarousel = true
                }
                completeWork()
            case .createAccount:
                phase = .preparing
                syncProgressStep()
                await processing.ensureCredentialImportResolved()
                let isActive = try await syncSummaryThenPrefetch()
                if !isActive {
                    skipsSetupCarousel = true
                }
                completeWork()
            case .postPurchase:
                try await runPostPurchase()
            }
        } catch is CancellationError {
            return
        } catch {
            fail(with: error)
        }
    }

    private func completeDeeplinkLoginIfNeeded() async throws {
        guard let deeplinkLoginCallbackURL else { return }
        try await processing.storeDeeplink(callbackURLString: deeplinkLoginCallbackURL)
        try Task.checkCancellation()
        try await processing.registerAccountIfNeeded()
        try Task.checkCancellation()
    }

    /// Login/create-account: sync the account summary, then prefetch zk-nyms when active.
    /// Returns whether the account is active after the summary sync.
    private func syncSummaryThenPrefetch() async throws -> Bool {
        try Task.checkCancellation()
        if !hasReachedPrefetchPhase {
            phase = .syncing
            syncProgressStep()
        }
        await processing.updateAccountSummary(force: true, untilActive: true)
        try Task.checkCancellation()
        let isActive = processing.isAccountActive()
        if AccountZkNymPrefetchGate.shouldPrefetchAfterSummarySync(
            isAccountActive: isActive
        ) {
            if !hasReachedPrefetchPhase {
                phase = .prefetching
                hasReachedPrefetchPhase = true
                syncProgressStep()
            }
            _ = await processing.prefetchZkNyms(timeout: LoginProcessingUI.prefetchTimeoutSeconds)
            syncProgressStep()
        }
        try Task.checkCancellation()
        return isActive
    }

    private func applyBackendAccountPhase(
        _ accountPhase: OnboardingAccountPreparationPolicy.AccountStatePhase
    ) {
        guard let displayPhase = LoginProcessingBackendPhasePolicy.displayPhase(for: accountPhase) else {
            return
        }
        switch displayPhase {
        case .syncing:
            phase = .syncing
        case .prefetching:
            phase = .prefetching
            hasReachedPrefetchPhase = true
        }
        syncProgressStep()
    }

    /// Post-IAP: sync the StoreKit receipt, fail if the account isn't active, else prefetch.
    private func runPostPurchase() async throws {
        phase = .syncing
        var didSync = true
        do {
            try await processing.handleSubscriptionPayment()
        } catch is CancellationError {
            throw CancellationError()
        } catch {
            didSync = false
        }
        try Task.checkCancellation()

        let active = processing.isAccountActive()
        guard PostPurchaseProcessingPolicy.shouldCompleteNavigation(
            didSyncSubscription: didSync,
            isAccountActive: active
        ) else {
            fail(.generic("purchasePlan.paymentFailedAlert".localizedString))
            return
        }

        phase = .prefetching
        _ = await processing.prefetchZkNyms(timeout: LoginProcessingUI.prefetchTimeoutSeconds)
        phase = .finished
        sessionCoordinator?.handle(.session(.processingFinished))
    }

    func cancel() {
        processingTask?.cancel()
        processingTask = nil
        finalMessageTask?.cancel()
        finalMessageTask = nil
    }

    func dismissPostPurchaseProcessing() {
        guard flow == .postPurchase else { return }
        cancel()
        sessionCoordinator?.handle(.dismissPostPurchaseProcessing)
    }

    func noteSetupCarouselStepBarTick(atIndex index: Int) {
        setupCarouselIndex = index
        syncProgressStep()
    }

    func animationDidFinish() {
        didFinishSetupCarousel = true
        syncProgressStep()
        updateAnimationReady()
    }

    /// Background / teardown only. Foreground work-complete still waits for animation (#6156).
    func noteCarouselInterrupted() {
        isCarouselInterrupted = true
        latchCarouselIfWorkCompleteAndInterrupted()
    }

    func noteCarouselResumed() {
        latchCarouselIfWorkCompleteAndInterrupted()
        isCarouselInterrupted = false
    }

    /// Awaits the post-advance welcome-message delay. Test hook only.
    func awaitFinalMessage() async {
        await finalMessageTask?.value
    }

    private func completeWork() {
        guard !usesStaticCopy else {
            phase = .finished
            sessionCoordinator?.handle(.session(.processingFinished))
            return
        }
        phase = .awaitingAdvance
        syncProgressStep()
        workCompleted = true
        latchCarouselIfWorkCompleteAndInterrupted()
        updateAnimationReady()
    }

    private func latchCarouselIfWorkCompleteAndInterrupted() {
        guard !usesStaticCopy, workCompleted, isCarouselInterrupted, !didFinishSetupCarousel else {
            return
        }
        animationDidFinish()
    }

    private func updateAnimationReady() {
        guard workCompleted, didFinishSetupCarousel else { return }
        didFinishAnimatingText = true
    }

    private func syncProgressStep() {
        currentStep = LoginProcessingProgressPolicy.progressStep(
            setupCarouselIndex: setupCarouselIndex,
            didFinishSetupCarousel: didFinishSetupCarousel,
            isPrefetching: phase == .prefetching,
            isAwaitingAdvance: phase == .awaitingAdvance,
            hasReachedPrefetchPhase: hasReachedPrefetchPhase
        )
    }

    private func fail(with error: Error) {
        fail(Self.mapFailure(error))
    }

    private func fail(_ failure: ProcessingFailure) {
        phase = .failed(failure)
        sessionCoordinator?.handle(.session(.processingFailed(failure)))
    }

    private func evaluateAdvance() {
        guard phase == .awaitingAdvance,
              ProcessingAccountReadiness.canAdvanceNavigation(
                  didCompleteAccountPrep: workCompleted,
                  didFinishAnimatingText: didFinishAnimatingText,
                  requiresCarousel: !usesStaticCopy
              ) else { return }
        phase = .finalizing
        finalMessageTask = Task { @MainActor [weak self] in
            guard let self else { return }
            try? await Task.sleep(for: .seconds(finalMessageDuration))
            guard !Task.isCancelled else { return }
            phase = .finished
            sessionCoordinator?.handle(.session(.processingFinished))
        }
    }

    private static func mapFailure(_ error: Error) -> ProcessingFailure {
#if os(iOS)
        if let reason = error as? VPNErrorReason {
            return .registration(reason.localizedDescription)
        }
#endif
        return .generic(error.localizedDescription)
    }
}
