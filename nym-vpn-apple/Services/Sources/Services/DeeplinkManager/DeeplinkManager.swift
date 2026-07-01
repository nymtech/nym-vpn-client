import Foundation
import Constants
import ConnectionManager
import CredentialsManager

@Observable
public final class DeeplinkManager {
    private let credentialsManager: CredentialsManager
    private let connectionManager: ConnectionManager

    public var onPrivyLoginDeeplink: ((String) -> Void)?

    public init(credentialsManager: CredentialsManager, connectionManager: ConnectionManager) {
        self.credentialsManager = credentialsManager
        self.connectionManager = connectionManager
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

        // VPN toggle from widget
        if components.host == "vpn", components.path == "/toggle" {
            try? await connectionManager.connectDisconnect()
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
