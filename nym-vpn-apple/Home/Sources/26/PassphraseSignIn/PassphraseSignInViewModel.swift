import Foundation
import SwiftUI
import AccountPrefetchGates
import CredentialsManager
import SnackbarManager
import Theme
#if os(iOS)
import ErrorHandler
#endif

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
    @ObservationIgnored public var onWillRegister: (() -> Void)?
    @ObservationIgnored public var onAuthHandoffCancelled: (() -> Void)?
    @ObservationIgnored public var onAuthCompleted: ((AuthCompletionOutcome) -> Void)?

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
                onWillRegister?()
                try await credentialsManager.performAccountRegistration(loginCredential: credential)
                passphraseText = ""
                submissionState = .idle
                let outcome: AuthCompletionOutcome = credentialsManager.isAccountActive()
                    ? .loginReady
                    : .registeredNeedsPurchase
                onAuthCompleted?(outcome)
            } catch is CancellationError {
                onAuthHandoffCancelled?()
            } catch let error as VPNErrorReason {
                onAuthHandoffCancelled?()
                submissionState = .failed
                SnackbarManager.shared.enqueue(
                    SnackbarItem(
                        style: .critical,
                        title: "error".localizedString,
                        message: error.localizedDescription
                    )
                )
            } catch {
                onAuthHandoffCancelled?()
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
