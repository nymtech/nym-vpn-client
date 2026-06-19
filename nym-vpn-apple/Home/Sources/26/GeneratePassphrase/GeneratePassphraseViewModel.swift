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
    @ObservationIgnored public weak var sessionCoordinator: AppSessionCoordinating?

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
                sessionCoordinator?.handleSessionEvent(
                    .authWillBegin(flow: .createAccount, completesOnCredentialImport: false)
                )
                try await credentialsManager.performAccountRegistration()
                didRegisterAccount = true
                advanceIfAuthComplete()
            } catch is CancellationError {
                sessionCoordinator?.handleSessionEvent(.authHandoffCancelled)
                return
            } catch let error as VPNErrorReason {
                sessionCoordinator?.handleSessionEvent(.authHandoffCancelled)
                didRegisterAccount = false
                errorMessage = error.localizedDescription
            } catch {
                sessionCoordinator?.handleSessionEvent(.authHandoffCancelled)
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
        Task { @MainActor [weak self] in
            guard let self else { return }
            let outcome = await AuthCompletionOutcomeResolver.resolve(
                flow: .createAccount,
                isAccountActive: { credentialsManager.isAccountActive() },
                updateAccountSummary: { untilActive in
                    await credentialsManager.updateAccountSummary(
                        force: true,
                        untilActive: untilActive
                    )
                }
            )
            sessionCoordinator?.handleSessionEvent(
                .authCompleted(outcome: outcome, flow: .createAccount)
            )
        }
    }

    func retry() {
        errorMessage = nil
        start()
    }

    func dismissError() {
        errorMessage = nil
    }
}
