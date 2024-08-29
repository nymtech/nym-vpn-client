import Foundation
import Theme

public enum GeneralNymError: Error, Equatable {
    case invalidUrl
    case cannotFetchCountries
    case noPrebundledCountries
    case cannotParseCountries
    case library(message: String)
    case invalidCredential
    case noEnvFile
}

extension GeneralNymError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .invalidUrl:
            return "generalNymError.invalidUrl".localizedString
        case .cannotFetchCountries:
            return "generalNymError.cannotFetchCountries".localizedString
        case .noPrebundledCountries:
            return "generalNymError.noPrebundledCountries".localizedString
        case .cannotParseCountries:
            return "generalNymError.cannotParseCountries".localizedString
        case .library(message: let message):
            return message
        case .invalidCredential:
            return "error.noValidCredential".localizedString
        case .noEnvFile:
            return "generalNymError.noEnvFile".localizedString
        }
    }
}
