import Foundation
import Constants
import CredentialsManager

@Observable
public final class DeeplinkManager {
    private let credentialsManager: CredentialsManager

    public var onPrivyLoginDeeplink: ((String) -> Void)?

    public init(credentialsManager: CredentialsManager) {
        self.credentialsManager = credentialsManager
    }

    public func handle(url: URL) {
        Task { @MainActor in
            await handleURL(url)
        }
    }

    @MainActor
    public func handleURL(_ url: URL) async {
        guard url.scheme == Constants.appUrlScheme.rawValue,
              let components = URLComponents(url: url, resolvingAgainstBaseURL: true)
        else {
            return
        }

        // Privy login
        if components.host == "auth", components.path == "/privy/privateKey" {
            if let onPrivyLoginDeeplink {
                onPrivyLoginDeeplink(url.absoluteString)
            } else {
                try? await credentialsManager.storeDeeplink(callbackURLString: url.absoluteString)
            }
            return
        }
        // Account response
        if components.host == "account", components.path == "/response" {
            let hasDeeplinkId = components.queryItems?.contains(where: { $0.name == "deeplink_id" }) == true
            if hasDeeplinkId {
                // nymvpn://account/response?deeplink_id=...&payload=...
                try? await credentialsManager.storeDeeplink(callbackURLString: url.absoluteString)
            } else {
                // nymvpn://account/response (subscription payment)
                try? await credentialsManager.handleSubscriptionPayment()
            }
        }
    }
}
