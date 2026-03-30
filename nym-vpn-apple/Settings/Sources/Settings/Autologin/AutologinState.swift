import Foundation
import ConnectionTypes
import CredentialsManager

@Observable
@MainActor
final class AutologinState {
    var isPinCodeDisplayed = false
    var pinCode: String = ""
    var isLoading = false
    var isError = false
    var errorMessage = ""
    var url = ""
    var task: Task<Void, Never>?

    func start(kind: NymDeeplinkKind, using credentialsManager: CredentialsManager) {
        isLoading = true
        task = Task {
            await perform(kind: kind, using: credentialsManager)
        }
    }

    func perform(kind: NymDeeplinkKind, using credentialsManager: CredentialsManager) async {
        do {
            guard let result = try await credentialsManager.autologin(kind: kind) else {
                isLoading = false
                return
            }
            isLoading = false
            pinCode = result.pinCode
            url = result.url
            isPinCodeDisplayed = true
        } catch is CancellationError {
            isLoading = false
        } catch {
            isLoading = false
            errorMessage = error.localizedDescription
            isError = true
        }
    }

    func cancel() {
        task?.cancel()
        task = nil
        isLoading = false
    }
}
