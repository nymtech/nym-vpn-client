import Foundation
import SwiftUI
import AccountPrefetchGates
import CredentialsManager
import SnackbarManager
import Theme

@MainActor
@Observable
public final class PassphraseSignInViewModel {
    enum SubmissionState: Equatable {
        case idle
        case loading
        case failed
    }

    private let credentialsManager: CredentialsManager
    @ObservationIgnored private var loginTask: Task<Void, Never>?
    @ObservationIgnored public weak var sessionCoordinator: AppSessionCoordinating?

    var passphraseText: String = "" {
        didSet {
            if submissionState == .failed {
                submissionState = .idle
            }
        }
    }

    var submissionState: SubmissionState = .idle

    public init(credentialsManager: CredentialsManager) {
        self.credentialsManager = credentialsManager
    }

    func loginButtonTapped() {
        let credential = passphraseText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !credential.isEmpty else { return }
        submissionState = .loading
        loginTask?.cancel()
        loginTask = Task { @MainActor [weak self] in
            guard let self else { return }
            do {
                sessionCoordinator?.handleSessionEvent(
                    .authWillBegin(flow: .login, completesOnCredentialImport: false)
                )
#if os(iOS)
                try await credentialsManager.performAccountRegistration(loginCredential: credential)
#else
                try await credentialsManager.add(credential: credential)
#endif
                passphraseText = ""
                let outcome = await AuthCompletionOutcomeResolver.resolve(
                    flow: .login,
                    isAccountActive: { self.credentialsManager.isAccountActive() },
                    updateAccountSummary: { untilActive in
                        await self.credentialsManager.updateAccountSummary(
                            force: true,
                            untilActive: untilActive
                        )
                    }
                )
                submissionState = .idle
                sessionCoordinator?.handleSessionEvent(
                    .authCompleted(outcome: outcome, flow: .login)
                )
            } catch is CancellationError {
                sessionCoordinator?.handleSessionEvent(.authHandoffCancelled)
            } catch {
                sessionCoordinator?.handleSessionEvent(.authHandoffCancelled)
                submissionState = .failed
                SnackbarManager.shared.enqueue(
                    SnackbarItem(
                        style: .critical,
                        title: "error".localizedString,
                        message: error.localizedDescription
                    )
                )
            }
        }
    }
}
