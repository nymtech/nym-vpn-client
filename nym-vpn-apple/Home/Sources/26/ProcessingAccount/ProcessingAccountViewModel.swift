import Combine
import Foundation
import Logging
import SwiftUI
#if os(iOS)
import ErrorHandler
#endif
import CredentialsManager
import UIComponents

public enum ProcessingFlow: Sendable {
    case createAccount
    case login
    case postPurchase

    var processingMode: ProcessingAccountMode {
        switch self {
        case .postPurchase:
            return .postPurchase
        case .createAccount, .login:
            return .prePurchase
        }
    }

    var carouselStepCount: Int {
        switch self {
        case .postPurchase:
            return 4
        case .createAccount, .login:
            return 3
        }
    }
}

@MainActor
@Observable
public final class ProcessingAccountViewModel {
    /// Pacing for the post-auth carousel while account prep and zk-nym prefetch run in parallel.
    public static let processingStepInterval = SwitchingTitlesView.accountProcessingStepInterval
    /// Minimum carousel pacing before account-ready (see spec A5).
    static let minimumCarouselPacing: TimeInterval = processingStepInterval
    private static let finalMessageDuration = 2
    private static let subscriptionVerificationRetryInterval: Duration = .seconds(2)
    private static let logger = Logger(label: "ProcessingAccount")

    private let prepareAccount: @MainActor (Bool) async throws -> Void
    private let canPrefetchZkNyms: @MainActor () -> Bool
    @ObservationIgnored private var processingTask: Task<Void, Never>?
    @ObservationIgnored private var finalMessageTask: Task<Void, Never>?
    @ObservationIgnored private var minimumPacingTask: Task<Void, Never>?
    // Combine publisher subscription; @Observable tracks $accountSetupPhase in CredentialsManager directly.
    // credentialsManager is captured strongly in prepareAccount closure and outlives this viewModel.
    @ObservationIgnored private var phaseCancellable: AnyCancellable?
    @ObservationIgnored private var carouselSessionCancellable: AnyCancellable?
    @ObservationIgnored public var onFinished: (() -> Void)?

    let flow: ProcessingFlow
    var currentStep: Int = 1
    var didFinishAnimatingText = false
    var didShowFinalMessage = false
    var errorMessage: String?
    var titlesSessionID: UUID
    private var didSettleAccount = false
    private var processingStartedAt: Date?
    private var carouselTicksSinceStart = 0

    public init(
        credentialsManager: CredentialsManager,
        flow: ProcessingFlow,
        canPrefetchZkNyms: @escaping @MainActor () -> Bool = { true },
        carouselSessionID: UUID? = nil
    ) {
        let mode = flow.processingMode
        self.prepareAccount = { canPrefetch in
            try await ProcessingAccountCoordinator.prepare(
                credentialsManager: credentialsManager,
                mode: mode,
                canPrefetchZkNyms: canPrefetch
            )
        }
        self.flow = flow
        self.canPrefetchZkNyms = canPrefetchZkNyms
        self.titlesSessionID = OnboardingSession.shared.carouselSessionID
        carouselSessionCancellable = OnboardingSession.shared.$carouselSessionID
            .receive(on: DispatchQueue.main)
            .sink { [weak self] newID in
                self?.titlesSessionID = newID
            }
        phaseCancellable = credentialsManager.$accountSetupPhase
            .receive(on: DispatchQueue.main)
            .sink { [weak self] phase in
                self?.syncCarouselStep(for: phase)
            }
    }

    init(
        flow: ProcessingFlow,
        canPrefetchZkNyms: @escaping @MainActor () -> Bool = { true },
        carouselSessionID: UUID? = nil,
        prepareAccount: @escaping @MainActor (Bool) async throws -> Void
    ) {
        self.flow = flow
        self.canPrefetchZkNyms = canPrefetchZkNyms
        self.prepareAccount = prepareAccount
        self.titlesSessionID = carouselSessionID ?? OnboardingSession.shared.carouselSessionID
        if carouselSessionID == nil {
            carouselSessionCancellable = OnboardingSession.shared.$carouselSessionID
                .receive(on: DispatchQueue.main)
                .sink { [weak self] newID in
                    self?.titlesSessionID = newID
                }
        }
    }

