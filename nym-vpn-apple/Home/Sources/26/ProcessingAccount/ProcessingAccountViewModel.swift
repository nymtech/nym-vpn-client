import Foundation
import SwiftUI
import CredentialsManager
import UIComponents

public enum ProcessingFlow: Sendable {
    case createAccount
    case login
    case postPurchase
}

@MainActor
@Observable
public final class ProcessingAccountViewModel {
    /// Pacing for the post-auth carousel while account prep and zk-nym prefetch run in parallel.
    public static let processingStepInterval = SwitchingTitlesView.accountProcessingStepInterval
    private static let finalMessageDuration = 2

    private let prepareAccountForConnection: @MainActor (Bool) async throws -> Void
    private let canPrefetchZkNyms: @MainActor () -> Bool
    @ObservationIgnored private var processingTask: Task<Void, Never>?
    @ObservationIgnored private var finalMessageTask: Task<Void, Never>?
    @ObservationIgnored public var onFinished: (() -> Void)?

    let flow: ProcessingFlow
    var currentStep: Int = 1
    var didFinishAnimatingText = false
    var didShowFinalMessage = false
    var errorMessage: String?
    var titlesSessionID = UUID()
    private var didSettleAccount = false

    public init(
        credentialsManager: CredentialsManager,
        flow: ProcessingFlow,
        canPrefetchZkNyms: @escaping @MainActor () -> Bool = { true }
    ) {
        self.prepareAccountForConnection = {
            try await credentialsManager.prepareAccountForConnection(canPrefetchZkNyms: $0)
        }
        self.flow = flow
        self.canPrefetchZkNyms = canPrefetchZkNyms
    }

    init(
        flow: ProcessingFlow,
        canPrefetchZkNyms: @escaping @MainActor () -> Bool = { true },
        prepareAccountForConnection: @escaping @MainActor (Bool) async throws -> Void
    ) {
        self.flow = flow
        self.canPrefetchZkNyms = canPrefetchZkNyms
        self.prepareAccountForConnection = prepareAccountForConnection
    }

    func start() {
        processingTask?.cancel()
        resetProcessingState()
        processingTask = Task { @MainActor [weak self] in
            guard let self else { return }
            do {
                try await prepareAccountForConnection(canPrefetchZkNyms())
                guard !Task.isCancelled else { return }
                didSettleAccount = true
                advanceIfReady()
            } catch is CancellationError {
                return
            } catch {
                guard !Task.isCancelled else { return }
                errorMessage = "generalNymError.somethingWentWrong".localizedString
            }
        }
    }

    func cancel() {
        processingTask?.cancel()
        processingTask = nil
        finalMessageTask?.cancel()
        finalMessageTask = nil
    }

    func retry() {
        start()
    }

    func animationDidAdvance() {
        currentStep += 1
    }

    func animationDidFinish() {
        didFinishAnimatingText = true
        advanceIfReady()
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
        didSettleAccount = false
        didFinishAnimatingText = false
        didShowFinalMessage = false
        currentStep = 1
        errorMessage = nil
        titlesSessionID = UUID()
    }
}
