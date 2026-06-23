import Foundation
import ConnectionTypes
import CredentialsManager

@Observable
@MainActor
public final class AutologinState {
    public var isPinCodeDisplayed = false
    public var pinCode: String = ""
    public var isLoading = false
    public var isError = false
    public var errorMessage = ""
    public var url = ""
    public var task: Task<Void, Never>?

    public init() {}

    public func start(kind: NymDeeplinkKind, using credentialsManager: CredentialsManager) {
        task?.cancel()
        isError = false
        isLoading = true
        task = Task {
            await perform(kind: kind, using: credentialsManager)
        }
    }

    public func perform(kind: NymDeeplinkKind, using credentialsManager: CredentialsManager) async {
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

    public func cancel() {
        task?.cancel()
        task = nil
        isLoading = false
    }

    public func dismissAfterWebReturn() {
        cancel()
        isPinCodeDisplayed = false
        isError = false
        pinCode = ""
        url = ""
    }
}
