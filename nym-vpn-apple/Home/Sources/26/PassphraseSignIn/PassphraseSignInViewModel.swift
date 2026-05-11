import Foundation
import SwiftUI
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
                try await credentialsManager.add(credential: credential)
                try await credentialsManager.registerAccount()
                passphraseText = ""
                submissionState = .idle
            } catch is CancellationError {
                // Cancelled — keep current state.
            } catch {
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
