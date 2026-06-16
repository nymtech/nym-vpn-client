import Foundation
import SwiftUI
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

    var currentStep: Int = 1
    var didFinishAnimatingText = false
    var didRegisterAccount = false
    var errorMessage: String?

    private var isRegistering = false

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
                if !(await credentialsManager.isAccountStored()) {
                    try await credentialsManager.createMnemonic()
                }
                onWillRegister?()
                try await credentialsManager.registerAccount()
                didRegisterAccount = true
            } catch is CancellationError {
                return
            } catch let error as VPNErrorReason {
                didRegisterAccount = false
                errorMessage = error.localizedDescription
            } catch {
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
    }

    func retry() {
        errorMessage = nil
        start()
    }

    func dismissError() {
        errorMessage = nil
    }
}
