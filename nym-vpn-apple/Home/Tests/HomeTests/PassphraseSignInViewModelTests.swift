import Foundation
import Testing
import AccountPrefetchGates
import CredentialsManager
@testable import Home

@MainActor
private final class FakePassphraseSignInCredentialStore: PassphraseSignInCredentialStore {
    enum Failure: Error {
        case storeFailed
    }

    var storeError: Error?
    var accountActive = true
    private(set) var storedCredentials: [String] = []
    private(set) var summarySyncCount = 0
    private(set) var ensureResolvedCount = 0

    func storeLoginCredential(_ credential: String) async throws {
        if let storeError {
            throw storeError
        }
        storedCredentials.append(credential)
    }

    func isAccountActive() -> Bool {
        accountActive
    }

    func updateAccountSummary(force: Bool, untilActive: Bool) async {
        summarySyncCount += 1
    }

    func ensureCredentialImportResolved() async {
        ensureResolvedCount += 1
    }
}

@MainActor
private final class FakePassphraseSignInCoordinator: AppSessionCoordinating {
    private(set) var actions: [CoordinatorAction] = []

    func handle(_ action: CoordinatorAction) {
        actions.append(action)
    }
}

@MainActor
private enum PassphraseSignInTestSupport {
    static func firstAuthWillBegin(
        from actions: [CoordinatorAction]
    ) -> (AuthFlowKind, Bool)? {
        guard case let .session(.authWillBegin(flow, completesOnImport)) = actions.first else {
            return nil
        }
        return (flow, completesOnImport)
    }

    static func lastAuthCompleted(
        from actions: [CoordinatorAction]
    ) -> (AuthCompletionOutcome, AuthFlowKind)? {
        guard case let .session(.authCompleted(outcome, flow)) = actions.last else {
            return nil
        }
        return (outcome, flow)
    }
}

@MainActor
struct PassphraseSignInViewModelTests {
    @Test func credentialsManagerConformsToPassphraseSignInCredentialStore() {
        let store: PassphraseSignInCredentialStore = CredentialsManager.shared
        _ = store.isAccountActive()
    }

    @Test func successfulLoginEmitsAuthCompletedOnce() async {
        let store = FakePassphraseSignInCredentialStore()
        store.accountActive = true
        let coordinator = FakePassphraseSignInCoordinator()
        let viewModel = PassphraseSignInViewModel(credentialStore: store)
        viewModel.sessionCoordinator = coordinator
        viewModel.passphraseText = "alpha beta gamma"

        viewModel.loginButtonTapped()
        await viewModel.waitForLoginTask()

        #expect(store.storedCredentials == ["alpha beta gamma"])
        #expect(store.summarySyncCount == 1)
        #expect(store.storedCredentials.count == 1)

        let begin = PassphraseSignInTestSupport.firstAuthWillBegin(from: coordinator.actions)
        #expect(begin?.0 == .login)
        #expect(begin?.1 == false)

        let completed = PassphraseSignInTestSupport.lastAuthCompleted(from: coordinator.actions)
        #expect(completed?.0 == .loginReady)
        #expect(completed?.1 == .login)
        #expect(coordinator.actions.count == 2)
    }

    @Test func inactiveAccountEmitsRegisteredNeedsPurchase() async {
        let store = FakePassphraseSignInCredentialStore()
        store.accountActive = false
        let coordinator = FakePassphraseSignInCoordinator()
        let viewModel = PassphraseSignInViewModel(credentialStore: store)
        viewModel.sessionCoordinator = coordinator
        viewModel.passphraseText = "alpha beta gamma"

        viewModel.loginButtonTapped()
        await viewModel.waitForLoginTask()

        let completed = PassphraseSignInTestSupport.lastAuthCompleted(from: coordinator.actions)
        #expect(completed?.0 == .registeredNeedsPurchase)
        #expect(completed?.1 == .login)
    }

    @Test func secondSubmitWhileLoadingIsIgnored() async {
        let store = FakePassphraseSignInCredentialStore()
        store.accountActive = true
        let coordinator = FakePassphraseSignInCoordinator()
        let viewModel = PassphraseSignInViewModel(credentialStore: store)
        viewModel.sessionCoordinator = coordinator
        viewModel.passphraseText = "alpha beta gamma"

        viewModel.loginButtonTapped()
        #expect(viewModel.submissionState == .loading)

        viewModel.loginButtonTapped()
        await viewModel.waitForLoginTask()

        #expect(store.storedCredentials == ["alpha beta gamma"])
        #expect(coordinator.actions.count == 2)
    }

    @Test func storeFailureCancelsHandoffWithoutSecondStore() async {
        let store = FakePassphraseSignInCredentialStore()
        store.storeError = FakePassphraseSignInCredentialStore.Failure.storeFailed
        let coordinator = FakePassphraseSignInCoordinator()
        let viewModel = PassphraseSignInViewModel(credentialStore: store)
        viewModel.sessionCoordinator = coordinator
        viewModel.passphraseText = "alpha beta gamma"

        viewModel.loginButtonTapped()
        await viewModel.waitForLoginTask()

        #expect(store.storedCredentials.isEmpty)
        #expect(viewModel.submissionState == .failed)
        guard case .session(.authHandoffCancelled) = coordinator.actions.last else {
            Issue.record("Expected authHandoffCancelled")
            return
        }
        let hasAuthCompleted = coordinator.actions.contains { action in
            if case .session(.authCompleted) = action {
                return true
            }
            return false
        }
        #expect(!hasAuthCompleted)
    }

    @Test func alreadyStoredAccountCompletesLoginWithoutSnackbar() async {
        struct AlreadyStored: Error, LocalizedError {
            var errorDescription: String? { "an account is already stored" }
        }
        let store = FakePassphraseSignInCredentialStore()
        store.storeError = AlreadyStored()
        store.accountActive = false
        let coordinator = FakePassphraseSignInCoordinator()
        let viewModel = PassphraseSignInViewModel(credentialStore: store)
        viewModel.sessionCoordinator = coordinator
        viewModel.passphraseText = "alpha beta gamma"

        viewModel.loginButtonTapped()
        await viewModel.waitForLoginTask()

        #expect(store.storedCredentials.isEmpty)
        #expect(store.ensureResolvedCount == 1)
        #expect(viewModel.submissionState == .idle)
        let completed = PassphraseSignInTestSupport.lastAuthCompleted(from: coordinator.actions)
        #expect(completed?.0 == .registeredNeedsPurchase)
        #expect(completed?.1 == .login)
        let cancelled = coordinator.actions.contains { action in
            if case .session(.authHandoffCancelled) = action {
                return true
            }
            return false
        }
        #expect(!cancelled)
    }
}
