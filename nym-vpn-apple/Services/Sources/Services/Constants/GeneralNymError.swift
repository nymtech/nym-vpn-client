import Foundation
import Theme

public enum GeneralNymError: Error {
    case invalidUrl
    case cannotFetchCountries
}

extension GeneralNymError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .invalidUrl:
            return "generalNymError.invalidUrl".localizedString
        case .cannotFetchCountries:
            return "generalNymError.cannotFetchCountries".localizedString
        }
    }
}
