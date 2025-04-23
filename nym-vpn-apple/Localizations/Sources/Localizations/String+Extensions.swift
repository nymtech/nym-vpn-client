import Foundation

public extension String {
    var localizedString: String {
        LocalizationManager.shared.localizedString(forKey: self)
    }
}