    func start() {
        processingTask?.cancel()
        resetProcessingState()
        processingStartedAt = Date()
        carouselTicksSinceStart = 0
        processingTask = Task { @MainActor [weak self] in
            guard let self else { return }
            do {
                try await prepareAccount(canPrefetchZkNyms())
                guard !Task.isCancelled else { return }
                settleAccount()
            } catch is CancellationError {
                return
            } catch {
                guard !Task.isCancelled else { return }
#if os(iOS)
                let logDetail = ProcessingAccountErrorMapper.logSafeDescription(for: error)
                Self.logger.error("Account processing failed flow=\(String(describing: self.flow)) \(logDetail)")
                if case CredentialsManagerError.subscriptionVerifying = error {
                    errorMessage = CredentialsManagerError.subscriptionVerifying.localizedTitle
                    scheduleSubscriptionVerificationRetry()
                    return
                }
                if let credentialsError = error as? CredentialsManagerError,
                   let title = credentialsError.localizedTitle {
                    errorMessage = title
                } else {
                    errorMessage = ProcessingAccountErrorMapper.localizedMessage(for: error)
                }
#else
                Self.logger.error(
                    "Account processing failed flow=\(String(describing: self.flow), privacy: .public) errorType=\(String(describing: Swift.type(of: error)), privacy: .public)"
                )
                errorMessage = "generalNymError.somethingWentWrong".localizedString
#endif
            }
        }
    }

    func cancel() {
        processingTask?.cancel()
        processingTask = nil
        finalMessageTask?.cancel()
        finalMessageTask = nil
        minimumPacingTask?.cancel()
        minimumPacingTask = nil
    }

    func retry() {
        start()
    }

    func animationDidAdvance() {
        carouselTicksSinceStart += 1
        let maxStep = flow.carouselStepCount
        currentStep = min(currentStep + 1, maxStep)
        tryCompleteAfterMinimumPacing()
    }

    func animationDidFinish() {
        didFinishAnimatingText = true
        advanceIfReady()
    }

    var loopsCarouselUntilWorkCompletes: Bool {
        true
    }

    func syncCarouselStep(for phase: AccountSetupPhase) {
        guard let step = Self.carouselStep(for: phase, flow: flow) else { return }
        if step > currentStep {
            currentStep = step
        }
    }

    static func carouselStep(for phase: AccountSetupPhase, flow: ProcessingFlow) -> Int? {
        AccountSetupPhase.carouselStep(for: phase, postPurchase: flow == .postPurchase)
    }

    private func settleAccount() {
        didSettleAccount = true
        tryCompleteAfterMinimumPacing()
    }

    private var hasMetMinimumCarouselPacing: Bool {
        guard let processingStartedAt else { return carouselTicksSinceStart >= 1 }
        let elapsed = Date().timeIntervalSince(processingStartedAt)
        return carouselTicksSinceStart >= 1 || elapsed >= Self.minimumCarouselPacing
    }

    private func tryCompleteAfterMinimumPacing() {
        guard didSettleAccount, !didShowFinalMessage else { return }
        guard hasMetMinimumCarouselPacing else {
            scheduleMinimumPacingWait()
            return
        }
        minimumPacingTask?.cancel()
        minimumPacingTask = nil
        // Don't force didFinishAnimatingText=true here; let SwitchingTitlesView control animation completion.
        // Carousel completion and work completion are independent: minimum pacing gates final message display,
        // but animation timing is owned by the view layer via animationDidFinish() callback.
        advanceIfReady()
    }

    private func scheduleMinimumPacingWait() {
        guard minimumPacingTask == nil, let processingStartedAt else { return }
        let elapsed = Date().timeIntervalSince(processingStartedAt)
        let remaining = max(0, Self.minimumCarouselPacing - elapsed)
        minimumPacingTask = Task { @MainActor [weak self] in
            if remaining > 0 {
                try? await Task.sleep(for: .seconds(remaining))
            }
            guard !Task.isCancelled, let self else { return }
            self.minimumPacingTask = nil
            self.tryCompleteAfterMinimumPacing()
        }
    }

    private func advanceIfReady() {
        guard didSettleAccount, didFinishAnimatingText, !didShowFinalMessage else { return }
        didShowFinalMessage = true
        finalMessageTask = Task { @MainActor [weak self] in
            try? await Task.sleep(for: .seconds(ProcessingAccountViewModel.finalMessageDuration))
            guard !Task.isCancelled else { return }
            self?.onFinished?()
        }
    }

    private func resetProcessingState() {
        finalMessageTask?.cancel()
        finalMessageTask = nil
        minimumPacingTask?.cancel()
        minimumPacingTask = nil
        didSettleAccount = false
        didFinishAnimatingText = false
        didShowFinalMessage = false
        processingStartedAt = nil
        carouselTicksSinceStart = 0
        currentStep = 1
        errorMessage = nil
        titlesSessionID = OnboardingSession.shared.carouselSessionID
    }

    private func scheduleSubscriptionVerificationRetry() {
        processingTask = Task { @MainActor [weak self] in
            try? await Task.sleep(for: Self.subscriptionVerificationRetryInterval)
            guard !Task.isCancelled, let self else { return }
            guard OnboardingSession.shared.isWithinPostPurchaseVerificationGracePeriod() else { return }
            self.errorMessage = nil
            self.start()
        }
    }
}
