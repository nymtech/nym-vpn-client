import Foundation

public enum WebCheckoutReturnPolicy: Equatable, Sendable {
    public static func shouldDismissOnDeeplink(host: String?, path: String?, hasDeeplinkId: Bool) -> Bool {
        guard host == "account", path == "/response" else {
            return false
        }
        return !hasDeeplinkId
    }

    public static func shouldDismissOnDeeplink(url: URL) -> Bool {
        guard url.scheme == "nymvpn",
              let components = URLComponents(url: url, resolvingAgainstBaseURL: true)
        else {
            return false
        }
        let hasDeeplinkId = components.queryItems?.contains(where: { $0.name == "deeplink_id" }) == true
        return shouldDismissOnDeeplink(
            host: components.host,
            path: components.path,
            hasDeeplinkId: hasDeeplinkId
        )
    }
}
