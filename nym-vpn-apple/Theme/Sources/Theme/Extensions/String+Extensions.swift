import Foundation

public extension String {
    var localizedString: String {
        Bundle.main.localizedStringFallback(forKey: self)
    }
}
