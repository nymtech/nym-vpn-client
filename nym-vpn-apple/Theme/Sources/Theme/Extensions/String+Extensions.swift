import Foundation

public extension String {
    var localizedString: String {
        let catalog = String(localized: String.LocalizationValue(self), bundle: .main)
        if catalog != self {
            return catalog
        }
        return Bundle.main.localizedStringFallback(forKey: self)
    }
}
