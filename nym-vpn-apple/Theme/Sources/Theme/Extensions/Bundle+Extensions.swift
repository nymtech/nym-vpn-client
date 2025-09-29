import Foundation

public extension Bundle {
    /// Lookup order: current localization → en.lproj → Base.lproj → key
    func localizedStringFallback(forKey key: String, table: String? = nil) -> String {
        let missing = "\u{FFFF}\u{FFFF}"

        // 1) Current localization
        let primary = localizedString(forKey: key, value: missing, table: table)
        if primary != missing {
            return primary
        }

        // 2) English
        if let enPath = path(forResource: "en", ofType: "lproj"),
           let enBundle = Bundle(path: enPath) {
            let en = enBundle.localizedString(forKey: key, value: missing, table: table)
            if en != missing {
                return en
            }
        }

        // 3) Base
        if let basePath = path(forResource: "Base", ofType: "lproj"),
           let baseBundle = Bundle(path: basePath) {
            let base = baseBundle.localizedString(forKey: key, value: missing, table: table)
            if base != missing {
                return base
            }
        }

        // 4) Fallback to key
        return key
    }
}
