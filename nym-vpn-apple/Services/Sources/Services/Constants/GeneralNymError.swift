import Foundation
import Theme

public enum GeneralNymError: Error, Equatable {
    case invalidUrl
    case cannotFetchCountries
    case library(message: String)
    case invalidCredential
}

extension GeneralNymError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .invalidUrl:
            return "generalNymError.invalidUrl".localizedString
        case .cannotFetchCountries:
            return "generalNymError.cannotFetchCountries".localizedString
        case .library(message: let message):
            return message
        case .invalidCredential:
            return "error.noValidCredential".localizedString
        }
    }
}
