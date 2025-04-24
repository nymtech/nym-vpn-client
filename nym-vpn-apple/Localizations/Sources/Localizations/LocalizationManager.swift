import SwiftUI
import Theme

public class LocalizationManager: ObservableObject {
    private static let groupID = "group.net.nymtech.vpn"
    private static let languageKey = "language"

    public static let shared = LocalizationManager()

    private init() {}

    public func localizedString(forKey key: String) -> String {
        NSLocalizedString(key, bundle: .main, comment: "")
    }
}
