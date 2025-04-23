import SwiftUI
import Theme

public class LocalizationManager: ObservableObject {
    private static let groupID = "group.net.nymtech.vpn"
    private static let languageKey = "language"
    private let userDefaults: UserDefaults?

    public static let shared = LocalizationManager()

    @Published public var language: String {
        didSet {
            userDefaults?.set(language, forKey: Self.languageKey)
        }
    }

    private var bundle: Bundle {
        guard let path = ThemeResources.bundle.path(forResource: language, ofType: "lproj"),
              let localizedBundle = Bundle(path: path)
        else {
            return ThemeResources.bundle
        }
        return localizedBundle
    }

    public init() {
        let currentLanguageCode = Locale.current.language.languageCode?.identifier
        self.userDefaults = UserDefaults(suiteName: Self.groupID)
        self.language = userDefaults?.string(forKey: Self.languageKey) ?? currentLanguageCode ?? "en"
    }

    public func localizedString(forKey key: String) -> String {
        NSLocalizedString(key, bundle: bundle, comment: "")
    }
}
