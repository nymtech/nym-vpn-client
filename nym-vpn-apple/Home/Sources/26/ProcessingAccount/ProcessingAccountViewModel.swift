import Foundation
import SwiftUI
import CredentialsManager

public enum ProcessingFlow: Sendable {
    case createAccount
    case login
    case postPurchase
}

@MainActor
@Observable
public final class ProcessingAccountViewModel {
    private static let finalMessageDuration = 2

    private let credentialsManager: CredentialsManager
    @ObservationIgnored private var processingTask: Task<Void, Never>?
    @ObservationIgnored private var finalMessageTask: Task<Void, Never>?
    @ObservationIgnored public var onFinished: (() -> Void)?

    let flow: ProcessingFlow
    var currentStep: Int = 1
    var didFinishAnimatingText = false
    var didShowFinalMessage = false
    private var didBecomeActive = false

    public init(credentialsManager: CredentialsManager, flow: ProcessingFlow) {
        self.credentialsManager = credentialsManager
        self.flow = flow
    }

    func start() {
        processingTask?.cancel()
        processingTask = Task { @MainActor [weak self] in
            guard let self else { return }
            await credentialsManager.updateAccountSummary(force: true, untilActive: true)
            guard !Task.isCancelled else { return }
            if AccountZkNymPrefetchGate.shouldPrefetchAfterSummarySync(
                isAccountActive: credentialsManager.isAccountActive()
            ) {
                _ = await credentialsManager.prefetchZkNyms()
                guard !Task.isCancelled else { return }
            }
            didBecomeActive = true
            advanceIfReady()
        }
    }

    func cancel() {
        processingTask?.cancel()
        processingTask = nil
        finalMessageTask?.cancel()
        finalMessageTask = nil
    }

    func animationDidAdvance() {
        currentStep += 1
    }

    func animationDidFinish() {
        didFinishAnimatingText = true
        advanceIfReady()
    }

    private func advanceIfReady() {
        guard didBecomeActive, didFinishAnimatingText, !didShowFinalMessage else { return }
        didShowFinalMessage = true
        finalMessageTask = Task { @MainActor [weak self] in
            try? await Task.sleep(for: .seconds(ProcessingAccountViewModel.finalMessageDuration))
            guard !Task.isCancelled else { return }
            self?.onFinished?()
        }
    }
}
