import Foundation
import SwiftUI
import CredentialsManager

@MainActor
@Observable
public final class GeneratePassphraseViewModel {
    private let isValidCredentialImported: () -> Bool
    private let createMnemonic: () async throws -> Void
    private let registerAccount: () async throws -> Void
    @ObservationIgnored private var registrationTask: Task<Void, Never>?
    @ObservationIgnored public var onWillRegister: (() -> Void)?
    @ObservationIgnored public var onAuthComplete: (() -> Void)?

    var currentStep: Int = 1
    var didFinishAnimatingText = false
    var didRegisterAccount = false
    var errorMessage: String?

    private var isRegistering = false

    public init(credentialsManager: CredentialsManager) {
        self.isValidCredentialImported = { credentialsManager.isValidCredentialImported }
        self.createMnemonic = { try await credentialsManager.createMnemonic() }
        self.registerAccount = { try await credentialsManager.registerAccount() }
    }

    init(
        isValidCredentialImported: @escaping () -> Bool = { true },
        createMnemonic: @escaping () async throws -> Void = {},
        registerAccount: @escaping () async throws -> Void
    ) {
        self.isValidCredentialImported = isValidCredentialImported
        self.createMnemonic = createMnemonic
        self.registerAccount = registerAccount
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
                if !isValidCredentialImported() {
                    try await createMnemonic()
                }
                try await registerAccount()
                didRegisterAccount = true
                onAuthComplete?()
            } catch is CancellationError {
                return
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
