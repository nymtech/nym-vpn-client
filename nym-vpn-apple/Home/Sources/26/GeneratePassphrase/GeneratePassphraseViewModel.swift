import Foundation
import SwiftUI
import AccountPrefetchGates
import CredentialsManager
#if os(iOS)
import ErrorHandler
#endif

@MainActor
@Observable
public final class GeneratePassphraseViewModel {
    private let credentialsManager: CredentialsManager
    @ObservationIgnored private var registrationTask: Task<Void, Never>?
    @ObservationIgnored public var onWillRegister: (() -> Void)?
    @ObservationIgnored public var onAuthHandoffCancelled: (() -> Void)?
    @ObservationIgnored public var onAuthCompleted: ((AuthCompletionOutcome) -> Void)?

    var currentStep: Int = 1
    var didFinishAnimatingText = false
    var didRegisterAccount = false
    var errorMessage: String?

    private var isRegistering = false
    private var didEmitAuthCompleted = false

    public init(credentialsManager: CredentialsManager) {
        self.credentialsManager = credentialsManager
    }

    func start() {
        guard !isRegistering, !didRegisterAccount else { return }
        isRegistering = true
        errorMessage = nil
        registrationTask?.cancel()
        registrationTask = Task { @MainActor [weak self] in
            guard let self else { return }
            defer { self.isRegistering = false }
            do {
                onWillRegister?()
                try await credentialsManager.performAccountRegistration()
                didRegisterAccount = true
                advanceIfAuthComplete()
            } catch is CancellationError {
                onAuthHandoffCancelled?()
                return
            } catch let error as VPNErrorReason {
                onAuthHandoffCancelled?()
                didRegisterAccount = false
                errorMessage = error.localizedDescription
            } catch {
                onAuthHandoffCancelled?()
                didRegisterAccount = false
                errorMessage = error.localizedDescription
            }
        }
    }

    func animationDidAdvance() {
        currentStep += 1
    }

    func animationDidFinish() {
        didFinishAnimatingText = true
        advanceIfAuthComplete()
    }

    private func advanceIfAuthComplete() {
        guard didRegisterAccount, didFinishAnimatingText, !didEmitAuthCompleted else { return }
        didEmitAuthCompleted = true
        let outcome: AuthCompletionOutcome = credentialsManager.isAccountActive()
            ? .registeredActive
            : .registeredNeedsPurchase
        onAuthCompleted?(outcome)
    }

    func retry() {
        errorMessage = nil
        start()
    }

    func dismissError() {
        errorMessage = nil
    }
}
