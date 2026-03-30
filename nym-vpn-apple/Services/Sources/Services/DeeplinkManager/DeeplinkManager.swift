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
                try await credentialsManager.storeDeeplink(callbackURLString: url.absoluteString)
            }
        }
        // Account response
        if components.host == "account", components.path == "/response" {
            let hasDeeplinkId = components.queryItems?.contains(where: { $0.name == "deeplink_id" }) == true
            Task {
                if hasDeeplinkId {
                    // nymvpn://account/response?deeplink_id=...&payload=...
                    try await credentialsManager.storeDeeplink(callbackURLString: url.absoluteString)
                } else {
                    // nymvpn://account/response (subscription payment)
                    try await credentialsManager.handleSubscriptionPayment()
                }
            }
        }
    }
}
