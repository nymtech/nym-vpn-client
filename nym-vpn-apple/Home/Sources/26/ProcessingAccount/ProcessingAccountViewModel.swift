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
    var errorMessage: String?
    private var didSettleAccount = false

    public init(credentialsManager: CredentialsManager, flow: ProcessingFlow) {
        self.credentialsManager = credentialsManager
        self.flow = flow
    }

    func start() {
        processingTask?.cancel()
        errorMessage = nil
        processingTask = Task { @MainActor [weak self] in
            guard let self else { return }
            do {
                try await credentialsManager.prepareAccountForConnection()
                guard !Task.isCancelled else { return }
                didSettleAccount = true
                advanceIfReady()
            } catch is CancellationError {
                return
            } catch {
                guard !Task.isCancelled else { return }
                errorMessage = error.localizedDescription
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
        didSettleAccount = false
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
}
