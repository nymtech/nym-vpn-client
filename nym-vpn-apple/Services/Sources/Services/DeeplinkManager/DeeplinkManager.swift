import Foundation
import Constants
import CredentialsManager

@Observable
public final class DeeplinkManager {
    private let credentialsManager: CredentialsManager

    public init (credentialsManager: CredentialsManager) {
        self.credentialsManager = credentialsManager
    }

    public func handle(url: URL) {
        guard url.scheme == Constants.appUrlScheme.rawValue,
              let components = URLComponents(url: url, resolvingAgainstBaseURL: true)
        else {
            return
        }

        // Privy login
        if components.host == "auth", components.path == "/privy/privateKey" {
            Task {
                try await credentialsManager.privyLoginStore(callbackURLString: url.path())
            }
        }
    }
}
