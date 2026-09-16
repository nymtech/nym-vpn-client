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

    private let credentialStore: PassphraseSignInCredentialStore
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
        self.credentialStore = credentialsManager
    }

    init(credentialStore: PassphraseSignInCredentialStore) {
        self.credentialStore = credentialStore
    }

    func loginButtonTapped() {
        guard submissionState != .loading else { return }
        let credential = passphraseText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !credential.isEmpty else { return }
        submissionState = .loading
        loginTask?.cancel()
        loginTask = Task { @MainActor [weak self] in
            guard let self else { return }
            do {
                sessionCoordinator?.handle(
                    .session(.authWillBegin(flow: .login, completesOnCredentialImport: false))
                )
                try await storeCredentialTreatingExistingAccountAsSuccess(credential)
                await completeLoginAfterStore()
            } catch is CancellationError {
                sessionCoordinator?.handle(.session(.authHandoffCancelled))
                submissionState = .idle
            } catch {
                sessionCoordinator?.handle(.session(.authHandoffCancelled))
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

    private func storeCredentialTreatingExistingAccountAsSuccess(_ credential: String) async throws {
        do {
            try await credentialStore.storeLoginCredential(credential)
        } catch {
            guard OnboardingSessionPolicy.isExistingAccountStoreError(error) else { throw error }
            await credentialStore.ensureCredentialImportResolved()
        }
    }

    private func completeLoginAfterStore() async {
        let outcome = await AuthCompletionOutcomeResolver.resolveAfterLoginRegistration(
            isAccountActive: { self.credentialStore.isAccountActive() },
            updateAccountSummary: {
                await self.credentialStore.updateAccountSummary(force: true, untilActive: false)
            }
        )
        sessionCoordinator?.handle(
            .session(.authCompleted(outcome: outcome, flow: .login))
        )
        submissionState = .idle
    }

    func waitForLoginTask() async {
        await loginTask?.value
    }
}
