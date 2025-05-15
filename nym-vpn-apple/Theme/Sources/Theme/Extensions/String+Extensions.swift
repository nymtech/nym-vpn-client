import Foundation

public extension String {
    var localizedString: String {
        Bundle.main.localizedString(forKey: self)
    }
}
